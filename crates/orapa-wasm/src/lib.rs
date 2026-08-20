//! Bindings WASM pour `orapa-core`, utilisés côté client pour l'aide au
//! placement (validation temps réel) et le mode aide (vérification des
//! hypothèses contre les indices reçus). Étoffé à l'étape 4 du plan
//! (client React) ; stub minimal pour l'instant afin que le workspace
//! compile de bout en bout.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
