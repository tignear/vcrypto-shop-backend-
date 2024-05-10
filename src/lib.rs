use serverless_discord_interactions_cloudflare_backend::RequestProcessor;
use tracing::instrument;
use tracing_subscriber::{fmt::{format::Pretty, time::UtcTime}, layer::SubscriberExt as _, util::SubscriberInitExt as _};
use tracing_web::{performance_layer, MakeConsoleWriter};
use worker::*;

mod company;
mod service;
mod vcrypto;
mod verifying_key;

#[event(start)]
fn start() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_ansi(false) // Only partially supported across JavaScript runtimes
        .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
        .with_writer(MakeConsoleWriter); // write events to the console
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();
}

#[event(fetch)]
#[instrument(skip(env,ctx))]
async fn main(req: Request, env: Env, ctx: Context) -> Result<worker::Response> {
    match sub_main(req, env, ctx).await {
        Err(err) => {
            tracing::error!("{}", err);
            Ok(Response::error("bad request", 400)?)
        }
        Ok(v) => Ok(v),
    }
}
#[instrument(skip(env,ctx))]
async fn sub_main(req: Request, env: Env, ctx: Context) -> anyhow::Result<Response> {
    use service::discord;
    if req.path().starts_with("/discord") {
        let vk = verifying_key::verifying_key(&env, "DISCORD_PUBLIC_KEY");
        Ok(RequestProcessor::new(vk, discord::Handler { env, ctx })
            .process_request(req)
            .await?)
    } else if req.path().starts_with("/vcrypto") {
        Ok(service::vcrypto::process_request(req, &env, ctx).await?)
    } else {
        Ok(Response::error("not found", 404)?)
    }
}
