//! Protocole WebSocket (JSON, `#[serde(tag = "type")]`) entre client et
//! serveur, tel que décrit au §5 du plan. Le serveur ne renvoie jamais les
//! placements secrets adverses avant la fin de partie.

use orapa_core::{BeamOutcome, GemColor, PlacedPiece, Violation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoomOptions {
    #[serde(default)]
    pub diamond: bool,
    #[serde(default)]
    pub black: bool,
    /// Nombre de vies avant élimination (1 = règle officielle, 2 = variante
    /// plus permissive proposée en lobby).
    #[serde(default = "default_lives")]
    pub lives: u8,
    #[serde(default)]
    pub turn_timer_secs: Option<u64>,
    /// Mode aide : le client peut demander au serveur de rejouer les
    /// indices déjà reçus contre une hypothèse (voir `CheckHypothesis`).
    #[serde(default)]
    pub help_mode: bool,
}

fn default_lives() -> u8 {
    1
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMsg {
    CreateGame { pseudo: String, options: RoomOptions },
    JoinGame { code: String, pseudo: String },
    Reconnect { token: String },
    StartSolo { pseudo: String, options: RoomOptions },
    SetPlacement { pieces: Vec<PlacedPiece> },
    RequestRandomPlacement,
    ReadyToPlay,
    FireBeam { entry: String },
    Probe { x: i32, y: i32 },
    SubmitSolution { pieces: Vec<PlacedPiece> },
    /// Mode aide : vérifie une hypothèse complète contre tout l'historique
    /// reçu jusqu'ici, sans consommer de tour ni toucher l'état de partie.
    CheckHypothesis { pieces: Vec<PlacedPiece> },
    Reaction { id: String },
    RequestRematch,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProbeOutcome {
    Empty,
    Color(GemColor),
    /// Gemme présente mais sans couleur (corps noir ou diamant).
    OccupiedNoColor,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind")]
pub enum HistoryEntry {
    Beam {
        actor: usize,
        entry: String,
        outcome: BeamOutcome,
    },
    Probe {
        actor: usize,
        x: i32,
        y: i32,
        result: ProbeOutcome,
    },
    Solution {
        actor: usize,
        correct: bool,
    },
    Timeout {
        actor: usize,
    },
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Lobby,
    Placement,
    Playing,
    Finished,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerPublicInfo {
    pub pseudo: String,
    pub connected: bool,
    pub ready: bool,
    pub has_placement: bool,
    pub lives: u8,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ServerMsg {
    GameCreated {
        code: String,
        token: String,
        player_index: usize,
    },
    Joined {
        token: String,
        player_index: usize,
    },
    StateUpdate {
        phase: Phase,
        players: Vec<PlayerPublicInfo>,
        your_index: usize,
        current_turn: Option<usize>,
        first_player_index: Option<usize>,
        sudden_death: bool,
        options: RoomOptions,
        history: Vec<HistoryEntry>,
        /// La disposition déjà soumise par ce joueur lui-même (jamais
        /// secrète pour son propre auteur), pour que le client puisse la
        /// restaurer après un rechargement de page ou une reconnexion —
        /// sans cela, la partie "reprend" côté serveur mais le joueur se
        /// retrouve face à une grille vide alors qu'il avait déjà posé ses
        /// gemmes. `None` tant qu'il n'a rien soumis.
        your_placement: Option<Vec<PlacedPiece>>,
    },
    PlacementRejected {
        violations: Vec<Violation>,
    },
    RandomPlacement {
        pieces: Vec<PlacedPiece>,
    },
    BeamResult {
        entry: String,
        outcome: BeamOutcome,
    },
    ProbeResult {
        x: i32,
        y: i32,
        result: ProbeOutcome,
    },
    SolutionResult {
        correct: bool,
    },
    HypothesisCheckResult {
        consistent: bool,
        contradicting_entries: Vec<String>,
    },
    GameOver {
        winner: Option<usize>,
        boards: Vec<Vec<PlacedPiece>>,
        history: Vec<HistoryEntry>,
    },
    ReactionReceived {
        player_index: usize,
        id: String,
    },
    Error {
        code: String,
        message: String,
    },
}
