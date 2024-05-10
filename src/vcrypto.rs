use core::str;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serverless_discord_interactions::twilight_model::user::User as DiscordUser;
pub type Metadata = HashMap<String, String>;

#[derive(Deserialize,Serialize,Debug)]
pub struct User {
    pub id: String,
    pub discord: Option<DiscordUser>
}
#[derive(Serialize, Deserialize,Debug)]
pub enum Status {
    Pending,
    Approved,
    Canceled,
    Denied,
}

#[derive(Deserialize, Serialize,Debug)]
pub struct Currency {
    pub id: String,
    pub unit: String,
    pub guild: String,
    pub name: String,
}

#[derive(Deserialize, Serialize,Debug)]
pub struct Claim {
    pub id: String,
    pub amount: String,
    pub claimant: User,
    pub payer: User,
    pub currency: Currency,
    pub status: Status,
    pub metadata: Metadata,
}

pub struct VCryptoREST<'a> {
    api: &'a str,
    token: &'a str,
    client: reqwest::Client,
}
#[derive(Serialize)]
struct CreateClaim {
    payer_discord_id: String,
    unit: String,
    amount: String,
    metadata: Metadata,
}
const API: &str = "https://vcrypto.sumidora.com/api/v2";
impl<'a> VCryptoREST<'a> {
    pub fn new(token: &'a str) -> Self {
        Self {
            api: &API,
            client: reqwest::Client::new(),
            token,
        }
    }
    pub async fn create_claim<'b>(
        &self,
        payer_discord_id: &str,
        unit: &str,
        amount: &str,
        metadata: impl IntoIterator<Item = (&'b str, &'b str)>,
    ) -> anyhow::Result<Claim> {
        let url = format!("{}/users/@me/claims", self.api);
        let body = CreateClaim {
            amount: amount.to_owned(),
            metadata: metadata
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
            payer_discord_id: payer_discord_id.to_owned(),
            unit: unit.to_owned(),
        };
        let res = self
            .client
            .post(&url)
            .bearer_auth(self.token)
            .json(&body)
            .send()
            .await?;
        Ok(res.json().await?)
    }
}
