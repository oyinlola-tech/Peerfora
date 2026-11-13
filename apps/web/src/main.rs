use axum::Router;
use axum::http::HeaderValue;
use axum::http::header::CACHE_CONTROL;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use peerfora_web::{PeerforaWebApp, render_web_document_shell};
use tower_http::set_header::SetResponseHeaderLayer;

#[tokio::main]
async fn main() {
    let configuration = get_configuration(None).expect("Web configuration must be valid");
    let leptos_options = configuration.leptos_options;
    let address = leptos_options.site_addr;
    let routes = generate_route_list(PeerforaWebApp);
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let options = leptos_options.clone();
            move || render_web_document_shell(options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(
            render_web_document_shell,
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            CACHE_CONTROL,
            HeaderValue::from_static("private, no-store"),
        ))
        .with_state(leptos_options);
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("Web listener must bind");
    axum::serve(listener, app)
        .await
        .expect("Web server must continue serving requests");
}
