use leptos::prelude::*;
use peerfora_web_foundation::EcosystemApplication;

pub const APPLICATION_TITLE: &str = "Peerfora Agents";
pub const PLATFORM_NAME: &str = "Peerfora";

#[cfg(feature = "hydrate")]
mod hydrate;

#[component]
pub fn AgentsWebApp() -> impl IntoView {
    view! { <EcosystemApplication title=APPLICATION_TITLE application_name=PLATFORM_NAME /> }
}
