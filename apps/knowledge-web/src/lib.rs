use leptos::prelude::*;
use peerfora_web_foundation::EcosystemApplication;

pub const APPLICATION_TITLE: &str = "Peerfora Knowledge";
pub const PLATFORM_NAME: &str = "Peerfora";

#[cfg(feature = "hydrate")]
mod hydrate;

#[component]
pub fn KnowledgeWebApp() -> impl IntoView {
    view! { <EcosystemApplication title=APPLICATION_TITLE application_name=PLATFORM_NAME /> }
}
