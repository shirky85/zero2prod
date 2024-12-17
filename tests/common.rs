use std::net::TcpListener;
use zero2prod::in_memory::AppState;
use actix_web::web;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use std::sync::LazyLock;
/// The closure passed to LazyLock::new is executed only once, when TRACING is first accessed.
/// This ensures that the tracing stack is initialized only once.
/// It is is thread-safe, meaning it can be safely accessed from multiple threads without 
/// fear of multiple initializations.
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    // We cannot assign the output of `get_subscriber` to a variable based on the
    // value TEST_LOG` because the sink is part of the type returned by
    // `get_subscriber`, therefore they are not the same type. We could work around
    // it, but this is the most straight-forward way of moving forward.
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(
            subscriber_name,
            default_filter_level,
            std::io::stdout
        );
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(
            subscriber_name,
            default_filter_level,
            std::io::sink
        );
        init_subscriber(subscriber);
    };
});

pub fn spawn_app(listener: TcpListener) {
    // This line forces the initialization of the TRACING variable if it has not already been initialized.
    // If we call it again later on in the code - it will do nothing.
    LazyLock::force(&TRACING);
    let app_state: AppState = AppState::new();
    let data_store_shared = web::Data::new(app_state);
    let server = zero2prod::startup::run(listener, data_store_shared).expect("Failed to bind address");
    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let
    let _ = tokio::spawn(server);
}