//! Modèle géométrique de base : cases, demi-cases triangulaires, coins,
//! directions, et les transformations (rotation 90°, miroir) utilisées pour
//! dériver les orientations autorisées d'une pièce à partir de sa forme de
//! base (voir `pieces.rs`).
//!
//! Repère : x croît vers la droite, y croît vers le bas. Une case pleine
//! occupe le carré unité `[x, x+1] × [y, y+1]`. Un triangle occupe la moitié
//! de cette case adjacente à l'un des 4 coins (voir `Corner`).

use serde::{Deserialize, Serialize};
use std::fmt;

/// Un des 4 coins d'une case, désignant quelle moitié de la case un
/// triangle occupe (la moitié adjacente à ce coin).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Corner {
    Nw,
    Ne,
    Se,
    Sw,
}

impl Corner {
    /// Rotation 90° horaire du coin (utilisée pour dériver les orientations
    /// et pour faire tourner une pièce visuellement).
    pub fn rotate_cw(self) -> Corner {
        match self {
            Corner::Nw => Corner::Ne,
            Corner::Ne => Corner::Se,
            Corner::Se => Corner::Sw,
            Corner::Sw => Corner::Nw,
        }
    }

    /// Symétrie miroir horizontale (axe vertical) : W <-> E.
    pub fn mirror_horizontal(self) -> Corner {
        match self {
            Corner::Nw => Corner::Ne,
            Corner::Ne => Corner::Nw,
            Corner::Se => Corner::Sw,
            Corner::Sw => Corner::Se,
        }
    }

    /// La diagonale de la case coupée par ce triangle : `NwSe` (coins NW/SE)
    /// ou `NeSw` (coins NE/SW). C'est cette diagonale qui détermine l'effet
    /// miroir du faisceau (voir `beam.rs`).
    pub fn hypotenuse_diagonal(self) -> Diagonal {
        match self {
            Corner::Nw | Corner::Se => Diagonal::NeSw,
            Corner::Ne | Corner::Sw => Diagonal::NwSe,
        }
    }

    /// Les deux côtés de case (orthogonaux) qui forment les cathètes du
    /// triangle, exprimés comme les `Side` adjacents à ce coin.
    pub fn legs(self) -> [Side; 2] {
        match self {
            Corner::Nw => [Side::North, Side::West],
            Corner::Ne => [Side::North, Side::East],
            Corner::Se => [Side::South, Side::East],
            Corner::Sw => [Side::South, Side::West],
        }
    }
}

/// La diagonale (hypoténuse) d'une case coupée en triangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Diagonal {
    /// Coins NW-SE (visuellement `\` en repère y-vers-le-bas).
    NwSe,
    /// Coins NE-SW (visuellement `/` en repère y-vers-le-bas).
    NeSw,
}

/// Un des 4 côtés (orthogonaux) d'une case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    North,
    South,
    East,
    West,
}

/// Contenu d'une case du plateau.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CellKind {
    Square,
    Triangle(Corner),
}

// Sérialisation manuelle (plutôt que dérivée) pour produire les mêmes
// chaînes plates ("square", "tri_nw", ...) que `as_str`/`parse`, utilisées
// à la fois par `data/pieces.json` et par le JSON envoyé au client (voir
// `orapa-wasm`) — une seule convention de nommage des cases côté JSON.
impl Serialize for CellKind {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for CellKind {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        CellKind::parse(&s).ok_or_else(|| serde::de::Error::custom(format!("kind inconnu: {s}")))
    }
}

impl CellKind {
    pub fn rotate_cw(self) -> CellKind {
        match self {
            CellKind::Square => CellKind::Square,
            CellKind::Triangle(c) => CellKind::Triangle(c.rotate_cw()),
        }
    }

    pub fn mirror_horizontal(self) -> CellKind {
        match self {
            CellKind::Square => CellKind::Square,
            CellKind::Triangle(c) => CellKind::Triangle(c.mirror_horizontal()),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            CellKind::Square => "square",
            CellKind::Triangle(Corner::Nw) => "tri_nw",
            CellKind::Triangle(Corner::Ne) => "tri_ne",
            CellKind::Triangle(Corner::Se) => "tri_se",
            CellKind::Triangle(Corner::Sw) => "tri_sw",
        }
    }

    pub fn parse(s: &str) -> Option<CellKind> {
        Some(match s {
            "square" => CellKind::Square,
            "tri_nw" => CellKind::Triangle(Corner::Nw),
            "tri_ne" => CellKind::Triangle(Corner::Ne),
            "tri_se" => CellKind::Triangle(Corner::Se),
            "tri_sw" => CellKind::Triangle(Corner::Sw),
            _ => return None,
        })
    }
}

impl fmt::Display for CellKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Une case (position relative dans la forme d'une pièce, ou absolue sur le
/// plateau) et son contenu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
    pub kind: CellKind,
}

impl Cell {
    pub fn new(x: i32, y: i32, kind: CellKind) -> Self {
        Cell { x, y, kind }
    }
}

/// Direction cardinale de déplacement du faisceau, en pas de case (dx, dy).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Direction {
    pub dx: i32,
    pub dy: i32,
}

impl Direction {
    pub const UP: Direction = Direction { dx: 0, dy: -1 };
    pub const DOWN: Direction = Direction { dx: 0, dy: 1 };
    pub const LEFT: Direction = Direction { dx: -1, dy: 0 };
    pub const RIGHT: Direction = Direction { dx: 1, dy: 0 };

    pub fn reverse(self) -> Direction {
        Direction {
            dx: -self.dx,
            dy: -self.dy,
        }
    }

    /// Réflexion à 90° par un miroir orienté selon `diag`.
    pub fn reflect(self, diag: Diagonal) -> Direction {
        match diag {
            // Hypoténuse NW-SE : (dx,dy) -> (dy,dx)
            Diagonal::NwSe => Direction {
                dx: self.dy,
                dy: self.dx,
            },
            // Hypoténuse NE-SW : (dx,dy) -> (-dy,-dx)
            Diagonal::NeSw => Direction {
                dx: -self.dy,
                dy: -self.dx,
            },
        }
    }

    /// Le côté de la case par lequel on entre quand on avance dans cette
    /// direction (ex: se déplacer vers la droite entre par le côté Ouest).
    pub fn entry_side(self) -> Side {
        match (self.dx, self.dy) {
            (1, 0) => Side::West,
            (-1, 0) => Side::East,
            (0, 1) => Side::North,
            (0, -1) => Side::South,
            _ => unreachable!("direction non cardinale"),
        }
    }
}

/// Boîte englobante (largeur, hauteur) d'un ensemble de cases normalisé
/// (min x = 0, min y = 0).
pub fn bounding_box(cells: &[Cell]) -> (i32, i32) {
    let w = cells.iter().map(|c| c.x).max().unwrap_or(-1) + 1;
    let h = cells.iter().map(|c| c.y).max().unwrap_or(-1) + 1;
    (w, h)
}

/// Rotation 90° horaire d'une forme normalisée (min x=0, min y=0).
/// Le nouveau plan a pour dimensions (hauteur, largeur) de l'original.
pub fn rotate_cw(cells: &[Cell]) -> Vec<Cell> {
    let (_, h) = bounding_box(cells);
    cells
        .iter()
        .map(|c| Cell::new(h - 1 - c.y, c.x, c.kind.rotate_cw()))
        .collect()
}

/// Symétrie miroir horizontale (axe vertical) d'une forme normalisée.
pub fn mirror_horizontal(cells: &[Cell]) -> Vec<Cell> {
    let (w, _) = bounding_box(cells);
    cells
        .iter()
        .map(|c| Cell::new(w - 1 - c.x, c.y, c.kind.mirror_horizontal()))
        .collect()
}

/// Forme canonique (triée) utilisée pour dédupliquer des orientations
/// géométriquement identiques.
pub fn canonical(cells: &[Cell]) -> Vec<Cell> {
    let mut v = cells.to_vec();
    v.sort();
    v
}
