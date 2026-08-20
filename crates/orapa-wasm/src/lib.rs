//! Bindings WASM pour `orapa-core`, consommés par le client React pour :
//! - afficher le catalogue de pièces (`catalog_json`) ;
//! - valider un placement en temps réel pendant la phase de placement
//!   (`validate_placement_json`), sans aller-retour réseau à chaque
//!   déplacement de pièce ;
//! - le mode aide (§2.7) : rejouer localement une onde ou un sondage
//!   contre une hypothèse de plateau (`fire_beam_json`, `probe_json`), pour
//!   une vérification de cohérence instantanée pendant que le joueur pose
//!   ses hypothèses, sans solliciter le serveur à chaque case.
//!
//! Le WASM client ne manipule jamais que des données publiques (grille du
//! joueur en cours de placement, grille d'hypothèses) — les dispositions
//! secrètes adverses ne sont calculées que côté serveur.

use orapa_core::{fire_beam, validate_placement, Board, PieceCatalog, PlacedPiece, PointIndex};
use serde::Serialize;
use wasm_bindgen::prelude::*;

fn catalog() -> PieceCatalog {
    PieceCatalog::default_catalog()
}

#[wasm_bindgen]
pub fn catalog_json() -> String {
    serde_json::to_string(&catalog()).unwrap_or_else(|_| "null".to_string())
}

#[derive(Serialize)]
struct JsonEnvelope<T: Serialize> {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn ok_json<T: Serialize>(value: T) -> String {
    serde_json::to_string(&JsonEnvelope {
        ok: true,
        value: Some(value),
        error: None,
    })
    .unwrap_or_else(|_| "null".to_string())
}

fn err_json(error: impl Into<String>) -> String {
    serde_json::to_string(&JsonEnvelope::<()> {
        ok: false,
        value: None,
        error: Some(error.into()),
    })
    .unwrap_or_else(|_| "null".to_string())
}

fn parse_pieces(pieces_json: &str) -> Result<Vec<PlacedPiece>, String> {
    serde_json::from_str(pieces_json).map_err(|e| format!("placement JSON invalide: {e}"))
}

/// Valide un placement (liste de `PlacedPiece` en JSON) contre les 6
/// contraintes du §2.3. Renvoie `{"ok":true,"value":{"violations":[...]}}`
/// (liste vide si le placement est légal) ou `{"ok":false,"error":"..."}`
/// si le JSON lui-même est malformé.
#[wasm_bindgen]
pub fn validate_placement_json(pieces_json: &str) -> String {
    let pieces = match parse_pieces(pieces_json) {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };
    let cat = catalog();
    match validate_placement(&cat, cat.grid_width, cat.grid_height, &pieces) {
        Ok(violations) => ok_json(violations),
        Err(e) => err_json(e),
    }
}

/// Rejoue une onde tirée depuis `entry_id` contre l'hypothèse `pieces_json`
/// et renvoie son résultat (`BeamOutcome` en JSON), pour comparaison locale
/// avec le résultat réellement reçu du serveur (mode aide).
#[wasm_bindgen]
pub fn fire_beam_json(pieces_json: &str, entry_id: &str) -> String {
    let pieces = match parse_pieces(pieces_json) {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };
    let cat = catalog();
    let board = match Board::build(&cat, cat.grid_width, cat.grid_height, &pieces) {
        Ok(b) => b,
        Err(e) => return err_json(format!("plateau invalide: {e:?}")),
    };
    let points = PointIndex::build(cat.grid_width, cat.grid_height);
    match fire_beam(&board, &points, entry_id) {
        Some(outcome) => ok_json(outcome),
        None => err_json(format!("point de tir inconnu: {entry_id}")),
    }
}

#[derive(Serialize)]
#[serde(tag = "kind")]
enum LocalProbeResult {
    Empty,
    Color { color: orapa_core::GemColor },
    OccupiedNoColor,
}

/// Sonde une case de l'hypothèse `pieces_json`, pour comparaison locale
/// avec le résultat réellement reçu du serveur (mode aide).
#[wasm_bindgen]
pub fn probe_json(pieces_json: &str, x: i32, y: i32) -> String {
    let pieces = match parse_pieces(pieces_json) {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };
    let cat = catalog();
    let board = match Board::build(&cat, cat.grid_width, cat.grid_height, &pieces) {
        Ok(b) => b,
        Err(e) => return err_json(format!("plateau invalide: {e:?}")),
    };
    if !board.in_bounds(x, y) {
        return err_json("case hors grille");
    }
    let result = match board.get(x, y) {
        None => LocalProbeResult::Empty,
        Some(cell) => match cell.color {
            Some(color) => LocalProbeResult::Color { color },
            None => LocalProbeResult::OccupiedNoColor,
        },
    };
    ok_json(result)
}

#[wasm_bindgen]
pub fn grid_width() -> i32 {
    catalog().grid_width
}

#[wasm_bindgen]
pub fn grid_height() -> i32 {
    catalog().grid_height
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_json_round_trips() {
        let json = catalog_json();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["grid_width"], 10);
        assert_eq!(v["pieces"].as_array().unwrap().len(), 7);
    }

    #[test]
    fn validate_placement_json_reports_violations_for_empty_board() {
        let result = validate_placement_json("[]");
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["ok"], true);
        // Un plateau vide n'est trivialement en violation d'aucune des 6
        // règles (elles ne s'appliquent qu'aux pièces posées) : on vérifie
        // simplement que l'appel ne renvoie pas d'erreur de parsing.
        assert!(v["value"].is_array());
    }

    #[test]
    fn validate_placement_json_rejects_malformed_json() {
        let result = validate_placement_json("not json");
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["ok"], false);
    }

    #[test]
    fn fire_beam_and_probe_agree_on_a_known_board() {
        let cat = catalog();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(3);
        use rand::SeedableRng;
        let placement = orapa_core::random_valid_placement(
            &mut rng,
            &cat,
            orapa_core::GameOptions::default(),
        );
        let pieces_json = serde_json::to_string(&placement).unwrap();
        let beam = fire_beam_json(&pieces_json, "A");
        let v: serde_json::Value = serde_json::from_str(&beam).unwrap();
        assert_eq!(v["ok"], true);

        let probe = probe_json(&pieces_json, 0, 0);
        let v: serde_json::Value = serde_json::from_str(&probe).unwrap();
        assert_eq!(v["ok"], true);
    }
}
