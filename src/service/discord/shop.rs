use std::future::IntoFuture;

use crate::company::{self, CompanyClient, StartTransactionResult};
use anyhow::bail;
use serde::{Deserialize, Serialize};
use serverless_discord_interactions::rest::REST;
use serverless_discord_interactions::twilight_model::application::interaction::application_command::{CommandData, CommandOptionValue};
use serverless_discord_interactions::twilight_model::application::interaction::Interaction;
use serverless_discord_interactions::twilight_model::channel::message::component::{
    ActionRow, Button, ButtonStyle,
};
use serverless_discord_interactions::twilight_model::channel::message::Component;
use serverless_discord_interactions::twilight_model::http::interaction::InteractionResponseData;
use serverless_discord_interactions::RESTConfig;
use serverless_discord_interactions::{
    rest::{RequestData, RequestMethod},
    twilight_model::http::interaction::{InteractionResponse, InteractionResponseType},
};
use tracing::instrument;
use worker::{Context, Env};

pub(crate) async fn place_shop(data: &Box<CommandData>) -> anyhow::Result<InteractionResponse> {
    let product_name = match &data.options[0].value {
        CommandOptionValue::SubCommand(options) => match &options[0].value {
            CommandOptionValue::String(x) => x,
            _ => bail!("failed to get product name"),
        },
        _ => bail!("failed to get product_name"),
    };
    let res = InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(InteractionResponseData {
            components: Some(vec![Component::ActionRow(ActionRow {
                components: vec![Component::Button(Button {
                    custom_id: Some(format!("/actions/buy-product@{product_name}")),
                    label: Some(product_name.to_string()),
                    style: ButtonStyle::Primary,
                    disabled: false,
                    emoji: None,
                    url: None,
                })],
            })]),
            ..Default::default()
        }),
    };
    Ok(res)
}

#[instrument(skip(env, ctx))]
pub(crate) async fn buy_product(
    interaction: &Interaction,
    guild_id: String,
    user_id: String,
    param: String,
    ctx: Context,
    env: Env,
) -> InteractionResponse {
    let product_id = param;
    let id = interaction.id.to_string();
    let token = interaction.token.to_owned();
    ctx.wait_until(buy_product_continue_wrap(id, token, guild_id, product_id, user_id, env).into_future());

    InteractionResponse {
        kind: InteractionResponseType::DeferredChannelMessageWithSource,
        data: None,
    }
}

#[derive(Deserialize, Debug)]
struct InteractionWebhookResult;

#[derive(Serialize)]
struct InteractionWebhookBody {
    content: String,
}
#[instrument(skip(env))]
pub(crate) async fn buy_product_continue_wrap(
    interaction_id: String,
    interaction_token: String,
    guild_id: String,
    product_id: String,
    user_id: String,
    env: Env,
) -> () {
    match buy_product_continue(
        interaction_id,
        interaction_token,
        guild_id,
        product_id,
        user_id,
        env,
    )
    .await
    {
        Err(err) => {
            tracing::error!("{}", err);
        }
        Ok(_) => {
            tracing::debug!("ok");
        }
    }
}

#[instrument(skip(env))]
pub(crate) async fn buy_product_continue(
    interaction_id: String,
    interaction_token: String,
    guild_id: String,
    product_id: String,
    user_id: String,
    env: Env,
) -> anyhow::Result<()> {
    tracing::debug!("xxxxx");
    let token = env.var("DISCORD_BOT_TOKEN")?.to_string();
    let client = CompanyClient::new(env, guild_id);
    let mut rest = serverless_discord_interactions::rest_client(RESTConfig::new(&token));
    tracing::debug!("yyy");

    let start_transaction_result = client.start_transaction(product_id, user_id).await?;
    println!("xxxxx");
    tracing::debug!("xxxx: {:?}", start_transaction_result);
    match start_transaction_result {
        StartTransactionResult::Ok(claim) => {
            let res: InteractionWebhookResult = rest
                .request_json(
                    RequestMethod::POST,
                    format!(
                        "/interactions/{}/{}/callback",
                        interaction_id, interaction_token
                    ),
                    RequestData {
                        body: Some(InteractionWebhookBody {
                            content: format!("請求:{}を作成しました。", claim.id),
                        }),
                        ..Default::default()
                    },
                )
                .await?;
            tracing::error!("{:?}", res);
        }
        StartTransactionResult::InvalidRequest => {
            bail!("invalid request")
        }
    }
    Ok(())
}
