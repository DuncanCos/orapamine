// Types miroir du protocole WebSocket serveur (voir
// crates/orapa-server/src/protocol.rs). Les formes JSON doivent rester
// exactement synchronisées avec les types Rust `#[serde(...)]`.

export type GemColor = "red" | "yellow" | "blue" | "white";

export type ResultColor =
  | "transparent"
  | "red"
  | "blue"
  | "yellow"
  | "white"
  | "purple"
  | "orange"
  | "green"
  | "pink"
  | "sky_blue"
  | "lemon"
  | "black"
  | "light_purple"
  | "light_green"
  | "light_orange"
  | "gray";

export interface PlacedPiece {
  piece_id: string;
  orientation: number;
  anchor_x: number;
  anchor_y: number;
}

export type BeamOutcome =
  | { kind: "Exit"; point: string; color: ResultColor }
  | { kind: "Absorbed" }
  | { kind: "Lost" };

export type ProbeOutcome = "empty" | "occupied_no_color" | { color: GemColor };

export type Violation =
  | { kind: "OutOfBounds"; placement_index: number }
  | { kind: "Overlap"; placement_index: number; other_index: number }
  | { kind: "EdgeContact"; placement_index: number; other_index: number }
  | { kind: "Unreachable"; placement_index: number }
  | { kind: "WhiteSymmetry" };

export type HistoryEntry =
  | { kind: "Beam"; actor: number; entry: string; outcome: BeamOutcome }
  | { kind: "Probe"; actor: number; x: number; y: number; result: ProbeOutcome }
  | { kind: "Solution"; actor: number; correct: boolean }
  | { kind: "Timeout"; actor: number };

export type Phase = "lobby" | "placement" | "playing" | "finished";

export interface RoomOptions {
  diamond: boolean;
  black: boolean;
  lives: number;
  turn_timer_secs: number | null;
  help_mode: boolean;
}

export interface PlayerPublicInfo {
  pseudo: string;
  connected: boolean;
  ready: boolean;
  has_placement: boolean;
  lives: number;
}

// --- Client -> Serveur ---

export type ClientMsg =
  | { type: "CreateGame"; pseudo: string; options: RoomOptions }
  | { type: "JoinGame"; code: string; pseudo: string }
  | { type: "Reconnect"; token: string }
  | { type: "StartSolo"; pseudo: string; options: RoomOptions }
  | { type: "SetPlacement"; pieces: PlacedPiece[] }
  | { type: "RequestRandomPlacement" }
  | { type: "ReadyToPlay" }
  | { type: "FireBeam"; entry: string }
  | { type: "Probe"; x: number; y: number }
  | { type: "SubmitSolution"; pieces: PlacedPiece[] }
  | { type: "CheckHypothesis"; pieces: PlacedPiece[] }
  | { type: "Reaction"; id: string }
  | { type: "RequestRematch" }
  | { type: "RevealSolution" };

// --- Serveur -> Client ---

export type ServerMsg =
  | { type: "GameCreated"; code: string; token: string; player_index: number }
  | { type: "Joined"; token: string; player_index: number }
  | {
      type: "StateUpdate";
      phase: Phase;
      players: PlayerPublicInfo[];
      your_index: number;
      current_turn: number | null;
      first_player_index: number | null;
      sudden_death: boolean;
      options: RoomOptions;
      history: HistoryEntry[];
      your_placement: PlacedPiece[] | null;
    }
  | { type: "PlacementRejected"; violations: Violation[] }
  | { type: "RandomPlacement"; pieces: PlacedPiece[] }
  | { type: "BeamResult"; entry: string; outcome: BeamOutcome }
  | { type: "ProbeResult"; x: number; y: number; result: ProbeOutcome }
  | { type: "SolutionResult"; correct: boolean }
  | { type: "HypothesisCheckResult"; consistent: boolean; contradicting_entries: string[] }
  | { type: "GameOver"; winner: number | null; boards: PlacedPiece[][]; history: HistoryEntry[] }
  | { type: "ReactionReceived"; player_index: number; id: string }
  | { type: "SolutionRevealed"; board: PlacedPiece[] }
  | { type: "Error"; code: string; message: string };

// --- Catalogue de pièces (voir orapa-wasm::catalog_json) ---

export type Corner = "nw" | "ne" | "se" | "sw";
export type CellKind = "square" | `tri_${Corner}`;

export interface Cell {
  x: number;
  y: number;
  kind: CellKind;
}

export type Expansion = "diamond" | "black";
export type Special = "absorb" | "transparent";

export interface PieceDef {
  id: string;
  label_fr: string;
  color: GemColor | null;
  expansion: Expansion | null;
  special: Special | null;
  orientations: Cell[][];
}

export interface PieceCatalog {
  grid_width: number;
  grid_height: number;
  white_symmetry_forbidden: boolean;
  pieces: PieceDef[];
}
