use crate::KnowledgeWebApp;
use leptos::mount::hydrate_body;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn hydrate() {
    hydrate_body(KnowledgeWebApp);
}
