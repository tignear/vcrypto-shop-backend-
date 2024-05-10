use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, mem::ManuallyDrop};
use worker::*;

use crate::vcrypto::VCryptoREST;

use super::product::Product;

#[durable_object]
pub(crate) struct Company {
    state: State,
    env: Env,
}

#[derive(Serialize, Deserialize)]
#[non_exhaustive]
pub enum ErrorCode {
    ProductGetError,
}
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct StartTransactionRequestData {
    pub user_id: String,
    pub product_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct RegisterProductRequest {
    pub product_name: String,
    pub product_data: Product,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TransactionApprovedRequestData {
    pub product_name: String,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProductGetError => write!(f, "ProductGetError"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    code: ErrorCode,
}

struct RequestContext {
    storage: Storage,
}
impl RequestContext {
    fn new(storage: Storage) -> Self {
        Self { storage }
    }
    async fn get_product(&self, product_name: &str) -> Option<Product> {
        tracing::debug!("getting product: {}", product_name);
        // The key must survive until the end of get.
        // Normally it should be extended, but it is not, so I write it like this. @tignear
        // self.storage.get(&format!("/products/{product_name}")).await; // this doesn't work
        let key = format!("/products/{product_name}");
        let result = self.storage.get(&key).await;
        std::mem::drop(key); // This guarantees that the key will survive until this point.
        let product: Product = match result {
            Ok(v) => {
                tracing::debug!("got product: {:?}", v);
                v
            }
            Err(err) => {
                tracing::error!("{:?}", err);
                return None;
            }
        };
        Some(product)
    }
    async fn register_product(&mut self, product_name: &str, product: &Product) -> Result<()> {
        tracing::debug!("registering product: {}, {:?}", product_name, product);

        self.storage
            .put(&format!("/products/{product_name}"), product)
            .await?;

        let product: Product = self
            .storage
            .get(&format!("/products/{product_name}"))
            .await?;
        tracing::debug!("registered product: {}, {:?}", product_name, product);

        Ok(())
    }
    async fn start_transaction(
        &self,
        user_id: &str,
        product_name: &str,
    ) -> anyhow::Result<Response> {
        let product = self.get_product(product_name).await;
        let product = match product {
            None => {
                return Ok(Response::from_json(&ErrorResponse {
                    code: ErrorCode::ProductGetError,
                })?
                .with_status(400))
            }
            Some(v) => v,
        };
        let rest = VCryptoREST::new("");
        match product {
            Product::Role(product) => {
                let metadata = HashMap::from([("version", "1"), ("product_name", product_name)]);
                let claim = rest
                    .create_claim(user_id, &product.unit, &product.price, metadata.into_iter())
                    .await?;
                Ok(Response::from_json(&claim)?)
            }
        }
    }
    async fn transaction_approved(&self, product_name: &str) -> anyhow::Result<Response> {
        let product = self.get_product(product_name).await;
        let product = match product {
            None => {
                return Ok(Response::from_json(&ErrorResponse {
                    code: ErrorCode::ProductGetError,
                })?
                .with_status(400))
            }
            Some(v) => v,
        };
        Ok(Response::from_json(&product)?)
    }
}
impl Company {}

#[durable_object]
impl DurableObject for Company {
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }
    async fn fetch(&mut self, req: Request) -> Result<Response> {
        tracing::debug!("running durable object: {}", self.state.id().to_string());
        Router::with_data(RequestContext::new(self.state.storage()))
            .post_async("/products", |mut req, mut ctx| async move {
                let RegisterProductRequest {
                    product_name,
                    product_data,
                } = req.json().await?;
                ctx.data
                    .register_product(&product_name, &product_data)
                    .await?;
                Response::empty()
            })
            .post_async("/transactions", |mut req, ctx| async move {
                let StartTransactionRequestData {
                    user_id,
                    product_name,
                } = req.json().await?;
                let claim = ctx
                    .data
                    .start_transaction(&user_id, &product_name)
                    .await
                    .unwrap();
                Ok(claim)
            })
            .post_async("/transactions/:txid/approved", |mut req, ctx| async move {
                let txid = ctx.param("txid");
                if let Some(_txid) = txid {
                    let TransactionApprovedRequestData { product_name } = req.json().await?;
                    ctx.data.transaction_approved(&product_name).await.unwrap();
                };

                unreachable!();
            })
            .run(req, self.env.clone())
            .await
    }
}
