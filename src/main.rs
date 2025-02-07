// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use anstream::AutoStream;
use anstyle::Style;
use clap::{
    builder::{styling::AnsiColor, PossibleValue, Styles},
    Parser, ValueEnum,
};
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::{rt::TokioIo, server::graceful::GracefulShutdown};
use serve::Size;
use std::{fmt, io::Write as _, net::SocketAddr, pin::pin, time::Duration};
use tokio::{net::TcpListener, time};

const CLAP_V3_STYLES: Styles = Styles::styled()
    .error(AnsiColor::Red.on_default())
    .header(AnsiColor::Yellow.on_default())
    .invalid(AnsiColor::Yellow.on_default())
    .literal(AnsiColor::Green.on_default())
    .usage(AnsiColor::Green.on_default());

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let choice = args.color.into();
    let mut stdout = AutoStream::new(std::io::stdout(), choice);
    let mut stderr = AutoStream::new(std::io::stderr(), choice);
    let success = anstyle::Style::new().fg_color(Some(AnsiColor::Green.into()));
    let warning = anstyle::Style::new().fg_color(Some(AnsiColor::Yellow.into()));
    let error = Style::new().fg_color(Some(AnsiColor::Red.into()));

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    let listener = TcpListener::bind(addr).await?;
    writeln!(stdout, "{success}Listening on http://{addr}{success:#}")?;
    if let Some(timeout) = args.timeout {
        writeln!(
            stdout,
            "Shutting down in {}, or press Ctrl+C",
            humantime::format_duration(timeout)
        )?;
    } else {
        writeln!(stdout, "Press Ctrl+C to shut down")?;
    }

    let graceful = GracefulShutdown::new();
    let mut signal = pin!(async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C signal handler")
    });

    // Accept incoming connections.
    loop {
        tokio::select! {
            Ok((stream, source)) = listener.accept() => {
                writeln!(stdout, "Request from {source:?}")?;

                let io = TokioIo::new(stream);
                let conn = http1::Builder::new().serve_connection(io, service_fn(serve::serve));
                let watcher = graceful.watch(conn);

                tokio::task::spawn(watcher);
            },
            _ = &mut signal => {
                writeln!(stdout, "{warning}Shutting down...{warning:#}")?;
                break;
            },
            _ = time::sleep(args.timeout.unwrap_or_default()), if args.timeout.is_some() => {
                writeln!(stderr, "{warning}Shutting down after {} timeout...{warning:#}", humantime::format_duration(args.timeout.unwrap()))?;
                break;
            }
        };
    }

    writeln!(stdout, "Closing connections")?;
    tokio::select! {
        _ = graceful.shutdown() => {
            writeln!(stdout, "All connections closed")?;
        },
        _ = time::sleep(Duration::from_secs(3)) => {
            writeln!(stderr, "{error}Timed out waiting for connections to close{error:#}")?;
            std::process::exit(1);
        }
    }

    Ok(())
}

#[derive(Debug, Parser)]
#[command(author, version, styles = CLAP_V3_STYLES)]
struct Args {
    /// The size of blocks of the response to send e.g., "32kb", "1 mib", etc.
    ///
    /// Supports bytes ("b") through petabytes ("pb") and pebibytes ("pib").
    #[arg(short = 'b', long)]
    pub block_size: Option<Size>,

    /// When to show color output.
    #[arg(long, default_value_t = ColorChoice::default())]
    pub color: ColorChoice,

    /// Port to bind.
    #[arg(short = 'p', long, default_value_t = 4000)]
    pub port: u16,

    /// When to shut down the service e.g., 500ms, 10s, "1 hour", etc.
    #[arg(long, value_parser = humantime::parse_duration)]
    pub timeout: Option<Duration>,
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
enum ColorChoice {
    #[default]
    Auto,
    Always,
    Never,
}

impl fmt::Display for ColorChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorChoice::Auto => f.write_str("auto"),
            ColorChoice::Always => f.write_str("always"),
            ColorChoice::Never => f.write_str("never"),
        }
    }
}

impl From<ColorChoice> for anstream::ColorChoice {
    fn from(value: ColorChoice) -> Self {
        match value {
            ColorChoice::Always => Self::Always,
            ColorChoice::Auto => Self::Auto,
            ColorChoice::Never => Self::Never,
        }
    }
}

impl ValueEnum for ColorChoice {
    fn value_variants<'a>() -> &'a [Self] {
        &[ColorChoice::Auto, ColorChoice::Always, ColorChoice::Never]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            ColorChoice::Auto => PossibleValue::new("auto"),
            ColorChoice::Always => PossibleValue::new("always"),
            ColorChoice::Never => PossibleValue::new("never"),
        })
    }
}
