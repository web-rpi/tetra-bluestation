use std::sync::Once;
use std::fs::OpenOptions;
use std::fmt;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt as tracingfmt, EnvFilter};
use tracing_subscriber::prelude::*;
use tracing_subscriber::fmt::format::{self, FormatEvent, FormatFields};
use tracing_subscriber::fmt::FmtContext;
use tracing_subscriber::registry::LookupSpan;


#[macro_export]
macro_rules! unimplemented_log {
    ( $($arg:tt)* ) => {{
        tracing::warn!(
            "unimplemented: {}",
            format_args!($($arg)*),
            // file!(),
            // line!(),
        );
    }};
}

/// if `cond` is false, logs a warning with your message.
#[macro_export]
macro_rules! assert_warn {
    ($cond:expr, $($arg:tt)+) => {{
        if !$cond {
            tracing::warn!(
                target: module_path!(),
                "assertion warning: `{}` failed: {} at {}:{}",
                stringify!($cond),
                format_args!($($arg)+),
                file!(),
                line!(),
            );
        }
    }};
}

struct AlignedFormatter;

impl<S, N> FormatEvent<S, N> for AlignedFormatter
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> fmt::Result {
        let metadata = event.metadata();
        
        // Add ANSI color codes for different log levels
        let (level_color, reset) = match *metadata.level() {
            tracing::Level::ERROR => ("\x1b[31m", "\x1b[0m"),
            tracing::Level::WARN => ("\x1b[33m", "\x1b[0m"),
            tracing::Level::INFO => ("\x1b[32m", "\x1b[0m"),
            tracing::Level::DEBUG => ("\x1b[34m", "\x1b[0m"),
            tracing::Level::TRACE => ("\x1b[35m", "\x1b[0m"),
        };
        
        // Format: "LEVEL file:line: message"
        let location = format!(
            "{}{}{} {}:{}:",
            level_color,
            metadata.level(),
            reset,
            metadata.file().unwrap_or("unknown"),
            metadata.line().unwrap_or(0)
        );
        
        // Capture the message to check for special prefixes
        let mut message_buf = String::new();
        let message_writer = format::Writer::new(&mut message_buf);
        ctx.field_format().format_fields(message_writer, event)?;
        
        // Check if the message starts with "->" or "<-" to reduce indentation
        let mut padding = 60; // Default alignment
        if message_buf.starts_with("->") || message_buf.starts_with("<-") {
            padding -= 3;  // Reduce by 3 characters
        }
        
        write!(writer, "{:<width$} {}", location, message_buf, width = padding)?;
        writeln!(writer)
    }
}

static INIT_LOG: Once = Once::new();

/// Sets up logging with maximum verbosity (trace level)
/// Mainly for unit tests
pub fn setup_logging_verbose() {
    let stdout_filter = EnvFilter::new("trace");
    setup_logging(stdout_filter, None);
}

/// Sets up default logging to stdout and optionally, a verbose log file
/// Returns a guard, that needs to be kept alive for logging to file to work
pub fn setup_logging_default(verbose_logfile: Option<String>) -> Option<WorkerGuard> {

    let stdout_filter = get_default_stdout_filter();
    let logfile_and_filter = if let Some(file) = verbose_logfile {
        Some((file, get_default_logfile_filter()))
    } else {
        None
    };
    setup_logging(stdout_filter, logfile_and_filter)
}    

pub fn get_default_filter() -> EnvFilter {
    EnvFilter::new("info")
}

pub fn get_default_stdout_filter() -> EnvFilter {

    EnvFilter::new("info")

        // Hide continuous logs from lower layers
        .add_directive("tetra_bluestation::common::messagerouter=warn".parse().unwrap())
        .add_directive("tetra_bluestation::common::bitbuffer=warn".parse().unwrap())

        // Basic level for tetra entities
        // .add_directive("tetra_bluestation::entities=info".parse().unwrap())

        // Phy
        .add_directive("tetra_bluestation::entities::phy::components=warn".parse().unwrap())
        .add_directive("tetra_bluestation::entities::phy::phy_bs=debug".parse().unwrap())
        
        
        // Lmac
        .add_directive("tetra_bluestation::entities::lmac=info".parse().unwrap())
        .add_directive("tetra_bluestation::entities::lmac::components=info".parse().unwrap())

        // Umac
        .add_directive("tetra_bluestation::entities::umac::subcomp::slotter=debug".parse().unwrap())
        .add_directive("tetra_bluestation::entities::umac=debug".parse().unwrap())

        // Llc
        .add_directive("tetra_bluestation::entities::llc=debug".parse().unwrap())

        // Higher layers
        .add_directive("tetra_bluestation::entities::mle=trace".parse().unwrap())
        .add_directive("tetra_bluestation::entities::cmce=trace".parse().unwrap())
        .add_directive("tetra_bluestation::entities::sndcp=trace".parse().unwrap())
        .add_directive("tetra_bluestation::entities::mm=trace".parse().unwrap())
}


fn get_default_logfile_filter() -> EnvFilter {
    EnvFilter::new("debug")
}
// fn setup_default_logging

/// Sets up logging to stdout and optionally, a verbose log file
/// If an output file  is requested, returns Some<WorkerGuard>. Keep this value alive
/// or logging to file may cease working. If no output file is provided, returns None. 
fn setup_logging(stdout_filter: EnvFilter, outfile: Option<(String, EnvFilter)>) -> Option<WorkerGuard> {

    if let Some((outfile, outfile_filter)) = outfile {
        // Setup logging with a verbose log file
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(outfile)
            .expect("Failed to open log file");
        let (file_writer, guard) = tracing_appender::non_blocking(file);
        
        // Setup once
        INIT_LOG.call_once(||{
            let file_layer = tracingfmt::layer()
                .event_format(AlignedFormatter)
                .with_writer(file_writer)
                .with_ansi(false);

            // Change both here and below in the non-logfile variant.     
            let stdout_layer = tracingfmt::layer()
                .event_format(AlignedFormatter);
                
            tracing_subscriber::registry()
                .with(file_layer.with_filter(outfile_filter))
                .with(stdout_layer.with_filter(stdout_filter))
                .init();
        });

        Some(guard)
    } else {
        // Setup once
        INIT_LOG.call_once(||{
            
            // Change both here and below in the non-logfile variant.     
            let stdout_layer = tracingfmt::layer()
                .event_format(AlignedFormatter);
                
            tracing_subscriber::registry()
                .with(stdout_layer.with_filter(stdout_filter))
                .init();
        });
        None
    }
}
