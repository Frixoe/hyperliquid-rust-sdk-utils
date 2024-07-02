use tokio::task::JoinHandle;
use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::EnvFilter;

pub fn get_subscriber<Sink>(
    _name: String,
    env_filter: String,
    _sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    tracing_subscriber::fmt()
        .with_ansi(true)
        .with_env_filter(env_filter)
        .with_thread_ids(true)
        .with_level(true)
        .finish()
    //let formatting_layer = BunyanFormattingLayer::new(name, sink);
    //Registry::default()
    //    .with(env_filter)
    //    .with(JsonStorageLayer)
    //    .with(formatting_layer)
}

/// Register a subscriber as a global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");

    set_global_default(subscriber).expect("Failed to set subscriber");
}

pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}
