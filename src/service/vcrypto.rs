use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use worker::*;

use crate::{
    vcrypto::{Currency, Metadata, User},
    verifying_key::verifying_key,
};
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
enum WebhookKind {
    Ping = 1,
    ClaimUpdate = 2,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateClaimEvent {
    id: u64,
    status: String,
    amount: String,
    updated_at: String,
    metadata: Metadata,
    payer: User,
    currency: Currency,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebhookBody {
    #[serde(rename = "type")]
    kind: WebhookKind,
    data: Option<Vec<UpdateClaimEvent>>,
}

const VIRTUAL_CRYPTO_WEBHOOK_RESPONSE_TYPE_PONG: u32 = 1;
#[derive(Debug, Serialize, Deserialize)]
struct VirtualCryptoWebhookResponseBody {
    #[serde(rename = "type")]
    pub kind: u32,
}
pub async fn process_request(
    mut req: Request,
    env: &Env,
    ctx: Context,
) -> anyhow::Result<Response> {
    let vk = verifying_key(env, "VCRYPTO_PUBLIC_KEY");

    let sign = req
        .headers()
        .get("X-Signature-Ed25519")?
        .ok_or_else(|| anyhow::anyhow!("missing request header: X-Signature-Ed25519"))?;
    let timestamp = req
        .headers()
        .get("X-Signature-Timestamp")?
        .ok_or_else(|| anyhow::anyhow!("missing request header: X-Signature-Timestamp"))?;
    let body = req.text().await?;
    match verify(&vk, &body, &timestamp, &sign) {
        anyhow::Result::Ok(_) => {
            let body: WebhookBody = serde_json::from_str(&body)?;
            if body.kind == WebhookKind::Ping {
                Ok(Response::from_json(&VirtualCryptoWebhookResponseBody {
                    kind: VIRTUAL_CRYPTO_WEBHOOK_RESPONSE_TYPE_PONG,
                })?)
            } else {
                let env = env.clone();
                ctx.wait_until(async move { on_webhook(&body.data.unwrap(), &env).await });
                Ok(Response::empty()?)
            }
        }
        Err(_) => Ok(Response::error("invalid request signature", 401)?),
    }
}

async fn on_webhook(_events: &[UpdateClaimEvent], _env: &Env) {
    
}

fn verify(vk: &VerifyingKey, body: &str, timestamp: &str, sign: &str) -> anyhow::Result<()> {
    let mut sign_array: [u8; 64] = [0; 64];
    hex::decode_to_slice(sign, &mut sign_array)?;

    vk.verify_strict(
        format!("{}{}", timestamp, body).as_bytes(),
        &Signature::from_bytes(&sign_array),
    )?;
    Ok(())
}
