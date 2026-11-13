use leptos::mount::hydrate_body;
use peerfora_web::PeerforaWebApp;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn hydrate() {
    hydrate_body(PeerforaWebApp);
}
