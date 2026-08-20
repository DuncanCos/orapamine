//! Génération d'un placement aléatoire valide ("placement aléatoire",
//! bouton demandé en §2.3 / §2.6.2). Recherche par tirage-et-rejet avec
//! reprise complète si un tirage se coince — le plateau est suffisamment
//! grand par rapport aux 5-7 pièces pour que ça converge vite en pratique
//! (voir le test de propriété `many_random_placements_are_valid`).

use crate::board::PlacedPiece;
use crate::pieces::PieceCatalog;
use crate::placement::validate_placement;
use rand::Rng;

/// Quelles pièces d'extension inclure en plus des 5 pièces de base.
#[derive(Debug, Clone, Copy, Default)]
pub struct GameOptions {
    pub diamond: bool,
    pub black: bool,
}

impl GameOptions {
    pub fn piece_ids(&self) -> Vec<&'static str> {
        let mut ids: Vec<&'static str> = PieceCatalog::base_ids().to_vec();
        if self.diamond {
            ids.push("diamond");
        }
        if self.black {
            ids.push("black");
        }
        ids
    }
}

const MAX_ATTEMPTS: usize = 5000;
const MAX_LOCAL_TRIES_PER_PIECE: usize = 300;

/// Tire un placement aléatoire respectant les 6 contraintes du §2.3.
/// Panique si aucun placement valide n'a pu être trouvé en `MAX_ATTEMPTS`
/// essais (ne devrait jamais arriver sur une grille 10×8 avec 5 à 7 pièces).
pub fn random_valid_placement(
    rng: &mut impl Rng,
    catalog: &PieceCatalog,
    options: GameOptions,
) -> Vec<PlacedPiece> {
    let ids = options.piece_ids();
    let width = catalog.grid_width;
    let height = catalog.grid_height;

    for _attempt in 0..MAX_ATTEMPTS {
        if let Some(placements) = try_one_attempt(rng, catalog, &ids, width, height) {
            // Vérifie les contraintes globales (atteignabilité, symétrie
            // des blanches) qui ne peuvent être garanties pièce par pièce.
            if let Ok(violations) = validate_placement(catalog, width, height, &placements) {
                if violations.is_empty() {
                    return placements;
                }
            }
        }
    }
    panic!("random_valid_placement: aucun placement valide trouvé après {MAX_ATTEMPTS} essais");
}

fn try_one_attempt(
    rng: &mut impl Rng,
    catalog: &PieceCatalog,
    ids: &[&str],
    width: i32,
    height: i32,
) -> Option<Vec<PlacedPiece>> {
    let mut placed: Vec<PlacedPiece> = Vec::with_capacity(ids.len());
    for &id in ids {
        let piece = catalog.piece(id)?;
        let mut ok = false;
        for _ in 0..MAX_LOCAL_TRIES_PER_PIECE {
            let orientation = rng.gen_range(0..piece.orientations.len());
            let (bw, bh) = piece.bounding_box(orientation);
            if bw > width || bh > height {
                continue;
            }
            let anchor_x = rng.gen_range(0..=(width - bw));
            let anchor_y = rng.gen_range(0..=(height - bh));
            let candidate = PlacedPiece {
                piece_id: id.to_string(),
                orientation,
                anchor_x,
                anchor_y,
            };
            let mut trial = placed.clone();
            trial.push(candidate.clone());
            // Rejet rapide : bornes + chevauchement + contact d'arêtes
            // seulement (pas l'atteignabilité, qui nécessite le plateau
            // complet — vérifiée une fois toutes les pièces posées).
            if let Ok(violations) = validate_placement(catalog, width, height, &trial) {
                let blocking = violations.iter().any(|v| {
                    !matches!(v, crate::placement::Violation::Unreachable { .. })
                        && !matches!(v, crate::placement::Violation::WhiteSymmetry)
                });
                if !blocking {
                    placed.push(candidate);
                    ok = true;
                    break;
                }
            }
        }
        if !ok {
            return None;
        }
    }
    Some(placed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn many_random_placements_are_valid() {
        let catalog = PieceCatalog::default_catalog();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        for i in 0..200 {
            let options = GameOptions {
                diamond: i % 2 == 0,
                black: i % 3 == 0,
            };
            let placements = random_valid_placement(&mut rng, &catalog, options);
            let violations =
                validate_placement(&catalog, catalog.grid_width, catalog.grid_height, &placements)
                    .unwrap();
            assert!(
                violations.is_empty(),
                "placement invalide (tirage {i}): {violations:?}"
            );
        }
    }
}
