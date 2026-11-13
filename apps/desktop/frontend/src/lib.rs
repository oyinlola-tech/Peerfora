use leptos::mount::mount_to_body;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start_peerfora_desktop_frontend() {
    mount_to_body(PeerforaDesktopApp);
}

#[component]
fn PeerforaDesktopApp() -> impl IntoView {
    view! {
        <main>
            <header><a href="/">"Peerfora"</a></header>
            <Router>
                <Routes fallback=|| view! { <DesktopPlaceholderPage /> }>
                    <Route path=path!("/") view=DesktopPlaceholderPage />
                    <Route path=path!("/communities") view=DesktopPlaceholderPage />
                    <Route path=path!("/inbox") view=DesktopPlaceholderPage />
                    <Route path=path!("/settings") view=DesktopPlaceholderPage />
                </Routes>
            </Router>
        </main>
    }
}

#[component]
fn DesktopPlaceholderPage() -> impl IntoView {
    view! {
        <section aria-label="Peerfora desktop development preview">
            <h1>"Peerfora desktop foundation is ready"</h1>
            <p>"This development preview will be replaced after product design approval."</p>
        </section>
    }
}
