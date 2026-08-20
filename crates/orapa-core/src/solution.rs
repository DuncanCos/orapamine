//! Comparaison d'une solution proposée à la disposition secrète réelle.
//!
//! La comparaison se fait sur le **rendu géométrique** du plateau (quelle
//! case contient quel type de contenu, de quelle couleur), pas sur les
//! identifiants ou orientations de pièces choisis pour y arriver : deux
//! placements différents qui produisent exactement le même plateau comptent
//! comme identiques.

use crate::board::{Board, PlacedPiece};
use crate::pieces::PieceCatalog;

/// Compare une solution proposée `guess` à la disposition secrète `secret`.
/// `Ok(true)` si les deux plateaux sont géométriquement identiques case par
/// case (contenu + couleur), `Ok(false)` sinon (y compris si `guess` est
/// lui-même mal formé : chevauchement, hors grille...). `Err` uniquement si
/// `secret` — censé avoir déjà été validé au moment du placement — s'avère
/// invalide (signe d'un bug appelant).
pub fn compare_solution(
    catalog: &PieceCatalog,
    width: i32,
    height: i32,
    secret: &[PlacedPiece],
    guess: &[PlacedPiece],
) -> Result<bool, String> {
    let secret_board = Board::build(catalog, width, height, secret)
        .map_err(|e| format!("disposition secrète invalide (bug appelant): {e:?}"))?;
    let guess_board = match Board::build(catalog, width, height, guess) {
        Ok(b) => b,
        Err(_) => return Ok(false),
    };
    Ok(boards_equal(&secret_board, &guess_board))
}

fn boards_equal(a: &Board, b: &Board) -> bool {
    if a.width != b.width || a.height != b.height {
        return false;
    }
    for y in 0..a.height {
        for x in 0..a.width {
            let ca = a.get(x, y).map(|c| (c.kind, c.color, c.special));
            let cb = b.get(x, y).map(|c| (c.kind, c.color, c.special));
            if ca != cb {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::{random_valid_placement, GameOptions};
    use rand::SeedableRng;

    #[test]
    fn identical_placement_matches() {
        let catalog = PieceCatalog::default_catalog();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
        let placements = random_valid_placement(&mut rng, &catalog, GameOptions::default());
        assert!(compare_solution(
            &catalog,
            catalog.grid_width,
            catalog.grid_height,
            &placements,
            &placements
        )
        .unwrap());
    }

    #[test]
    fn different_placement_does_not_match() {
        let catalog = PieceCatalog::default_catalog();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(2);
        let secret = random_valid_placement(&mut rng, &catalog, GameOptions::default());
        let guess = random_valid_placement(&mut rng, &catalog, GameOptions::default());
        // Avec une graine différente il est extrêmement improbable (mais
        // pas mathématiquement impossible) que les deux tirages coïncident ;
        // on vérifie donc surtout que la fonction ne dit pas toujours vrai.
        let result = compare_solution(
            &catalog,
            catalog.grid_width,
            catalog.grid_height,
            &secret,
            &guess,
        )
        .unwrap();
        assert!(!result || secret == guess);
    }

    #[test]
    fn overlapping_guess_never_matches() {
        let catalog = PieceCatalog::default_catalog();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(3);
        let secret = random_valid_placement(&mut rng, &catalog, GameOptions::default());
        let mut bad_guess = secret.clone();
        // Force un chevauchement en dupliquant l'ancre de la première pièce
        // sur la deuxième.
        if bad_guess.len() >= 2 {
            bad_guess[1].anchor_x = bad_guess[0].anchor_x;
            bad_guess[1].anchor_y = bad_guess[0].anchor_y;
            bad_guess[1].orientation = 0;
        }
        let result = compare_solution(
            &catalog,
            catalog.grid_width,
            catalog.grid_height,
            &secret,
            &bad_guess,
        )
        .unwrap();
        assert!(!result);
    }
}
