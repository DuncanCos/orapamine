//! Validation des contraintes de placement (§2.3 du cahier des charges).
//! Retourne la liste de toutes les violations détectées (pas seulement la
//! première), pour permettre à l'interface de les afficher en temps réel.

use crate::beam::{fire_beam_traced, entry_points, PointIndex};
use crate::board::{Board, BoardCell, PlacedPiece};
use crate::geometry::{CellKind, Side};
use crate::pieces::PieceCatalog;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Une violation détectée dans un placement. Les index font référence à la
/// position dans la tranche `placements` passée à `validate_placement`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Violation {
    /// Une case de la pièce sort de la grille.
    OutOfBounds { placement_index: usize },
    /// Deux pièces occupent (au moins en partie) la même case.
    Overlap {
        placement_index: usize,
        other_index: usize,
    },
    /// Deux pièces différentes ont des arêtes qui se touchent directement.
    EdgeContact {
        placement_index: usize,
        other_index: usize,
    },
    /// Aucune onde tirée depuis le pourtour ne touche cette pièce.
    Unreachable { placement_index: usize },
    /// Les deux gemmes blanches sont disposées de façon parfaitement
    /// symétrique (miroir vertical ou horizontal du plateau).
    WhiteSymmetry,
}

fn covers_side(kind: CellKind, side: Side) -> bool {
    match kind {
        CellKind::Square => true,
        CellKind::Triangle(corner) => corner.legs().contains(&side),
    }
}

/// Valide un placement complet (typiquement les 5 pièces de base, plus
/// éventuellement diamant/corps noir) contre les 6 contraintes du §2.3.
/// `Err` signale une entrée malformée (id de pièce ou orientation inconnus
/// — un bug appelant, pas une violation de règle) ; `Ok` contient la liste
/// (éventuellement vide) des violations trouvées.
pub fn validate_placement(
    catalog: &PieceCatalog,
    width: i32,
    height: i32,
    placements: &[PlacedPiece],
) -> Result<Vec<Violation>, String> {
    // Résout chaque placement vers sa forme (liste de cases relatives).
    let mut shapes = Vec::with_capacity(placements.len());
    for p in placements {
        let piece = catalog
            .piece(&p.piece_id)
            .ok_or_else(|| format!("pièce inconnue: {}", p.piece_id))?;
        let shape = piece
            .orientations
            .get(p.orientation)
            .ok_or_else(|| format!("orientation invalide pour {}: {}", p.piece_id, p.orientation))?;
        shapes.push(shape);
    }

    let mut violations = Vec::new();

    // 1&2. Bornes de la grille (les sommets sur la grille sont garantis par
    // construction du modèle en cases/demi-cases) + occupation par case.
    let mut occupancy: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    let mut out_of_bounds: HashSet<usize> = HashSet::new();
    // Première pièce déposée par case, pour la contrainte de contact (une
    // case en conflit n'entre pas dans ce calcul, il est de toute façon déjà
    // signalé comme un chevauchement).
    let mut cellmap: HashMap<(i32, i32), (usize, CellKind)> = HashMap::new();

    for (i, (placement, shape)) in placements.iter().zip(shapes.iter()).enumerate() {
        for c in shape.iter() {
            let x = placement.anchor_x + c.x;
            let y = placement.anchor_y + c.y;
            if x < 0 || y < 0 || x >= width || y >= height {
                out_of_bounds.insert(i);
                continue;
            }
            occupancy.entry((x, y)).or_default().push(i);
            cellmap.entry((x, y)).or_insert((i, c.kind));
        }
    }

    let mut oob_sorted: Vec<usize> = out_of_bounds.into_iter().collect();
    oob_sorted.sort_unstable();
    for i in oob_sorted {
        violations.push(Violation::OutOfBounds { placement_index: i });
    }

    // 3. Chevauchements.
    let mut overlap_pairs: HashSet<(usize, usize)> = HashSet::new();
    for idxs in occupancy.values() {
        if idxs.len() > 1 {
            for a in 0..idxs.len() {
                for b in (a + 1)..idxs.len() {
                    if idxs[a] != idxs[b] {
                        overlap_pairs.insert((idxs[a].min(idxs[b]), idxs[a].max(idxs[b])));
                    }
                }
            }
        }
    }
    let mut overlap_sorted: Vec<(usize, usize)> = overlap_pairs.into_iter().collect();
    overlap_sorted.sort_unstable();
    for (a, b) in overlap_sorted {
        violations.push(Violation::Overlap {
            placement_index: a,
            other_index: b,
        });
    }

    // 4. Contact arête-à-arête entre pièces différentes : interdit
    // uniquement quand les deux côtés en vis-à-vis sont *entièrement*
    // couverts (un contact coin-à-coin ou coin-à-arête est autorisé).
    let mut edge_pairs: HashSet<(usize, usize)> = HashSet::new();
    for (&(x, y), &(i, kind)) in cellmap.iter() {
        for &(dx, dy, side_here, side_there) in &[
            (1, 0, Side::East, Side::West),
            (0, 1, Side::South, Side::North),
        ] {
            if let Some(&(j, kind2)) = cellmap.get(&(x + dx, y + dy)) {
                if i != j && covers_side(kind, side_here) && covers_side(kind2, side_there) {
                    edge_pairs.insert((i.min(j), i.max(j)));
                }
            }
        }
    }
    let mut edge_sorted: Vec<(usize, usize)> = edge_pairs.into_iter().collect();
    edge_sorted.sort_unstable();
    for (a, b) in edge_sorted {
        violations.push(Violation::EdgeContact {
            placement_index: a,
            other_index: b,
        });
    }

    // 5. Atteignabilité : chaque pièce doit être touchée par au moins une
    // des 36 ondes possibles. Calculée sur un plateau approximatif à partir
    // de `cellmap` (premier occupant par case) pour rester robuste même en
    // présence d'un chevauchement déjà signalé ci-dessus.
    let mut approx_board = Board::empty(width, height);
    for (&(x, y), &(i, kind)) in cellmap.iter() {
        let piece = catalog.piece(&placements[i].piece_id).unwrap();
        approx_board.set(
            x,
            y,
            BoardCell {
                placement_index: i,
                kind,
                color: piece.color,
                special: piece.special,
            },
        );
    }
    let points = PointIndex::build(width, height);
    let mut reached: HashSet<usize> = HashSet::new();
    for p in entry_points(width, height) {
        if let Some((_, hits)) = fire_beam_traced(&approx_board, &points, &p.id) {
            reached.extend(hits);
        }
    }
    let mut unreachable: Vec<usize> = (0..placements.len())
        .filter(|i| !reached.contains(i))
        .collect();
    unreachable.sort_unstable();
    for i in unreachable {
        violations.push(Violation::Unreachable { placement_index: i });
    }

    // 6. Symétrie des deux gemmes blanches (désactivable via pieces.json).
    if catalog.white_symmetry_forbidden {
        let white_idxs: Vec<usize> = placements
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                catalog
                    .piece(&p.piece_id)
                    .and_then(|pc| pc.color)
                    .map(|c| c == crate::pieces::GemColor::White)
                    .unwrap_or(false)
            })
            .map(|(i, _)| i)
            .collect();
        if white_idxs.len() == 2 {
            let white_cells: HashSet<(i32, i32)> = white_idxs
                .iter()
                .flat_map(|&i| {
                    shapes[i]
                        .iter()
                        .map(move |c| (placements[i].anchor_x + c.x, placements[i].anchor_y + c.y))
                })
                .collect();
            if is_symmetric(&white_cells, width, height) {
                violations.push(Violation::WhiteSymmetry);
            }
        }
    }

    Ok(violations)
}

/// L'ensemble de cases est-il invariant par symétrie miroir verticale ou
/// horizontale du plateau ? Factorisé hors de `validate_placement` pour
/// être testable directement, indépendamment de la couleur "blanc" (avec
/// le catalogue par défaut, les deux gemmes blanches ont des formes
/// différentes et ne peuvent en pratique jamais déclencher cette
/// contrainte — voir les tests ci-dessous pour un cas synthétique où deux
/// formes identiques la déclenchent bien).
fn is_symmetric(cells: &HashSet<(i32, i32)>, width: i32, height: i32) -> bool {
    let mirrored_v: HashSet<(i32, i32)> = cells.iter().map(|&(x, y)| (width - 1 - x, y)).collect();
    let mirrored_h: HashSet<(i32, i32)> = cells.iter().map(|&(x, y)| (x, height - 1 - y)).collect();
    mirrored_v == *cells || mirrored_h == *cells
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pieces::PieceCatalog;

    fn catalog() -> PieceCatalog {
        PieceCatalog::default_catalog()
    }

    fn placed(piece_id: &str, orientation: usize, anchor_x: i32, anchor_y: i32) -> PlacedPiece {
        PlacedPiece {
            piece_id: piece_id.to_string(),
            orientation,
            anchor_x,
            anchor_y,
        }
    }

    #[test]
    fn detects_out_of_bounds() {
        let cat = catalog();
        // Rouge (boîte 3x1) ancré à x=8 sur une grille large de 10 : la
        // dernière colonne (x=10) sort de la grille.
        let placements = vec![placed("red", 0, 8, 0)];
        let violations =
            validate_placement(&cat, cat.grid_width, cat.grid_height, &placements).unwrap();
        assert!(violations
            .iter()
            .any(|v| matches!(v, Violation::OutOfBounds { placement_index: 0 })));
    }

    #[test]
    fn detects_overlap() {
        let cat = catalog();
        let placements = vec![placed("yellow", 0, 2, 2), placed("red", 0, 2, 2)];
        let violations =
            validate_placement(&cat, cat.grid_width, cat.grid_height, &placements).unwrap();
        assert!(violations.iter().any(|v| matches!(
            v,
            Violation::Overlap {
                placement_index: 0,
                other_index: 1
            }
        )));
    }

    #[test]
    fn detects_edge_to_edge_contact_between_squares() {
        let cat = catalog();
        // Deux "corps noir" (2 carrés) posés l'un directement sous l'autre :
        // arête pleine partagée, interdit.
        let placements = vec![placed("black", 0, 2, 2), placed("black", 0, 2, 3)];
        let violations =
            validate_placement(&cat, cat.grid_width, cat.grid_height, &placements).unwrap();
        assert!(violations.iter().any(|v| matches!(
            v,
            Violation::EdgeContact {
                placement_index: 0,
                other_index: 1
            }
        )));
    }

    #[test]
    fn corner_to_corner_contact_is_allowed() {
        let cat = catalog();
        // Même test mais décalé en diagonale : les deux pièces ne se
        // touchent qu'en un coin -> autorisé, aucun EdgeContact.
        let placements = vec![placed("black", 0, 2, 2), placed("black", 0, 4, 3)];
        let violations =
            validate_placement(&cat, cat.grid_width, cat.grid_height, &placements).unwrap();
        assert!(!violations
            .iter()
            .any(|v| matches!(v, Violation::EdgeContact { .. })));
    }

    #[test]
    fn triangle_hypotenuse_side_may_touch_a_square_edge() {
        let cat = catalog();
        // Le côté "hypoténuse" d'un triangle ne couvre son côté que
        // ponctuellement : un carré peut le toucher sans violation.
        // yellow(0,0)=carré,(1,0)=tri_nw,(0,1)=tri_nw à l'ancre (2,2) ->
        // cellule (3,2) est tri_nw, son côté Sud n'est pas une cathète
        // (legs = Nord,Ouest) donc un carré en (3,3) ne crée pas de contact.
        let placements = vec![placed("yellow", 0, 2, 2), placed("black", 1, 3, 3)];
        let violations =
            validate_placement(&cat, cat.grid_width, cat.grid_height, &placements).unwrap();
        assert!(!violations
            .iter()
            .any(|v| matches!(v, Violation::EdgeContact { .. })));
    }

    #[test]
    fn detects_unreachable_piece_fully_enclosed_by_square_walls() {
        let cat = catalog();
        // Pièce cible : rouge (parallélogramme) en ligne y=3, x=4..6.
        let target = placed("red", 0, 4, 3);
        // Mur complet de "corps noir" (carrés) bloquant les 4 lignes
        // d'approche (gauche, droite, haut, bas) de chacune des 3 cases de
        // la cible — voir le commentaire de conception détaillé dans le
        // plan/README : un carré fait toujours demi-tour au premier
        // contact, donc un seul carré sur chaque ligne droite suffit à la
        // bloquer entièrement.
        let walls = vec![
            placed("black", 1, 3, 2), // (3,2),(3,3) : bloque l'approche par la gauche
            placed("black", 0, 4, 2), // (4,2),(5,2) : bloque le haut de x=4,5
            placed("black", 1, 6, 1), // (6,1),(6,2) : bloque le haut de x=6
            placed("black", 0, 4, 4), // (4,4),(5,4) : bloque le bas de x=4,5
            placed("black", 1, 6, 4), // (6,4),(6,5) : bloque le bas de x=6
            placed("black", 1, 7, 3), // (7,3),(7,4) : bloque l'approche par la droite
        ];
        let mut placements = vec![target];
        placements.extend(walls);
        let violations =
            validate_placement(&cat, cat.grid_width, cat.grid_height, &placements).unwrap();
        assert!(
            violations
                .iter()
                .any(|v| matches!(v, Violation::Unreachable { placement_index: 0 })),
            "violations obtenues : {violations:?}"
        );
    }

    #[test]
    fn is_symmetric_detects_vertical_and_horizontal_mirror() {
        let width = 10;
        let height = 8;
        // Une case unique au centre exact d'une largeur paire n'existe pas
        // (10 est pair, pas de colonne centrale) ; on construit donc deux
        // cases symétriques l'une de l'autre par rapport à l'axe vertical.
        let mut cells = HashSet::new();
        cells.insert((2, 2));
        cells.insert((width - 1 - 2, 2)); // (7,2), symétrique de (2,2)
        assert!(is_symmetric(&cells, width, height));

        let mut cells_h = HashSet::new();
        cells_h.insert((3, 1));
        cells_h.insert((3, height - 1 - 1)); // (3,6), symétrique horizontal
        assert!(is_symmetric(&cells_h, width, height));

        let mut asym = HashSet::new();
        asym.insert((2, 2));
        asym.insert((3, 5));
        assert!(!is_symmetric(&asym, width, height));
    }

    #[test]
    fn real_white_pieces_never_trigger_symmetry_violation() {
        // Cas limite documenté : avec le catalogue par défaut, les deux
        // blanches ont des formes différentes (grand triangle vs losange),
        // donc leur union ne peut jamais être symétrique — la contrainte
        // reste vraie mais ne se déclenche jamais en pratique tant que le
        // livret officiel ne précise pas des formes identiques.
        let cat = catalog();
        let placements = vec![placed("white1", 0, 0, 0), placed("white2", 0, 0, 0)];
        // On ignore ici les autres violations (chevauchement probable) :
        // seul le comportement de la contrainte de symétrie nous intéresse.
        let violations =
            validate_placement(&cat, cat.grid_width, cat.grid_height, &placements).unwrap();
        assert!(!violations.contains(&Violation::WhiteSymmetry));
    }
}
