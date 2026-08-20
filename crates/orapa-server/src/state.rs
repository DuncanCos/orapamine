//! État serveur : registre des parties, machine à états par partie.
//! Persistance en mémoire uniquement (voir plan §1) : une partie survit à
//! un rechargement de page ou une coupure réseau (via `player_token`), pas
//! à un redémarrage du serveur.

use crate::protocol::{HistoryEntry, Phase, RoomOptions};
use axum::extract::ws::Message;
use dashmap::DashMap;
use orapa_core::{GameOptions, PieceCatalog, PlacedPiece};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Duel,
    Solo,
}

pub struct Player {
    pub token: Uuid,
    pub pseudo: String,
    pub connected: bool,
    pub ready: bool,
    pub lives: u8,
}

pub struct Game {
    pub code: String,
    pub kind: Kind,
    pub options: RoomOptions,
    pub catalog: Arc<PieceCatalog>,
    pub players: Vec<Player>,
    /// `secrets[i]` est la disposition secrète que défend `players[i]`
    /// (pour le solo, un seul joueur humain à l'index 0 défend un plateau
    /// vide non-jouable ; `secrets[1]` est la disposition aléatoire de
    /// l'"adversaire" généré par le serveur).
    pub secrets: Vec<Option<Vec<PlacedPiece>>>,
    pub phase: Phase,
    pub current_turn: usize,
    pub first_player_index: usize,
    pub sudden_death: bool,
    pub finisher: Option<usize>,
    pub outcome: Option<Option<usize>>,
    pub history: Vec<HistoryEntry>,
    pub rematch_requested: Vec<bool>,
    /// Compteur incrémenté à chaque changement de tour, pour annuler la
    /// tâche de timer précédente sans avoir à la traquer explicitement.
    pub turn_generation: u64,
    pub senders: Vec<Option<UnboundedSender<Message>>>,
}

impl Game {
    pub fn new_duel(code: String, catalog: Arc<PieceCatalog>, options: RoomOptions) -> Game {
        Game {
            code,
            kind: Kind::Duel,
            options,
            catalog,
            players: Vec::new(),
            secrets: vec![None, None],
            phase: Phase::Lobby,
            current_turn: 0,
            first_player_index: 0,
            sudden_death: false,
            finisher: None,
            outcome: None,
            history: Vec::new(),
            rematch_requested: vec![false, false],
            turn_generation: 0,
            senders: vec![None, None],
        }
    }

    pub fn new_solo(
        code: String,
        catalog: Arc<PieceCatalog>,
        options: RoomOptions,
        secret: Vec<PlacedPiece>,
    ) -> Game {
        Game {
            code,
            kind: Kind::Solo,
            options,
            catalog,
            players: Vec::new(),
            secrets: vec![None, Some(secret)],
            phase: Phase::Playing,
            current_turn: 0,
            first_player_index: 0,
            sudden_death: false,
            finisher: None,
            outcome: None,
            history: Vec::new(),
            rematch_requested: vec![false],
            turn_generation: 0,
            senders: vec![None],
        }
    }

    pub fn opponent_of(&self, idx: usize) -> usize {
        1 - idx
    }

    pub fn game_options(&self) -> GameOptions {
        GameOptions {
            diamond: self.options.diamond,
            black: self.options.black,
        }
    }

    pub fn find_player(&self, token: Uuid) -> Option<usize> {
        self.players.iter().position(|p| p.token == token)
    }
}

pub struct AppState {
    pub games: DashMap<String, Arc<Mutex<Game>>>,
    /// Index token -> code de partie, pour la reconnexion sans connaître
    /// le code.
    pub tokens: DashMap<Uuid, String>,
    pub catalog: Arc<PieceCatalog>,
    code_counter: AtomicU64,
}

impl AppState {
    pub fn new() -> Arc<AppState> {
        Arc::new(AppState {
            games: DashMap::new(),
            tokens: DashMap::new(),
            catalog: Arc::new(PieceCatalog::default_catalog()),
            code_counter: AtomicU64::new(0),
        })
    }

    /// Code de partie lisible à 5 lettres, garanti unique parmi les
    /// parties actuellement en mémoire.
    pub fn fresh_code(&self) -> String {
        const LETTERS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ";
        loop {
            let n = self.code_counter.fetch_add(1, Ordering::Relaxed);
            let mut seed = n
                ^ (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(0));
            let mut code = String::with_capacity(5);
            for _ in 0..5 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let idx = (seed >> 33) as usize % LETTERS.len();
                code.push(LETTERS[idx] as char);
            }
            if !self.games.contains_key(&code) {
                return code;
            }
        }
    }
}
