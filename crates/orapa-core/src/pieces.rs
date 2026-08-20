//! Catalogue des pièces, chargé depuis `data/pieces.json` (embarqué au
//! moment de la compilation via `include_str!`). Chaque pièce n'y est
//! décrite que par sa forme de base (orientation 0) ; toutes les
//! orientations valides (rotations 90° et miroir) sont dérivées par code et
//! dédupliquées, pour éviter toute divergence entre une orientation
//! "écrite à la main" et sa forme réelle.

use crate::geometry::{bounding_box, canonical, mirror_horizontal, rotate_cw, Cell, CellKind};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Couleur d'une gemme (les pièces "diamant" et "corps noir" n'ont pas de
/// couleur : `None`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GemColor {
    Red,
    Yellow,
    Blue,
    White,
}

/// Comportement spécial d'une pièce d'extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Special {
    /// Corps noir : absorbe le faisceau.
    Absorb,
    /// Diamant : réfléchit sans altérer la couleur.
    Transparent,
}

/// Extension optionnelle qui introduit cette pièce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Expansion {
    Diamond,
    Black,
}

#[derive(Debug, Clone, Deserialize)]
struct RawCell {
    x: i32,
    y: i32,
    kind: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RawPiece {
    id: String,
    label_fr: String,
    color: Option<GemColor>,
    expansion: Option<Expansion>,
    special: Option<Special>,
    cells: Vec<RawCell>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawGrid {
    width: i32,
    height: i32,
}

#[derive(Debug, Clone, Deserialize)]
struct RawRules {
    white_symmetry_forbidden: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct RawCatalog {
    grid: RawGrid,
    pieces: Vec<RawPiece>,
    rules: RawRules,
}

/// Définition complète d'une pièce : identité + toutes ses orientations
/// valides (dédupliquées), chacune sous forme de liste de cases relatives
/// normalisées (min x = 0, min y = 0).
#[derive(Debug, Clone, Serialize)]
pub struct PieceDef {
    pub id: String,
    pub label_fr: String,
    pub color: Option<GemColor>,
    pub expansion: Option<Expansion>,
    pub special: Option<Special>,
    pub orientations: Vec<Vec<Cell>>,
}

impl PieceDef {
    pub fn area(&self) -> f64 {
        self.orientations[0]
            .iter()
            .map(|c| match c.kind {
                CellKind::Square => 1.0,
                CellKind::Triangle(_) => 0.5,
            })
            .sum()
    }

    pub fn bounding_box(&self, orientation: usize) -> (i32, i32) {
        bounding_box(&self.orientations[orientation])
    }
}

/// Catalogue complet des pièces + dimensions de grille, tel que défini dans
/// `data/pieces.json`.
#[derive(Debug, Clone, Serialize)]
pub struct PieceCatalog {
    pub grid_width: i32,
    pub grid_height: i32,
    pub white_symmetry_forbidden: bool,
    pub pieces: Vec<PieceDef>,
}

impl PieceCatalog {
    pub fn piece(&self, id: &str) -> Option<&PieceDef> {
        self.pieces.iter().find(|p| p.id == id)
    }

    /// Les 5 pièces de la version de base : rouge, jaune, bleu, blanc x2.
    pub fn base_ids() -> [&'static str; 5] {
        ["red", "yellow", "blue", "white1", "white2"]
    }

    /// Charge le catalogue depuis une chaîne JSON (voir `data/pieces.json`).
    pub fn from_json(json: &str) -> Result<PieceCatalog, String> {
        let raw: RawCatalog = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let mut pieces = Vec::new();
        for rp in raw.pieces {
            let seed: Vec<Cell> = rp
                .cells
                .iter()
                .map(|rc| {
                    let kind = CellKind::parse(&rc.kind)
                        .ok_or_else(|| format!("kind inconnu: {}", rc.kind))?;
                    Ok(Cell::new(rc.x, rc.y, kind))
                })
                .collect::<Result<_, String>>()?;
            let orientations = generate_orientations(&seed);
            pieces.push(PieceDef {
                id: rp.id,
                label_fr: rp.label_fr,
                color: rp.color,
                expansion: rp.expansion,
                special: rp.special,
                orientations,
            });
        }
        Ok(PieceCatalog {
            grid_width: raw.grid.width,
            grid_height: raw.grid.height,
            white_symmetry_forbidden: raw.rules.white_symmetry_forbidden,
            pieces,
        })
    }

    /// Le catalogue par défaut, embarqué dans le binaire depuis
    /// `data/pieces.json` au moment de la compilation.
    pub fn default_catalog() -> PieceCatalog {
        PieceCatalog::from_json(DEFAULT_PIECES_JSON)
            .expect("data/pieces.json doit être un catalogue valide")
    }
}

/// Contenu figé de `data/pieces.json` au moment de la compilation.
pub const DEFAULT_PIECES_JSON: &str = include_str!("../../../data/pieces.json");

/// Génère les (au plus 8) orientations du groupe diédral D4 à partir d'une
/// forme de base, puis déduplique les formes géométriquement identiques
/// (ex: le losange n'a qu'1 orientation, un triangle rectangle isocèle en
/// a 4, un parallélogramme quelconque en a 4).
fn generate_orientations(seed: &[Cell]) -> Vec<Vec<Cell>> {
    let mut result: Vec<Vec<Cell>> = Vec::new();
    let mut seen: HashSet<Vec<Cell>> = HashSet::new();

    let push = |cells: Vec<Cell>, result: &mut Vec<Vec<Cell>>, seen: &mut HashSet<Vec<Cell>>| {
        let canon = canonical(&cells);
        if seen.insert(canon) {
            result.push(cells);
        }
    };

    let mut rot = seed.to_vec();
    for _ in 0..4 {
        push(rot.clone(), &mut result, &mut seen);
        rot = rotate_cw(&rot);
    }

    let mirrored = mirror_horizontal(seed);
    let mut rot = mirrored;
    for _ in 0..4 {
        push(rot.clone(), &mut result, &mut seen);
        rot = rotate_cw(&rot);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_loads_from_default_json() {
        let cat = PieceCatalog::default_catalog();
        assert_eq!(cat.grid_width, 10);
        assert_eq!(cat.grid_height, 8);
        assert_eq!(cat.pieces.len(), 7);
    }

    #[test]
    fn base_pieces_have_expected_area() {
        let cat = PieceCatalog::default_catalog();
        assert_eq!(cat.piece("red").unwrap().area(), 2.0);
        assert_eq!(cat.piece("yellow").unwrap().area(), 2.0);
        assert_eq!(cat.piece("blue").unwrap().area(), 4.0);
        assert_eq!(cat.piece("white1").unwrap().area(), 4.0);
        assert_eq!(cat.piece("white2").unwrap().area(), 2.0);
        assert_eq!(cat.piece("diamond").unwrap().area(), 2.0);
        assert_eq!(cat.piece("black").unwrap().area(), 2.0);
    }

    #[test]
    fn orientation_counts_match_expected_symmetry() {
        let cat = PieceCatalog::default_catalog();
        // Parallélogramme quelconque (pas de symétrie) : 4 orientations distinctes.
        assert_eq!(cat.piece("red").unwrap().orientations.len(), 4);
        // Triangle rectangle isocèle : symétrique par miroir -> 4 orientations.
        assert_eq!(cat.piece("yellow").unwrap().orientations.len(), 4);
        assert_eq!(cat.piece("blue").unwrap().orientations.len(), 4);
        assert_eq!(cat.piece("white1").unwrap().orientations.len(), 4);
        // Losange : symétrie complète -> 1 seule orientation.
        assert_eq!(cat.piece("white2").unwrap().orientations.len(), 1);
        assert_eq!(cat.piece("diamond").unwrap().orientations.len(), 1);
        // Rectangle 1x2 : 2 orientations (horizontal / vertical).
        assert_eq!(cat.piece("black").unwrap().orientations.len(), 2);
    }

    #[test]
    fn all_orientations_have_same_area_as_seed() {
        let cat = PieceCatalog::default_catalog();
        for piece in &cat.pieces {
            let expected = piece.area();
            for (i, _) in piece.orientations.iter().enumerate() {
                let area: f64 = piece.orientations[i]
                    .iter()
                    .map(|c| match c.kind {
                        CellKind::Square => 1.0,
                        CellKind::Triangle(_) => 0.5,
                    })
                    .sum();
                assert_eq!(area, expected, "pièce {} orientation {}", piece.id, i);
            }
        }
    }
}
