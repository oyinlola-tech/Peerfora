use leptos::mount::mount_to_body;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::hooks::use_location;
use leptos_router::path;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start_peerfora_admin_desktop_frontend() {
    mount_to_body(PeerforaAdminDesktopApp);
}

#[component]
fn PeerforaAdminDesktopApp() -> impl IntoView {
    view! {
        <main>
            <header><a href="/overview">"Peerfora administration"</a></header>
            <Router>
                <nav aria-label="Administration sections">
                    <a href="/overview">"Overview"</a>
                    <a href="/users">"Users"</a>
                    <a href="/communities">"Communities"</a>
                    <a href="/institutions">"Institutions"</a>
                    <a href="/courses">"Courses"</a>
                    <a href="/reports">"Reports"</a>
                    <a href="/moderation">"Moderation"</a>
                    <a href="/verification">"Verification"</a>
                    <a href="/appeals">"Appeals"</a>
                    <a href="/subscriptions">"Subscriptions"</a>
                    <a href="/finance">"Finance"</a>
                    <a href="/advertising">"Advertising"</a>
                    <a href="/opportunities">"Opportunities"</a>
                    <a href="/analytics">"Analytics"</a>
                    <a href="/support">"Support"</a>
                    <a href="/system">"System"</a>
                    <a href="/audit-logs">"Audit logs"</a>
                    <a href="/configuration">"Configuration"</a>
                </nav>
                <Routes fallback=|| view! { <AdminRoutePlaceholder /> }>
                    <Route path=path!("/") view=AdminRoutePlaceholder />
                    <Route path=path!("/overview") view=AdminRoutePlaceholder />
                    <Route path=path!("/users") view=AdminRoutePlaceholder />
                    <Route path=path!("/communities") view=AdminRoutePlaceholder />
                    <Route path=path!("/institutions") view=AdminRoutePlaceholder />
                    <Route path=path!("/courses") view=AdminRoutePlaceholder />
                    <Route path=path!("/reports") view=AdminRoutePlaceholder />
                    <Route path=path!("/moderation") view=AdminRoutePlaceholder />
                    <Route path=path!("/verification") view=AdminRoutePlaceholder />
                    <Route path=path!("/appeals") view=AdminRoutePlaceholder />
                    <Route path=path!("/subscriptions") view=AdminRoutePlaceholder />
                    <Route path=path!("/finance") view=AdminRoutePlaceholder />
                    <Route path=path!("/advertising") view=AdminRoutePlaceholder />
                    <Route path=path!("/opportunities") view=AdminRoutePlaceholder />
                    <Route path=path!("/analytics") view=AdminRoutePlaceholder />
                    <Route path=path!("/support") view=AdminRoutePlaceholder />
                    <Route path=path!("/system") view=AdminRoutePlaceholder />
                    <Route path=path!("/audit-logs") view=AdminRoutePlaceholder />
                    <Route path=path!("/configuration") view=AdminRoutePlaceholder />
                </Routes>
            </Router>
        </main>
    }
}

#[component]
fn AdminRoutePlaceholder() -> impl IntoView {
    let location = use_location();
    view! {
        <section aria-label="Peerfora administration development preview">
            <h1>{move || format!("{}", location.pathname.get())}</h1>
            <p>"Administrative authentication and APIs are not connected."</p>
        </section>
    }
}
