use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

#[component]
pub fn EcosystemApplication(title: &'static str, application_name: &'static str) -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="peerfora-foundation-style" href="/pkg/application.css" />
        <Title text=title />
        <Router>
            <main>
                <header><p>{application_name}</p></header>
                <Routes fallback=|| view! { <Page title="Page not found" /> }>
                    <Route path=path!("/") view=move || view! { <Page title=title /> } />
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn Page(title: &'static str) -> impl IntoView {
    view! {
        <section aria-labelledby="page-title">
            <h1 id="page-title">{title}</h1>
            <p>This application foundation is ready. Product layouts will follow approved designs.</p>
        </section>
    }
}

fn render_document_shell(
    options: leptos::config::LeptosOptions,
    application: impl IntoView,
) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <MetaTags />
                <AutoReload options=options.clone() />
                <HydrationScripts options=options />
            </head>
            <body>{application}</body>
        </html>
    }
}

pub fn render_application_shell(
    options: leptos::config::LeptosOptions,
    title: &'static str,
    application_name: &'static str,
) -> impl IntoView {
    render_document_shell(
        options,
        view! { <EcosystemApplication title=title application_name=application_name /> },
    )
}
