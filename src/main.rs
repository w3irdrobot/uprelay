#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{routing::post, Router};
    use futures::future;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use std::env;
    use uprelay::{app::*, db, fileserv::file_and_error_handler, nostr::*};

    dotenvy::dotenv().unwrap();
    simple_logger::init_with_level(log::Level::Debug).expect("couldn't initialize logging");

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(|cx| view! { cx, <App/> }).await;

    let database_url = env::var("DATABASE_URL").unwrap();
    let pool = db::new_db_pool(&database_url).await.unwrap();
    let relay_repo = RelayRepository::new(pool.clone()).unwrap();
    let nostr = Nostr::new(relay_repo.clone()).await.unwrap();

    let hydrate = nostr.hydrate();

    // build our application with a route
    let app = Router::new()
        .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
        .leptos_routes(&leptos_options, routes, |cx| view! { cx, <App/> })
        .fallback(file_and_error_handler)
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let server = axum::Server::bind(&addr).serve(app.into_make_service());

    future::join(hydrate, server).await;
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
