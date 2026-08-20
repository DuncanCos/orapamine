//! Logique d'une action de jeu déjà résolue vers (partie, index joueur).
//! La création/jonction/reconnexion de partie (qui a besoin d'`AppState`)
//! vit dans `ws.rs` ; ce module ne connaît que la partie en cours.
//!
//! Autorité serveur stricte (plan §5) : toute la géométrie (tracé du
//! faisceau, sondage, comparaison de solution) passe par `orapa-core` ;
//! aucune disposition secrète n'est jamais sérialisée vers l'adversaire
//! avant `GameOver`.

use crate::protocol::{HistoryEntry, Phase, PlayerPublicInfo, ProbeOutcome, ServerMsg};
use crate::state::{Game, Kind, Player};
use axum::extract::ws::Message;
use orapa_core::{
    compare_solution, fire_beam as core_fire_beam, random_valid_placement, validate_placement,
    Board, PieceCatalog, PlacedPiece, PointIndex,
};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

pub fn add_player(game: &mut Game, pseudo: String) -> (usize, Uuid) {
    let token = Uuid::new_v4();
    let lives = game.options.lives;
    game.players.push(Player {
        token,
        pseudo,
        connected: true,
        ready: false,
        lives,
    });
    (game.players.len() - 1, token)
}

fn send(game: &Game, idx: usize, msg: &ServerMsg) {
    if let Some(Some(tx)) = game.senders.get(idx) {
        if let Ok(json) = serde_json::to_string(msg) {
            let _ = tx.send(Message::Text(json));
        }
    }
}

fn broadcast(game: &Game, msg: &ServerMsg) {
    for i in 0..game.players.len() {
        send(game, i, msg);
    }
}

pub fn state_update_for(game: &Game, viewer: usize) -> ServerMsg {
    let players = game
        .players
        .iter()
        .enumerate()
        .map(|(i, p)| PlayerPublicInfo {
            pseudo: p.pseudo.clone(),
            connected: p.connected,
            ready: p.ready,
            has_placement: game.secrets.get(i).map(|s| s.is_some()).unwrap_or(false),
            lives: p.lives,
        })
        .collect();
    ServerMsg::StateUpdate {
        phase: game.phase,
        players,
        your_index: viewer,
        current_turn: if game.phase == Phase::Playing {
            Some(game.current_turn)
        } else {
            None
        },
        first_player_index: if game.phase == Phase::Playing || game.phase == Phase::Finished {
            Some(game.first_player_index)
        } else {
            None
        },
        sudden_death: game.sudden_death,
        options: game.options.clone(),
        history: game.history.clone(),
        your_placement: game.secrets.get(viewer).and_then(|s| s.clone()),
    }
}

pub fn broadcast_state(game: &Game) {
    for i in 0..game.players.len() {
        let msg = state_update_for(game, i);
        send(game, i, &msg);
    }
}

pub fn set_placement(game: &mut Game, idx: usize, pieces: Vec<PlacedPiece>) {
    if game.phase != Phase::Placement {
        send(
            game,
            idx,
            &ServerMsg::Error {
                code: "wrong_phase".into(),
                message: "Le placement n'est possible que pendant la phase de placement.".into(),
            },
        );
        return;
    }
    match validate_placement(&game.catalog, game.catalog.grid_width, game.catalog.grid_height, &pieces) {
        Ok(violations) if violations.is_empty() => {
            game.secrets[idx] = Some(pieces);
            broadcast_state(game);
        }
        Ok(violations) => {
            send(game, idx, &ServerMsg::PlacementRejected { violations });
        }
        Err(message) => {
            send(
                game,
                idx,
                &ServerMsg::Error {
                    code: "invalid_placement".into(),
                    message,
                },
            );
        }
    }
}

pub fn request_random_placement(game: &Game, idx: usize) {
    let mut rng = SmallRng::from_entropy();
    let pieces = random_valid_placement(&mut rng, &game.catalog, game.game_options());
    send(game, idx, &ServerMsg::RandomPlacement { pieces });
}

pub fn ready_to_play(game: &mut Game, idx: usize) {
    if game.phase != Phase::Placement || game.secrets[idx].is_none() {
        send(
            game,
            idx,
            &ServerMsg::Error {
                code: "not_placed".into(),
                message: "Pose ta disposition avant de te déclarer prêt.".into(),
            },
        );
        return;
    }
    game.players[idx].ready = true;
    let all_ready = game.players.iter().all(|p| p.ready)
        && game.secrets.iter().all(|s| s.is_some());
    if all_ready {
        let mut rng = SmallRng::from_entropy();
        game.first_player_index = rng.gen_range(0..game.players.len());
        game.current_turn = game.first_player_index;
        game.phase = Phase::Playing;
    }
    broadcast_state(game);
}

fn opponent_board(game: &Game, idx: usize) -> Option<Board> {
    let secret = game.secrets.get(game.opponent_of(idx)).and_then(|s| s.as_ref())?;
    Board::build(&game.catalog, game.catalog.grid_width, game.catalog.grid_height, secret).ok()
}

fn ensure_turn(game: &Game, idx: usize) -> Result<(), &'static str> {
    if game.phase != Phase::Playing {
        return Err("La partie n'est pas en cours.");
    }
    if game.kind == Kind::Duel && game.current_turn != idx {
        return Err("Ce n'est pas ton tour.");
    }
    Ok(())
}

pub fn fire_beam(game: &mut Game, idx: usize, entry: String) {
    if let Err(message) = ensure_turn(game, idx) {
        send(
            game,
            idx,
            &ServerMsg::Error {
                code: "wrong_turn".into(),
                message: message.into(),
            },
        );
        return;
    }
    let Some(board) = opponent_board(game, idx) else {
        send(
            game,
            idx,
            &ServerMsg::Error {
                code: "no_opponent_board".into(),
                message: "Le plateau adverse n'est pas encore prêt.".into(),
            },
        );
        return;
    };
    let points = PointIndex::build(game.catalog.grid_width, game.catalog.grid_height);
    let Some(outcome) = core_fire_beam(&board, &points, &entry) else {
        send(
            game,
            idx,
            &ServerMsg::Error {
                code: "invalid_entry".into(),
                message: format!("Point de tir inconnu : {entry}"),
            },
        );
        return;
    };
    game.history.push(HistoryEntry::Beam {
        actor: idx,
        entry: entry.clone(),
        outcome: outcome.clone(),
    });
    send(
        game,
        idx,
        &ServerMsg::BeamResult {
            entry,
            outcome,
        },
    );
    switch_turn(game, idx);
    broadcast_state(game);
}

pub fn probe(game: &mut Game, idx: usize, x: i32, y: i32) {
    if let Err(message) = ensure_turn(game, idx) {
        send(
            game,
            idx,
            &ServerMsg::Error {
                code: "wrong_turn".into(),
                message: message.into(),
            },
        );
        return;
    }
    let Some(board) = opponent_board(game, idx) else {
        send(
            game,
            idx,
            &ServerMsg::Error {
                code: "no_opponent_board".into(),
                message: "Le plateau adverse n'est pas encore prêt.".into(),
            },
        );
        return;
    };
    if !board.in_bounds(x, y) {
        send(
            game,
            idx,
            &ServerMsg::Error {
                code: "out_of_bounds".into(),
                message: "Case hors grille.".into(),
            },
        );
        return;
    }
    let result = match board.get(x, y) {
        None => ProbeOutcome::Empty,
        Some(cell) => match cell.color {
            Some(c) => ProbeOutcome::Color(c),
            None => ProbeOutcome::OccupiedNoColor,
        },
    };
    game.history.push(HistoryEntry::Probe {
        actor: idx,
        x,
        y,
        result: result.clone(),
    });
    send(game, idx, &ServerMsg::ProbeResult { x, y, result });
    switch_turn(game, idx);
    broadcast_state(game);
}

pub fn submit_solution(game: &mut Game, idx: usize, guess: Vec<PlacedPiece>) {
    if let Err(message) = ensure_turn(game, idx) {
        send(
            game,
            idx,
            &ServerMsg::Error {
                code: "wrong_turn".into(),
                message: message.into(),
            },
        );
        return;
    }
    let Some(secret) = game
        .secrets
        .get(game.opponent_of(idx))
        .and_then(|s| s.as_ref())
        .cloned()
    else {
        send(
            game,
            idx,
            &ServerMsg::Error {
                code: "no_opponent_board".into(),
                message: "Le plateau adverse n'est pas encore prêt.".into(),
            },
        );
        return;
    };
    let correct = compare_solution(
        &game.catalog,
        game.catalog.grid_width,
        game.catalog.grid_height,
        &secret,
        &guess,
    )
    .unwrap_or(false);

    game.history.push(HistoryEntry::Solution { actor: idx, correct });
    send(game, idx, &ServerMsg::SolutionResult { correct });

    resolve_solution_outcome(game, idx, correct);
    broadcast_state(game);
    if game.phase == Phase::Finished {
        send_game_over(game);
    }
}

fn resolve_solution_outcome(game: &mut Game, idx: usize, correct: bool) {
    if game.kind == Kind::Solo {
        if correct {
            game.outcome = Some(Some(0));
            game.phase = Phase::Finished;
        } else {
            game.players[idx].lives = game.players[idx].lives.saturating_sub(1);
            if game.players[idx].lives == 0 {
                game.outcome = Some(None);
                game.phase = Phase::Finished;
            }
        }
        return;
    }

    if game.sudden_death {
        // Dernier tour de l'adversaire du premier joueur qui a trouvé.
        let finisher = game.finisher.expect("sudden_death implique finisher");
        game.outcome = Some(if correct { None } else { Some(finisher) });
        game.phase = Phase::Finished;
        return;
    }

    if correct {
        if idx == game.first_player_index {
            game.finisher = Some(idx);
            game.sudden_death = true;
            switch_turn(game, idx);
        } else {
            game.outcome = Some(Some(idx));
            game.phase = Phase::Finished;
        }
    } else {
        game.players[idx].lives = game.players[idx].lives.saturating_sub(1);
        if game.players[idx].lives == 0 {
            game.outcome = Some(Some(game.opponent_of(idx)));
            game.phase = Phase::Finished;
        } else {
            switch_turn(game, idx);
        }
    }
}

fn send_game_over(game: &Game) {
    let boards: Vec<Vec<PlacedPiece>> = game
        .secrets
        .iter()
        .map(|s| s.clone().unwrap_or_default())
        .collect();
    let winner = game.outcome.flatten();
    broadcast(
        game,
        &ServerMsg::GameOver {
            winner,
            boards,
            history: game.history.clone(),
        },
    );
}

fn switch_turn(game: &mut Game, actor: usize) {
    if game.kind == Kind::Duel {
        game.current_turn = game.opponent_of(actor);
    }
    game.turn_generation += 1;
}

/// Mode aide (§2.7) : rejoue tout l'historique des ondes/sondages émis par
/// `idx` contre l'hypothèse fournie et signale les indices contredits.
/// Ne consomme pas de tour, ne modifie pas l'état de partie.
pub fn check_hypothesis(game: &Game, idx: usize, hypothesis: Vec<PlacedPiece>) {
    if !game.options.help_mode {
        send(
            game,
            idx,
            &ServerMsg::Error {
                code: "help_mode_disabled".into(),
                message: "Le mode aide n'est pas activé pour cette partie.".into(),
            },
        );
        return;
    }
    let board = match Board::build(
        &game.catalog,
        game.catalog.grid_width,
        game.catalog.grid_height,
        &hypothesis,
    ) {
        Ok(b) => b,
        Err(_) => {
            send(
                game,
                idx,
                &ServerMsg::HypothesisCheckResult {
                    consistent: false,
                    contradicting_entries: vec!["placement invalide (chevauchement ou hors grille)".into()],
                },
            );
            return;
        }
    };
    let points = PointIndex::build(game.catalog.grid_width, game.catalog.grid_height);
    let mut contradicting = Vec::new();
    for h in &game.history {
        match h {
            HistoryEntry::Beam { actor, entry, outcome } if *actor == idx => {
                if let Some(replayed) = core_fire_beam(&board, &points, entry) {
                    if &replayed != outcome {
                        contradicting.push(format!("onde {entry}"));
                    }
                }
            }
            HistoryEntry::Probe { actor, x, y, result } if *actor == idx => {
                let replayed = match board.get(*x, *y) {
                    None => ProbeOutcome::Empty,
                    Some(cell) => match cell.color {
                        Some(c) => ProbeOutcome::Color(c),
                        None => ProbeOutcome::OccupiedNoColor,
                    },
                };
                if &replayed != result {
                    contradicting.push(format!("sondage ({x},{y})"));
                }
            }
            _ => {}
        }
    }
    send(
        game,
        idx,
        &ServerMsg::HypothesisCheckResult {
            consistent: contradicting.is_empty(),
            contradicting_entries: contradicting,
        },
    );
}

pub fn reaction(game: &Game, idx: usize, id: String) {
    broadcast(game, &ServerMsg::ReactionReceived { player_index: idx, id });
}

/// Mode solo uniquement : révèle la disposition adverse générée par le
/// serveur sans mettre fin à la partie (bouton "solution" du mode
/// entraînement). Refusé en duel pour ne pas permettre de tricher.
pub fn reveal_solution(game: &Game, idx: usize) {
    if game.kind != Kind::Solo {
        send(
            game,
            idx,
            &ServerMsg::Error {
                code: "not_solo".into(),
                message: "La solution ne peut être révélée qu'en mode solo.".into(),
            },
        );
        return;
    }
    let board = game.secrets.get(1).and_then(|s| s.clone()).unwrap_or_default();
    send(game, idx, &ServerMsg::SolutionRevealed { board });
}

pub fn request_rematch(game: &mut Game, idx: usize) {
    if game.phase != Phase::Finished {
        return;
    }
    if let Some(flag) = game.rematch_requested.get_mut(idx) {
        *flag = true;
    }
    let all = game.rematch_requested.iter().all(|&r| r);
    if all {
        reset_for_rematch(game);
    }
    broadcast_state(game);
}

fn reset_for_rematch(game: &mut Game) {
    game.history.clear();
    game.sudden_death = false;
    game.finisher = None;
    game.outcome = None;
    game.turn_generation += 1;
    game.rematch_requested.iter_mut().for_each(|r| *r = false);
    for p in game.players.iter_mut() {
        p.lives = game.options.lives;
        p.ready = false;
    }
    match game.kind {
        Kind::Duel => {
            game.secrets = vec![None, None];
            game.phase = Phase::Placement;
            game.current_turn = 0;
        }
        Kind::Solo => {
            let mut rng = SmallRng::from_entropy();
            let catalog: &PieceCatalog = &game.catalog;
            let secret = random_valid_placement(&mut rng, catalog, game.game_options());
            game.secrets[1] = Some(secret);
            game.phase = Phase::Playing;
            game.current_turn = 0;
        }
    }
}

/// Programme (ou reprogramme) le timer de tour optionnel. À rappeler après
/// toute mutation qui change le tour actif. Le générateur incrémenté à
/// chaque changement de tour permet de détecter qu'un timer devenu obsolète
/// ne doit plus rien faire à son réveil.
pub fn schedule_timer_if_needed(game_arc: Arc<Mutex<Game>>) {
    let (secs, generation, turn, code) = {
        let game = game_arc.lock().unwrap();
        let Some(secs) = game.options.turn_timer_secs else {
            return;
        };
        if game.kind != Kind::Duel || game.phase != Phase::Playing {
            return;
        }
        (secs, game.turn_generation, game.current_turn, game.code.clone())
    };
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(secs)).await;
        let mut game = game_arc.lock().unwrap();
        if game.turn_generation != generation || game.phase != Phase::Playing || game.current_turn != turn {
            return; // le tour a déjà changé, ce timer est obsolète
        }
        tracing::info!(code = %code, player = turn, "tour expiré, passage automatique");
        game.history.push(HistoryEntry::Timeout { actor: turn });
        switch_turn(&mut game, turn);
        broadcast_state(&game);
        drop(game);
        schedule_timer_if_needed(game_arc.clone());
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::RoomOptions;
    use std::sync::Arc;

    fn duel_game(lives: u8) -> Game {
        let options = RoomOptions {
            lives,
            ..Default::default()
        };
        let mut game = Game::new_duel(
            "TEST".to_string(),
            Arc::new(PieceCatalog::default_catalog()),
            options,
        );
        add_player(&mut game, "Alice".to_string());
        add_player(&mut game, "Bob".to_string());
        game.first_player_index = 0;
        game.current_turn = 0;
        game.phase = Phase::Playing;
        game
    }

    #[test]
    fn first_player_finding_it_triggers_sudden_death_not_immediate_win() {
        let mut game = duel_game(1);
        resolve_solution_outcome(&mut game, 0, true);
        assert!(game.sudden_death);
        assert_eq!(game.finisher, Some(0));
        assert_eq!(game.phase, Phase::Playing);
        // Le tour est repassé à l'adversaire pour son dernier essai.
        assert_eq!(game.current_turn, 1);
    }

    #[test]
    fn second_player_finding_it_first_wins_immediately() {
        let mut game = duel_game(1);
        resolve_solution_outcome(&mut game, 1, true);
        assert_eq!(game.phase, Phase::Finished);
        assert_eq!(game.outcome, Some(Some(1)));
    }

    #[test]
    fn sudden_death_then_correct_guess_is_a_draw() {
        let mut game = duel_game(1);
        resolve_solution_outcome(&mut game, 0, true); // premier joueur trouve
        resolve_solution_outcome(&mut game, 1, true); // adversaire égalise
        assert_eq!(game.phase, Phase::Finished);
        assert_eq!(game.outcome, Some(None));
    }

    #[test]
    fn sudden_death_then_wrong_guess_lets_finisher_win() {
        let mut game = duel_game(1);
        resolve_solution_outcome(&mut game, 0, true); // premier joueur trouve
        resolve_solution_outcome(&mut game, 1, false); // dernier essai raté
        assert_eq!(game.phase, Phase::Finished);
        assert_eq!(game.outcome, Some(Some(0)));
    }

    #[test]
    fn wrong_guess_with_one_life_eliminates_the_player() {
        let mut game = duel_game(1);
        resolve_solution_outcome(&mut game, 0, false);
        assert_eq!(game.phase, Phase::Finished);
        assert_eq!(game.outcome, Some(Some(1)));
        assert_eq!(game.players[0].lives, 0);
    }

    #[test]
    fn wrong_guess_with_two_lives_only_costs_one_and_keeps_playing() {
        let mut game = duel_game(2);
        resolve_solution_outcome(&mut game, 0, false);
        assert_eq!(game.phase, Phase::Playing);
        assert_eq!(game.players[0].lives, 1);
        // Le tour passe à l'adversaire.
        assert_eq!(game.current_turn, 1);
    }

    #[test]
    fn is_symmetric_check_hypothesis_reports_empty_when_disabled() {
        let mut game = duel_game(1);
        game.options.help_mode = false;
        // Ne doit rien paniquer même sans destinataire connecté (senders
        // tous à `None` dans ce test) : `send` ignore silencieusement.
        check_hypothesis(&game, 0, vec![]);
        game.options.help_mode = true;
        check_hypothesis(&game, 0, vec![]);
    }
}
