use anyhow::Context;
use serverless_discord_interactions::twilight_model::{
    application::interaction::{
        application_command::{CommandData, CommandDataOption, CommandOptionValue},
        Interaction,
    },
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
};
use tracing::instrument;
use worker::Env;

use crate::company::{CompanyClient, Product, Role};
#[instrument(skip(env))]
pub(crate) async fn handle_product(
    intr: &Interaction,
    data: &Box<CommandData>,
    env: &Env,
) -> anyhow::Result<InteractionResponse> {
    let group = &data.options[0];
    match group.name.as_str() {
        "create" => handle_create_product(intr, data, group, env).await,
        _ => unreachable!(),
    }
}
#[instrument(skip(env))]
pub(crate) async fn handle_create_product(
    intr: &Interaction,
    data: &Box<CommandData>,
    group: &CommandDataOption,
    env: &Env,
) -> anyhow::Result<InteractionResponse> {
    match &group.value {
        CommandOptionValue::SubCommandGroup(group_data) => {
            let subcommand = &group_data[0];
            match subcommand.name.as_str() {
                "role" => handle_create_role_product(intr, data, subcommand, env).await,
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}
#[instrument(skip(env))]
pub(crate) async fn handle_create_role_product(
    intr: &Interaction,
    data: &Box<CommandData>,
    subcommand: &CommandDataOption,
    env: &Env,
) -> anyhow::Result<InteractionResponse> {
    match &subcommand.value {
        CommandOptionValue::SubCommand(options) => {
            let mut name: Option<String> = None;
            let mut price: Option<String> = None;
            let mut unit: Option<String> = None;
            let mut role_id: Option<String> = None;

            for option in options {
                match option.name.as_str() {
                    "name" => {
                        if let CommandOptionValue::String(v) = &option.value {
                            name = Some(v.to_owned());
                        } else {
                            unreachable!()
                        }
                    }
                    "price" => {
                        if let CommandOptionValue::Integer(v) = option.value {
                            price = Some(v.to_string());
                        } else {
                            unreachable!()
                        }
                    }
                    "unit" => {
                        if let CommandOptionValue::String(v) = &option.value {
                            unit = Some(v.to_owned());
                        } else {
                            unreachable!()
                        }
                    }
                    "product" => {
                        if let CommandOptionValue::Role(v) = &option.value {
                            role_id = Some(v.to_string());
                        } else {
                            unreachable!()
                        }
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
            process_create_role_product(
                intr.guild_id.context("invalid context")?.to_string(),
                name.context("missing name")?,
                price.context("missing price")?,
                unit.context("missing unit")?,
                role_id.context("missing role_id")?,
                env,
            )
            .await
        }
        _ => unreachable!(),
    }
}
#[instrument(skip(env))]
pub(crate) async fn process_create_role_product(
    guild_id: String,
    product_name: String,
    price: String,
    unit: String,
    role_id: String,
    env: &Env,
) -> anyhow::Result<InteractionResponse> {
    let client = CompanyClient::new(env.clone(), guild_id.clone());
    let content = format!("{}{}: {}", price, unit, role_id);
    tracing::debug!("aaaa");
    client
        .register_product(
            product_name,
            Product::Role(Role {
                unit,
                price,
                role_id,
            }),
        )
        .await?;

    Ok(InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(InteractionResponseData {
            content: Some(content),
            ..Default::default()
        }),
    })
}
