// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use clap::Parser;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::{rt::TokioIo, server::graceful::GracefulShutdown};
use std::{net::SocketAddr, pin::pin, time::Duration};
use tokio::{net::TcpListener, time};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{addr}");
    if let Some(timeout) = args.timeout {
        println!(
            "Shutting down in {}, or press Ctrl+C",
            humantime::format_duration(timeout)
        );
    } else {
        println!("Press Ctrl+C to shut down");
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
                println!("Request from {source:?}");

                let io = TokioIo::new(stream);
                let conn = http1::Builder::new().serve_connection(io, service_fn(serve::serve));
                let watcher = graceful.watch(conn);
                tokio::task::spawn(async move {
                    if let Err(err) = watcher.await
                    {
                        eprintln!("Error serving connection: {err:?}");
                    }
                });
            },
            _ = &mut signal => {
                println!("Shutting down...");
                break;
            },
            _ = time::sleep(args.timeout.unwrap_or_default()), if args.timeout.is_some() => {
                eprintln!("Shutting down after {} timeout...", humantime::format_duration(args.timeout.unwrap()));
                break;
            }
        };
    }

    println!("Closing connections");
    tokio::select! {
        _ = graceful.shutdown() => {
            println!("All connections closed");
        },
        _ = time::sleep(Duration::from_secs(3)) => {
            eprintln!("Timed out waiting for connections to close");
            std::process::exit(1);
        }
    }

    Ok(())
}

#[derive(Debug, Parser)]
#[command(author, version)]
struct Args {
    /// Port to bind.
    #[arg(short = 'p', long, default_value_t = 4000)]
    pub port: u16,

    /// When to shut down the service e.g., 500ms, 10s, "1 hour", etc.
    #[arg(long, value_parser = humantime::parse_duration)]
    pub timeout: Option<Duration>,
}
