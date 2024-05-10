mod client;
mod company;
mod product;

pub(crate) use client::CompanyClient;
pub(crate) use client::StartTransactionResult;
pub(crate) use company::RegisterProductRequest;
pub(crate) use product::*;

use worker::{Env, Result};
const COMPANY_DURABLE_OBJECT_BINDING_KEY: &str = "COMPANY";


