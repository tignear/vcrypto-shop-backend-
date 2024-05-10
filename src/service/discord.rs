mod product;
mod shop;

use anyhow::{bail, Context as _};
use serverless_discord_interactions::{
    twilight_model::{
        application::interaction::{Interaction, InteractionData},
        channel::message::MessageFlags,
        http::interaction::{
            InteractionResponse, InteractionResponseData, InteractionResponseType,
        },
    },
    InteractionHandler,
};
use worker::*;

use self::{product::handle_product, shop::{buy_product, place_shop}};

pub(crate) struct Handler {
    pub env: Env,
    pub ctx: Context,
}
impl InteractionHandler for Handler {
    
    async fn on_interaction(self, interaction: Interaction) -> anyhow::Result<InteractionResponse> {
        let user_id = interaction
            .author_id()
            .context("invalid execution context")?
            .to_string();
        let data = interaction
            .data
            .as_ref()
            .context("invalid request: interaction data is missing")?;
        let res = match data {
            InteractionData::ApplicationCommand(data) => match data.name.as_str() {
                "product" => handle_product(&interaction, &data, &self.env).await,
                "shop" => place_shop(data).await,
                _ => {
                    bail!("not implemented");
                }
            },
            InteractionData::MessageComponent(data) => Ok({
                let custom_id = data.custom_id.clone();
                let splitted: Vec<&str> = custom_id.split("@").collect();
                match splitted[0] {
                    "/actions/buy-product" => {
                        buy_product(
                            &interaction,
                            interaction
                                .guild_id
                                .context("invalid execution context")?
                                .to_string(),
                            user_id,
                            splitted[1].to_string(),
                            self.ctx,
                            self.env.clone(),
                        )
                        .await
                    }
                    _ => InteractionResponse {
                        kind: InteractionResponseType::ChannelMessageWithSource,
                        data: Some(InteractionResponseData {
                            content: Some("unknown command!".to_string()),
                            flags: Some(MessageFlags::EPHEMERAL),
                            ..Default::default()
                        }),
                    },
                }
            }),
            _ => {
                bail!("not implemented");
            }
        };
        Ok(res?)
    }
}
