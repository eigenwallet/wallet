use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use std::str::FromStr;

use anyhow::Result;
use tracing::field::Field;
use tracing_subscriber::filter::{Directive, LevelFilter};
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter, Layer};

use crate::cli::api::tauri_bindings::{CliLogEmittedEvent, TauriEmitter, TauriHandle};

/// Output formats for logging messages.
pub enum Format {
    /// Standard, human readable format.
    Raw,
    /// JSON, machine readable format.
    Json,
}

/// Initialize tracing and enable logging messages according to these options.
/// Besides printing to `stdout`, this will append to a log file.
/// Said file will contain JSON-formatted logs of all levels,
/// disregarding the arguments to this function.
pub fn init(
    level_filter: LevelFilter,
    format: Format,
    dir: impl AsRef<Path>,
    tauri_handle: Option<TauriHandle>,
) -> Result<()> {
    // file logger will always write in JSON format and with timestamps
    let file_appender = tracing_appender::rolling::never(&dir, "swap-all.log");

    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_timer(UtcTime::rfc_3339())
        .with_target(false)
        .json()
        .with_filter(env_filter(level_filter)?);

    // terminal logger
    let is_terminal = atty::is(atty::Stream::Stderr);
    let terminal_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(is_terminal)
        .with_timer(UtcTime::rfc_3339())
        .with_target(false);

    // tauri layer (forwards logs to the tauri guest when connected)
    let tauri_layer = TauriEmitLayer::new(tauri_handle)
        .with_filter(env_filter(level_filter)?);

    // combine the layers and start logging, format with json if specified
    if let Format::Json = format {
        tracing_subscriber::registry()
            .with(file_layer)
            .with(tauri_layer)
            .with(terminal_layer.json().with_filter(level_filter))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(file_layer)
            .with(tauri_layer)
            .with(terminal_layer.with_filter(level_filter))
            .init();
    }

    // now we can use the tracing macros to log messages
    tracing::info!(%level_filter, logs_dir=%dir.as_ref().display(), "Initialized tracing");

    Ok(())
}

/// This function controls which crate's logs actually get logged and from which level.
fn env_filter(level_filter: LevelFilter) -> Result<EnvFilter> {
    Ok(EnvFilter::from_default_env()
        .add_directive(Directive::from_str(&format!("asb={}", &level_filter))?)
        .add_directive(Directive::from_str(&format!("swap={}", &level_filter))?))
}

/// Emit log messages to the tauri guest.
struct TauriEmitLayer<Subscriber> {
    tauri_handle: Option<TauriHandle>,
    _subscriber: std::marker::PhantomData<Subscriber>,
}

impl<Subscriber> TauriEmitLayer<Subscriber> {
    fn new(tauri_handle: Option<TauriHandle>) -> Self {
        Self {
            tauri_handle,
            _subscriber: std::marker::PhantomData,
        }
    }
}

impl<Subscriber> Layer<Subscriber> for TauriEmitLayer<Subscriber>
where
    Subscriber: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, Subscriber>,
    ) {
        let level = event.metadata().level().as_str().to_owned();
        let span = event.metadata().name().to_owned();
        let mut fields = HashMap::new();

        let mut message: Option<String> = None;

        // Visit every field of the event and put it into the map
        event.record(&mut |field: &Field, value: &dyn Debug| {
            if field.name() == "message" {
                message = Some(format!("{:?}", value));
                return;
            }
            fields.insert(field.name().into(), format!("{:?}", value));
        });

        let log_event = CliLogEmittedEvent {
            level,
            span,
            message,
            fields,
        };

        self.tauri_handle.emit_cli_log_event(log_event);
    }
}
