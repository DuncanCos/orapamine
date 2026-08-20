//! Tracé du faisceau ("onde") depuis un point du bord de la grille.
//!
//! Modèle "centre-de-case" : le faisceau se propage de centre de case en
//! centre de case, entrant/sortant par le milieu des côtés. Il ne passe
//! donc jamais exactement par un sommet de la grille — le cas "faisceau qui
//! frappe un coin exactement" ne peut pas se produire avec ce modèle,
//! comme documenté dans le plan.
//!
//! Comportement à une case occupée :
//! - face orthogonale (carré, ou cathète d'un triangle) -> demi-tour ;
//! - hypoténuse d'un triangle -> déviation à 90° (miroir).
//!
//! Cas limites gérés explicitement (voir README pour la version longue) :
//! - Corps noir -> absorption (`BeamOutcome::Absorbed`), pas de sortie ni couleur ;
//! - boucle infinie (rebond en cycle fermé) -> détectée par répétition d'un
//!   état `(case, direction)` -> `BeamOutcome::Lost`, avec un filet de
//!   sécurité supplémentaire (nombre de pas maximum) au cas où la détection
//!   de cycle serait un jour contournée par une évolution du modèle.

use crate::board::Board;
use crate::colors::{mix_colors, ResultColor};
use crate::geometry::Direction;
use crate::pieces::Special;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};

/// Un des 36 points de tir sur le pourtour de la grille.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryPoint {
    pub id: String,
    pub cell: (i32, i32),
    /// Direction dans laquelle le faisceau se déplace en entrant dans la
    /// grille à ce point.
    pub dir: Direction,
}

/// Génère les 36 points de tir : lettres A–R à gauche (A–H, de haut en bas)
/// puis en bas (I–R, de gauche à droite) ; chiffres 1–18 en haut (1–10, de
/// gauche à droite) puis à droite (11–18, de haut en bas). Reproduit la
/// disposition décrite dans le cahier des charges (cf. adaptation Board
/// Game Arena).
pub fn entry_points(width: i32, height: i32) -> Vec<EntryPoint> {
    let letters = "ABCDEFGHIJKLMNOPQR";
    let mut points = Vec::with_capacity(36);
    let mut letter_iter = letters.chars();

    // Gauche : A..(A+height-1), du haut vers le bas, entre en allant à droite.
    for y in 0..height {
        let id = letter_iter.next().unwrap().to_string();
        points.push(EntryPoint {
            id,
            cell: (0, y),
            dir: Direction::RIGHT,
        });
    }
    // Bas : lettres suivantes, de gauche à droite, entre en allant vers le haut.
    for x in 0..width {
        let id = letter_iter.next().unwrap().to_string();
        points.push(EntryPoint {
            id,
            cell: (x, height - 1),
            dir: Direction::UP,
        });
    }
    // Haut : 1..width, de gauche à droite, entre en allant vers le bas.
    for x in 0..width {
        let id = (x + 1).to_string();
        points.push(EntryPoint {
            id,
            cell: (x, 0),
            dir: Direction::DOWN,
        });
    }
    // Droite : width+1..width+height, du haut vers le bas, entre en allant à gauche.
    for y in 0..height {
        let id = (width + y + 1).to_string();
        points.push(EntryPoint {
            id,
            cell: (width - 1, y),
            dir: Direction::LEFT,
        });
    }
    points
}

/// Index des points de tir, pour la recherche par id (entrée) et par
/// (case, direction de sortie) (sortie).
type CellCoord = (i32, i32);
type DirVec = (i32, i32);

pub struct PointIndex {
    by_id: HashMap<String, EntryPoint>,
    by_exit: HashMap<(CellCoord, DirVec), String>,
}

impl PointIndex {
    pub fn build(width: i32, height: i32) -> PointIndex {
        let points = entry_points(width, height);
        let mut by_id = HashMap::new();
        let mut by_exit = HashMap::new();
        for p in points {
            // Un faisceau qui sort par ce point voyage, au moment de
            // franchir le bord, dans la direction opposée à `p.dir`.
            let exit_dir = p.dir.reverse();
            by_exit.insert((p.cell, (exit_dir.dx, exit_dir.dy)), p.id.clone());
            by_id.insert(p.id.clone(), p);
        }
        PointIndex { by_id, by_exit }
    }

    pub fn get(&self, id: &str) -> Option<&EntryPoint> {
        self.by_id.get(id)
    }

    fn exit_id(&self, cell: (i32, i32), dir: Direction) -> Option<&String> {
        self.by_exit.get(&(cell, (dir.dx, dir.dy)))
    }
}

/// Résultat de l'envoi d'une onde.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum BeamOutcome {
    /// Le faisceau ressort par `point` (peut être le point d'entrée) avec
    /// la couleur finale `color`.
    Exit { point: String, color: ResultColor },
    /// Le faisceau a été absorbé par un corps noir.
    Absorbed,
    /// Le faisceau est resté piégé dans une boucle fermée entre plusieurs
    /// pièces et ne ressortira jamais (cas limite, protégé contre les
    /// boucles infinies).
    Lost,
}

/// Trace le faisceau entrant au point `entry_id` et renvoie son résultat.
/// Retourne `None` si `entry_id` n'est pas un point de tir valide.
pub fn fire_beam(board: &Board, points: &PointIndex, entry_id: &str) -> Option<BeamOutcome> {
    fire_beam_traced(board, points, entry_id).map(|(outcome, _)| outcome)
}

/// Comme `fire_beam`, mais renvoie en plus l'ensemble des index de pièces
/// (`placement_index`, voir `board.rs`) effectivement touchées en cours de
/// trajet — utilisé par `placement.rs` pour la contrainte d'atteignabilité
/// (§2.3.4 : chaque pièce doit être touchée par au moins une onde).
pub fn fire_beam_traced(
    board: &Board,
    points: &PointIndex,
    entry_id: &str,
) -> Option<(BeamOutcome, HashSet<usize>)> {
    let start = points.get(entry_id)?;
    Some(simulate(board, points, start.cell, start.dir))
}

/// Cœur du tracé, factorisé pour être testable directement avec un état de
/// départ arbitraire (pas seulement un point de tir réel) — utilisé par les
/// tests unitaires ci-dessous pour les cas limites (rebond, déviation,
/// absorption, boucle).
pub(crate) fn simulate(
    board: &Board,
    points: &PointIndex,
    start_cell: (i32, i32),
    start_dir: Direction,
) -> (BeamOutcome, HashSet<usize>) {
    let mut cell = start_cell;
    let mut dir = start_dir;
    let mut colors: BTreeSet<crate::pieces::GemColor> = BTreeSet::new();
    let mut hit_placements: HashSet<usize> = HashSet::new();
    let mut visited: HashSet<((i32, i32), (i32, i32))> = HashSet::new();
    let max_steps = (4 * board.width * board.height + 1) as usize;

    for _ in 0..max_steps {
        if !visited.insert((cell, (dir.dx, dir.dy))) {
            return (BeamOutcome::Lost, hit_placements);
        }

        let advance_dir = match board.get(cell.0, cell.1) {
            None => dir,
            Some(occ) => {
                hit_placements.insert(occ.placement_index);
                if occ.special == Some(Special::Absorb) {
                    return (BeamOutcome::Absorbed, hit_placements);
                }
                if let Some(c) = occ.color {
                    colors.insert(c);
                }
                match occ.kind {
                    crate::geometry::CellKind::Square => dir.reverse(),
                    crate::geometry::CellKind::Triangle(corner) => {
                        let entry_side = dir.entry_side();
                        if corner.legs().contains(&entry_side) {
                            dir.reverse()
                        } else {
                            dir.reflect(corner.hypotenuse_diagonal())
                        }
                    }
                }
            }
        };

        let next = (cell.0 + advance_dir.dx, cell.1 + advance_dir.dy);
        if !board.in_bounds(next.0, next.1) {
            let exit_id = points
                .exit_id(cell, advance_dir)
                .expect("tout franchissement de bord correspond à un point de tir");
            return (
                BeamOutcome::Exit {
                    point: exit_id.clone(),
                    color: mix_colors(&colors),
                },
                hit_placements,
            );
        }
        cell = next;
        dir = advance_dir;
    }
    (BeamOutcome::Lost, hit_placements)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::BoardCell;
    use crate::geometry::{CellKind, Corner};
    use crate::pieces::{GemColor, PieceCatalog};

    fn one_cell_board(width: i32, height: i32, x: i32, y: i32, cell: BoardCell) -> Board {
        let mut board = Board::empty(width, height);
        board.set(x, y, cell);
        board
    }

    fn red(kind: CellKind) -> BoardCell {
        BoardCell {
            placement_index: 0,
            kind,
            color: Some(GemColor::Red),
            special: None,
        }
    }

    /// Une face orthogonale (carré) renvoie le faisceau vers son point
    /// d'entrée : demi-tour, pas de déviation.
    #[test]
    fn square_bounces_straight_back() {
        let board = one_cell_board(4, 4, 1, 1, red(CellKind::Square));
        let points = PointIndex::build(4, 4);
        // Point "2" : haut, colonne x=1, entre en descendant.
        let outcome = fire_beam(&board, &points, "2").unwrap();
        assert_eq!(
            outcome,
            BeamOutcome::Exit {
                point: "2".to_string(),
                color: ResultColor::Red,
            }
        );
    }

    /// Un triangle touché sur l'une de ses deux cathètes (face orthogonale)
    /// se comporte comme un carré : demi-tour.
    #[test]
    fn triangle_leg_hit_bounces_back() {
        // Nw : cathètes Nord et Ouest. On tire du haut -> on touche la
        // cathète Nord -> demi-tour, ressort par le même point.
        let board = one_cell_board(4, 4, 1, 1, red(CellKind::Triangle(Corner::Nw)));
        let points = PointIndex::build(4, 4);
        let outcome = fire_beam(&board, &points, "2").unwrap();
        assert_eq!(
            outcome,
            BeamOutcome::Exit {
                point: "2".to_string(),
                color: ResultColor::Red,
            }
        );
    }

    /// Un triangle touché sur son hypoténuse dévie le faisceau à 90°,
    /// comme un miroir.
    #[test]
    fn triangle_hypotenuse_hit_deflects_90_degrees() {
        // Nw : hypoténuse tournée vers Sud/Est. On tire depuis la droite
        // (le faisceau entre par le côté Est de la case) -> déviation.
        let board = one_cell_board(4, 4, 1, 1, red(CellKind::Triangle(Corner::Nw)));
        let points = PointIndex::build(4, 4);
        // Point à droite, ligne y=1 : "6" (droite = width+y+1 = 4+1+1=6),
        // entre en allant vers la gauche, traverse la case vide (2,1) puis
        // touche (1,1) par l'Est -> dévie vers le Bas -> ressort en bas,
        // colonne x=1 -> lettre "F" (E..H, F=x=1).
        let outcome = fire_beam(&board, &points, "6").unwrap();
        assert_eq!(
            outcome,
            BeamOutcome::Exit {
                point: "F".to_string(),
                color: ResultColor::Red,
            }
        );
    }

    /// Le corps noir absorbe le faisceau : ni point de sortie, ni couleur.
    #[test]
    fn black_gem_absorbs_the_beam() {
        let cell = BoardCell {
            placement_index: 0,
            kind: CellKind::Square,
            color: None,
            special: Some(Special::Absorb),
        };
        let board = one_cell_board(4, 4, 1, 1, cell);
        let points = PointIndex::build(4, 4);
        let outcome = fire_beam(&board, &points, "2").unwrap();
        assert_eq!(outcome, BeamOutcome::Absorbed);
    }

    /// Le diamant (transparent) dévie comme un triangle normal mais ne
    /// contribue aucune couleur au mélange.
    #[test]
    fn diamond_deflects_without_adding_color() {
        let cell = BoardCell {
            placement_index: 0,
            kind: CellKind::Triangle(Corner::Nw),
            color: None,
            special: Some(Special::Transparent),
        };
        let board = one_cell_board(4, 4, 1, 1, cell);
        let points = PointIndex::build(4, 4);
        let outcome = fire_beam(&board, &points, "6").unwrap();
        assert_eq!(
            outcome,
            BeamOutcome::Exit {
                point: "F".to_string(),
                color: ResultColor::Transparent,
            }
        );
    }

    /// Une boucle fermée entre plusieurs pièces est détectée (répétition
    /// d'un état case+direction) plutôt que de tourner indéfiniment.
    /// Configuration synthétique : 4 triangles en "moulin à vent" dont les
    /// hypoténuses font toutes face au centre du bloc 2×2 ; un faisceau
    /// injecté au milieu de ce bloc y circule indéfiniment sans jamais
    /// atteindre un bord. (Cette configuration n'est pas nécessairement
    /// atteignable par un placement légal de pièces réelles — elle sert à
    /// valider le garde-fou anti-boucle lui-même.)
    #[test]
    fn closed_loop_between_pieces_is_detected_as_lost() {
        let mut board = Board::empty(6, 6);
        board.set(1, 1, red(CellKind::Triangle(Corner::Nw)));
        board.set(2, 1, red(CellKind::Triangle(Corner::Ne)));
        board.set(1, 2, red(CellKind::Triangle(Corner::Sw)));
        board.set(2, 2, red(CellKind::Triangle(Corner::Se)));
        let points = PointIndex::build(6, 6);
        let (outcome, _) = simulate(&board, &points, (1, 1), Direction::UP);
        assert_eq!(outcome, BeamOutcome::Lost);
    }

    /// L'entrée et la sortie d'un point de tir sont réciproques : tirer
    /// depuis le point de sortie annoncé doit reproduire le trajet inverse
    /// et ressortir exactement au point d'entrée d'origine (propriété de
    /// réversibilité optique).
    #[test]
    fn beam_paths_are_reversible() {
        use rand::SeedableRng;
        let catalog = PieceCatalog::default_catalog();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(99);
        let placements =
            crate::generator::random_valid_placement(&mut rng, &catalog, crate::generator::GameOptions::default());
        let board = Board::build(
            &catalog,
            catalog.grid_width,
            catalog.grid_height,
            &placements,
        )
        .unwrap();
        let points = PointIndex::build(catalog.grid_width, catalog.grid_height);
        for p in entry_points(catalog.grid_width, catalog.grid_height) {
            if let Some(BeamOutcome::Exit { point: exit_id, .. }) =
                fire_beam(&board, &points, &p.id)
            {
                let back = fire_beam(&board, &points, &exit_id).unwrap();
                assert_eq!(
                    back,
                    BeamOutcome::Exit {
                        point: p.id.clone(),
                        color: match fire_beam(&board, &points, &p.id).unwrap() {
                            BeamOutcome::Exit { color, .. } => color,
                            _ => unreachable!(),
                        },
                    },
                    "le trajet depuis {} devrait revenir vers {}",
                    exit_id,
                    p.id
                );
            }
        }
    }
}
