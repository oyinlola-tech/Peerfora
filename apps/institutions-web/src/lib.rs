use leptos::prelude::*;
use peerfora_web_foundation::EcosystemApplication;

pub const APPLICATION_TITLE: &str = "Peerfora Institutions";
pub const PLATFORM_NAME: &str = "Peerfora";

#[cfg(feature = "hydrate")]
mod hydrate;

#[component]
pub fn InstitutionsWebApp() -> impl IntoView {
    view! { <EcosystemApplication title=APPLICATION_TITLE application_name=PLATFORM_NAME /> }
}
