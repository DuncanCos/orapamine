//! Représentation d'un plateau 10×8 rempli à partir d'un placement de
//! pièces. Utilisé par `beam.rs` (tracé du faisceau), `placement.rs`
//! (validation) et `solution.rs` (comparaison de solutions).

use crate::geometry::{Cell, CellKind};
use crate::pieces::{GemColor, PieceCatalog, Special};
use serde::{Deserialize, Serialize};

/// Une pièce posée sur le plateau : quelle pièce, dans quelle orientation
/// (index dans `PieceDef::orientations`), et à quel point d'ancrage (coin
/// haut-gauche de sa boîte englobante).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacedPiece {
    pub piece_id: String,
    pub orientation: usize,
    pub anchor_x: i32,
    pub anchor_y: i32,
}

/// Contenu d'une case occupée du plateau.
#[derive(Debug, Clone, Copy)]
pub struct BoardCell {
    /// Index de la pièce posée (dans la liste de placements d'origine),
    /// utilisé pour les contraintes de contact et d'atteignabilité.
    pub placement_index: usize,
    pub kind: CellKind,
    pub color: Option<GemColor>,
    pub special: Option<Special>,
}

#[derive(Debug, Clone)]
pub struct Board {
    pub width: i32,
    pub height: i32,
    cells: Vec<Option<BoardCell>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildBoardError {
    OutOfBounds { placement_index: usize },
    Overlap { placement_index: usize },
    UnknownPiece { piece_id: String },
    InvalidOrientation { piece_id: String, orientation: usize },
}

impl Board {
    pub fn empty(width: i32, height: i32) -> Board {
        Board {
            width,
            height,
            cells: vec![None; (width * height) as usize],
        }
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width && y < self.height
    }

    fn index(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&BoardCell> {
        if !self.in_bounds(x, y) {
            return None;
        }
        self.cells[self.index(x, y)].as_ref()
    }

    /// Visible dans le crate uniquement : `placement.rs` s'en sert pour
    /// construire un plateau "au mieux" (premier occupant gagne) lors du
    /// calcul d'atteignabilité, même en présence d'un chevauchement déjà
    /// signalé par ailleurs comme violation.
    pub(crate) fn set(&mut self, x: i32, y: i32, cell: BoardCell) {
        let idx = self.index(x, y);
        self.cells[idx] = Some(cell);
    }

    /// Construit le plateau à partir d'une liste de pièces posées. Échoue
    /// si une pièce sort de la grille ou chevauche une autre (les autres
    /// contraintes, elles, sont vérifiées séparément par `placement.rs` et
    /// rapportées comme violations plutôt que comme erreurs dures).
    pub fn build(
        catalog: &PieceCatalog,
        width: i32,
        height: i32,
        placements: &[PlacedPiece],
    ) -> Result<Board, BuildBoardError> {
        let mut board = Board::empty(width, height);
        for (i, p) in placements.iter().enumerate() {
            let piece = catalog
                .piece(&p.piece_id)
                .ok_or_else(|| BuildBoardError::UnknownPiece {
                    piece_id: p.piece_id.clone(),
                })?;
            let shape: &Vec<Cell> =
                piece
                    .orientations
                    .get(p.orientation)
                    .ok_or_else(|| BuildBoardError::InvalidOrientation {
                        piece_id: p.piece_id.clone(),
                        orientation: p.orientation,
                    })?;
            for c in shape {
                let x = p.anchor_x + c.x;
                let y = p.anchor_y + c.y;
                if !board.in_bounds(x, y) {
                    return Err(BuildBoardError::OutOfBounds { placement_index: i });
                }
                if board.get(x, y).is_some() {
                    return Err(BuildBoardError::Overlap { placement_index: i });
                }
                board.set(
                    x,
                    y,
                    BoardCell {
                        placement_index: i,
                        kind: c.kind,
                        color: piece.color,
                        special: piece.special,
                    },
                );
            }
        }
        Ok(board)
    }
}
