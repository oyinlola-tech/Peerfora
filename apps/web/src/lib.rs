use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

#[cfg(feature = "hydrate")]
mod hydrate;

#[component]
pub fn PeerforaWebApp() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="peerfora-web-style" href="/pkg/peerfora-web.css" />
        <link rel="manifest" href="/manifest.webmanifest" />
        <Title text="Peerfora" />
        <Router>
            <main class="mx-auto min-h-screen max-w-5xl px-6 py-10">
                <header class="mb-10">
                    <a href="/" class="text-xl font-semibold">"Peerfora"</a>
                </header>
                <Routes fallback=|| view! { <PagePlaceholder title="Page not found" /> }>
                    <Route path=path!("/") view=HomePage />
                    <Route path=path!("/login") view=LoginPage />
                    <Route path=path!("/register") view=RegistrationPage />
                    <Route path=path!("/complete-social-signup") view=SocialSignupCompletionPage />
                    <Route path=path!("/communities") view=CommunityListPage />
                    <Route path=path!("/communities/:slug") view=CommunityDetailPage />
                    <Route path=path!("/posts/:id") view=PostDetailPage />
                    <Route path=path!("/search") view=SearchPage />
                    <Route path=path!("/profile/:username") view=ProfilePage />
                    <Route path=path!("/inbox") view=InboxPage />
                    <Route path=path!("/settings") view=SettingsPage />
                    <Route path=path!("/saved") view=SavedItemsPage />
                    <Route path=path!("/opportunities") view=OpportunitiesPage />
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! { <PagePlaceholder title="Peerfora web foundation is ready" /> }
}

#[component]
fn LoginPage() -> impl IntoView {
    view! { <PagePlaceholder title="Sign in" /> }
}

#[component]
fn RegistrationPage() -> impl IntoView {
    view! { <PagePlaceholder title="Create an account" /> }
}

#[component]
fn SocialSignupCompletionPage() -> impl IntoView {
    view! { <PagePlaceholder title="Complete your Peerfora sign-up" /> }
}

#[component]
fn CommunityListPage() -> impl IntoView {
    view! { <PagePlaceholder title="Communities" /> }
}

#[component]
fn CommunityDetailPage() -> impl IntoView {
    view! { <PagePlaceholder title="Community" /> }
}

#[component]
fn PostDetailPage() -> impl IntoView {
    view! { <PagePlaceholder title="Discussion" /> }
}

#[component]
fn SearchPage() -> impl IntoView {
    view! { <PagePlaceholder title="Search" /> }
}

#[component]
fn ProfilePage() -> impl IntoView {
    view! { <PagePlaceholder title="Profile" /> }
}

#[component]
fn InboxPage() -> impl IntoView {
    view! { <PagePlaceholder title="Inbox" /> }
}

#[component]
fn SettingsPage() -> impl IntoView {
    view! { <PagePlaceholder title="Settings" /> }
}

#[component]
fn SavedItemsPage() -> impl IntoView {
    view! { <PagePlaceholder title="Saved items" /> }
}

#[component]
fn OpportunitiesPage() -> impl IntoView {
    view! { <PagePlaceholder title="Opportunities" /> }
}

#[component]
fn PagePlaceholder(title: &'static str) -> impl IntoView {
    view! {
        <section aria-labelledby="page-title">
            <h1 id="page-title" class="text-2xl font-semibold">{title}</h1>
            <p class="mt-3">"Development foundation. Product layouts will follow approved designs."</p>
        </section>
    }
}

pub fn render_web_document_shell(options: leptos::config::LeptosOptions) -> impl IntoView {
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
            <body>
                <PeerforaWebApp />
            </body>
        </html>
    }
}
