// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

mod size;

use hyper::{body::Incoming, Request, Response};
pub use size::*;
use std::time::Duration;

pub async fn serve(
    _req: Request<Incoming>,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    tokio::time::sleep(Duration::from_millis(100000)).await;
    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body("hello, world!".into())?)
}
