use crate::error::DeepMailError;

/// Creates a NATS client connection and returns a JetStream context.
///
/// # Arguments
/// * `nats_url` — NATS server URL (e.g. `nats://localhost:4222`)
///
/// # Errors
/// Returns `DeepMailError::Nats` if the connection or JetStream
/// context acquisition fails.
#[tracing::instrument(skip(nats_url))]
pub async fn create_jetstream_context(
    nats_url: &str,
) -> Result<async_nats::jetstream::Context, DeepMailError> {
    let client = async_nats::connect(nats_url)
        .await
        .map_err(|e| DeepMailError::Nats(format!("failed to connect to NATS at {nats_url}: {e}")))?;

    let jetstream = async_nats::jetstream::new(client);

    tracing::info!("NATS JetStream context created");
    Ok(jetstream)
}

/// Ensures a JetStream stream exists with the given configuration.
///
/// Creates the stream if it doesn't exist, or updates it if the
/// config has changed. This is idempotent.
#[tracing::instrument(skip(js))]
pub async fn ensure_stream(
    js: &async_nats::jetstream::Context,
    stream_config: async_nats::jetstream::stream::Config,
) -> Result<async_nats::jetstream::stream::Stream, DeepMailError> {
    let stream = js
        .get_or_create_stream(stream_config)
        .await
        .map_err(|e| DeepMailError::Nats(format!("failed to ensure stream: {e}")))?;

    tracing::info!(stream_name = %stream.cached_info().config.name, "stream ensured");
    Ok(stream)
}
