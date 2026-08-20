//! Point d'entrée WebSocket : établissement de connexion (création /
//! jonction / reconnexion / solo, qui ont besoin d'`AppState`), puis
//! routage des actions de jeu vers `logic.rs` une fois la connexion liée à
//! une partie + un index de joueur.

use crate::logic;
use crate::protocol::{ClientMsg, RoomOptions, ServerMsg};
use crate::state::{AppState, Game, Kind};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use orapa_core::{random_valid_placement, GameOptions};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use uuid::Uuid;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut ws_tx, mut ws_rx) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    let writer = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut bound: Option<(Arc<Mutex<Game>>, usize)> = None;

    while let Some(Ok(msg)) = ws_rx.next().await {
        let text = match msg {
            Message::Text(t) => t,
            Message::Close(_) => break,
            _ => continue,
        };
        let parsed: Result<ClientMsg, _> = serde_json::from_str(&text);
        let client_msg = match parsed {
            Ok(m) => m,
            Err(e) => {
                let _ = tx.send(err_message("bad_message", &e.to_string()));
                continue;
            }
        };

        if bound.is_none() {
            match client_msg {
                ClientMsg::CreateGame { pseudo, options } => {
                    let (game_arc, idx, token, code) = create_duel(&state, pseudo, options);
                    {
                        let mut game = game_arc.lock().unwrap();
                        game.senders[idx] = Some(tx.clone());
                    }
                    let _ = tx.send(json_message(&ServerMsg::GameCreated {
                        code,
                        token: token.to_string(),
                        player_index: idx,
                    }));
                    bound = Some((game_arc.clone(), idx));
                    let game = game_arc.lock().unwrap();
                    logic::broadcast_state(&game);
                }
                ClientMsg::JoinGame { code, pseudo } => match join_duel(&state, &code, pseudo) {
                    Ok((game_arc, idx, token)) => {
                        {
                            let mut game = game_arc.lock().unwrap();
                            game.senders[idx] = Some(tx.clone());
                        }
                        let _ = tx.send(json_message(&ServerMsg::Joined {
                            token: token.to_string(),
                            player_index: idx,
                        }));
                        bound = Some((game_arc.clone(), idx));
                        let game = game_arc.lock().unwrap();
                        logic::broadcast_state(&game);
                    }
                    Err(message) => {
                        let _ = tx.send(err_message("join_failed", &message));
                    }
                },
                ClientMsg::Reconnect { token } => match reconnect(&state, &token) {
                    Ok((game_arc, idx)) => {
                        {
                            let mut game = game_arc.lock().unwrap();
                            game.senders[idx] = Some(tx.clone());
                            game.players[idx].connected = true;
                        }
                        bound = Some((game_arc.clone(), idx));
                        let game = game_arc.lock().unwrap();
                        logic::broadcast_state(&game);
                    }
                    Err(message) => {
                        let _ = tx.send(err_message("reconnect_failed", &message));
                    }
                },
                ClientMsg::StartSolo { pseudo, options } => {
                    let (game_arc, idx, token, code) = start_solo(&state, pseudo, options);
                    {
                        let mut game = game_arc.lock().unwrap();
                        game.senders[idx] = Some(tx.clone());
                    }
                    let _ = tx.send(json_message(&ServerMsg::GameCreated {
                        code,
                        token: token.to_string(),
                        player_index: idx,
                    }));
                    bound = Some((game_arc.clone(), idx));
                    let game = game_arc.lock().unwrap();
                    logic::broadcast_state(&game);
                }
                _ => {
                    let _ = tx.send(err_message(
                        "not_bound",
                        "Il faut créer, rejoindre ou reprendre une partie en premier.",
                    ));
                }
            }
            continue;
        }

        let (game_arc, idx) = bound.clone().unwrap();
        let reschedule_timer = matches!(
            client_msg,
            ClientMsg::ReadyToPlay | ClientMsg::FireBeam { .. } | ClientMsg::Probe { .. } | ClientMsg::SubmitSolution { .. }
        );
        {
            let mut game = game_arc.lock().unwrap();
            match client_msg {
                ClientMsg::SetPlacement { pieces } => logic::set_placement(&mut game, idx, pieces),
                ClientMsg::RequestRandomPlacement => logic::request_random_placement(&game, idx),
                ClientMsg::ReadyToPlay => logic::ready_to_play(&mut game, idx),
                ClientMsg::FireBeam { entry } => logic::fire_beam(&mut game, idx, entry),
                ClientMsg::Probe { x, y } => logic::probe(&mut game, idx, x, y),
                ClientMsg::SubmitSolution { pieces } => logic::submit_solution(&mut game, idx, pieces),
                ClientMsg::CheckHypothesis { pieces } => logic::check_hypothesis(&game, idx, pieces),
                ClientMsg::Reaction { id } => logic::reaction(&game, idx, id),
                ClientMsg::RequestRematch => logic::request_rematch(&mut game, idx),
                ClientMsg::RevealSolution => logic::reveal_solution(&game, idx),
                ClientMsg::CreateGame { .. }
                | ClientMsg::JoinGame { .. }
                | ClientMsg::Reconnect { .. }
                | ClientMsg::StartSolo { .. } => {
                    let _ = tx.send(err_message(
                        "already_bound",
                        "Cette connexion est déjà liée à une partie.",
                    ));
                }
            }
        }
        if reschedule_timer {
            logic::schedule_timer_if_needed(game_arc);
        }
    }

    if let Some((game_arc, idx)) = bound {
        let mut game = game_arc.lock().unwrap();
        game.players[idx].connected = false;
        game.senders[idx] = None;
        logic::broadcast_state(&game);
    }
    writer.abort();
}

fn err_message(code: &str, message: &str) -> Message {
    json_message(&ServerMsg::Error {
        code: code.to_string(),
        message: message.to_string(),
    })
}

fn json_message(msg: &ServerMsg) -> Message {
    Message::Text(serde_json::to_string(msg).unwrap_or_else(|_| "{}".to_string()))
}

fn create_duel(
    state: &AppState,
    pseudo: String,
    options: RoomOptions,
) -> (Arc<Mutex<Game>>, usize, Uuid, String) {
    let code = state.fresh_code();
    let mut game = Game::new_duel(code.clone(), state.catalog.clone(), options);
    let (idx, token) = logic::add_player(&mut game, pseudo);
    let game_arc = Arc::new(Mutex::new(game));
    state.games.insert(code.clone(), game_arc.clone());
    state.tokens.insert(token, code.clone());
    (game_arc, idx, token, code)
}

fn join_duel(
    state: &AppState,
    code: &str,
    pseudo: String,
) -> Result<(Arc<Mutex<Game>>, usize, Uuid), String> {
    let code = code.to_uppercase();
    let game_arc = state
        .games
        .get(&code)
        .map(|g| g.clone())
        .ok_or_else(|| "Code de partie inconnu.".to_string())?;
    let token = {
        let mut game = game_arc.lock().unwrap();
        if game.kind != Kind::Duel {
            return Err("Cette partie n'accepte pas de second joueur.".to_string());
        }
        if game.players.len() >= 2 {
            return Err("Cette partie est déjà complète.".to_string());
        }
        let (idx, token) = logic::add_player(&mut game, pseudo);
        debug_assert_eq!(idx, 1);
        if game.players.len() == 2 {
            game.phase = crate::protocol::Phase::Placement;
        }
        token
    };
    state.tokens.insert(token, code);
    let idx = 1;
    Ok((game_arc, idx, token))
}

fn reconnect(state: &AppState, token_str: &str) -> Result<(Arc<Mutex<Game>>, usize), String> {
    let token = Uuid::parse_str(token_str).map_err(|_| "Jeton invalide.".to_string())?;
    let code = state
        .tokens
        .get(&token)
        .map(|c| c.clone())
        .ok_or_else(|| "Partie introuvable pour ce jeton.".to_string())?;
    let game_arc = state
        .games
        .get(&code)
        .map(|g| g.clone())
        .ok_or_else(|| "Partie introuvable.".to_string())?;
    let idx = {
        let game = game_arc.lock().unwrap();
        game.find_player(token)
            .ok_or_else(|| "Joueur introuvable dans cette partie.".to_string())?
    };
    Ok((game_arc, idx))
}

fn start_solo(
    state: &AppState,
    pseudo: String,
    options: RoomOptions,
) -> (Arc<Mutex<Game>>, usize, Uuid, String) {
    let code = state.fresh_code();
    let mut rng = SmallRng::from_entropy();
    let game_options = GameOptions {
        diamond: options.diamond,
        black: options.black,
    };
    let secret = random_valid_placement(&mut rng, &state.catalog, game_options);
    let mut game = Game::new_solo(code.clone(), state.catalog.clone(), options, secret);
    let (idx, token) = logic::add_player(&mut game, pseudo);
    let game_arc = Arc::new(Mutex::new(game));
    state.games.insert(code.clone(), game_arc.clone());
    state.tokens.insert(token, code.clone());
    (game_arc, idx, token, code)
}
