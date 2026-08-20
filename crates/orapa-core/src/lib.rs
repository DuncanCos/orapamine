//! `orapa-core` : moteur de jeu pur d'Orapa Mine — géométrie des pièces,
//! validation de placement, tracé du faisceau, mélange des couleurs et
//! comparaison de solutions. Aucune dépendance UI ni réseau : ce module est
//! partagé tel quel par le serveur (via Rust natif) et par le client (via
//! WebAssembly, voir `orapa-wasm`).

pub mod beam;
pub mod board;
pub mod colors;
pub mod generator;
pub mod geometry;
pub mod pieces;
pub mod placement;
pub mod solution;

pub use beam::{entry_points, fire_beam, fire_beam_traced, BeamOutcome, EntryPoint, PointIndex};
pub use board::{Board, BoardCell, PlacedPiece};
pub use colors::{mix_colors, ResultColor};
pub use generator::{random_valid_placement, GameOptions};
pub use pieces::{Expansion, GemColor, PieceCatalog, PieceDef, Special};
pub use placement::{validate_placement, Violation};
pub use solution::compare_solution;
