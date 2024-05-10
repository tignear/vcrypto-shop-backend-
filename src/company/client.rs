use anyhow::bail;
use tracing::instrument;
use worker::*;

use crate::{company::company::StartTransactionRequestData, vcrypto::Claim};

use super::{product::Product, RegisterProductRequest, COMPANY_DURABLE_OBJECT_BINDING_KEY};

pub(crate) struct CompanyClient {
    env: Env,
    guild_id: String,
}
#[derive(Debug)]
pub(crate) enum StartTransactionResult {
    Ok(Claim),
    InvalidRequest,
}
impl CompanyClient {
    pub(crate) fn new(env: Env, guild_id: String) -> Self {
        Self { env, guild_id }
    }
    pub fn get_stub(&self) -> anyhow::Result<Stub> {
        let ns = self
            .env
            .durable_object(COMPANY_DURABLE_OBJECT_BINDING_KEY)?;
        let stub = ns.id_from_name(&self.guild_id)?.get_stub()?;
        Ok(stub)
    }
    #[instrument(skip(self))]
    pub async fn start_transaction(
        &self,
        product_name: String,
        user_id: String,
    ) -> anyhow::Result<StartTransactionResult> {
        tracing::debug!("????");
        let body = StartTransactionRequestData {
            product_name: product_name.into(),
            user_id: user_id.into(),
        };
        tracing::debug!("x");

        let body = serde_json::to_string(&body)?;
        tracing::debug!("!");
        tracing::debug!("!?");

        let mut res = match self
            .get_stub()?
            .fetch_with_request(Request::new_with_init(
                "http://tignear.com/transactions",
                RequestInit::new()
                    .with_method(Method::Post)
                    .with_body(Some(body.into())),
            )?)
            .await
        {
            Ok(v) => v,
            Err(err) => {
                tracing::error!("{:?}", err);
                return Err(err.into());
            }
        };
        tracing::debug!("!!");

        if res.status_code() == 200 {
            let data: Claim = res.json().await?;

            Ok(StartTransactionResult::Ok(data))
        } else if res.status_code() == 400 {
            tracing::error!("{:?}",res.text().await);
            Ok(StartTransactionResult::InvalidRequest)
        } else {
            bail!(res.text().await?)
        }
    }
    pub async fn register_product(
        &self,
        product_name: impl Into<String>,
        product_data: Product,
    ) -> anyhow::Result<()> {
        let body = RegisterProductRequest {
            product_name: product_name.into(),
            product_data,
        };
        let body = serde_json::to_string(&body)?;
        let mut header = Headers::new();
        header.append("content-type", "application/json")?;
        let res = self
            .get_stub()?
            .fetch_with_request(Request::new_with_init(
                "http://tignear.com/products",
                RequestInit::new()
                    .with_headers(header)
                    .with_method(Method::Post)
                    .with_body(Some(body.into())),
            )?)
            .await?;
        tracing::debug!("cccc");
        let code = res.status_code();
        if 200 <= code && code <= 299 {
            Ok(())
        } else {
            bail!("{:?}", res)
        }
    }
}
