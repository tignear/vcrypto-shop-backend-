import { REST, } from "@discordjs/rest";
import { SlashCommandBuilder } from "@discordjs/builders";
import { Routes } from "discord-api-types/v10";
const appId = process.env.DISCORD_APPLICATION_ID;
const token = process.env.DISCORD_BOT_TOKEN;
const rest = new REST({}).setToken(token);


const commands = [
  new SlashCommandBuilder().setName("shop")
    .setDescription("ショップ関連")
    .addSubcommand(
      cmd => cmd.setName("place").setDescription("設置").addStringOption(option => option.setName("product_id").setDescription("商品のid").setRequired(true))
    )
  ,
  new SlashCommandBuilder().setName("product")
    .setDescription("商品関連")
    .addSubcommandGroup(group => {
      return group.setName("create").setDescription("作成").addSubcommand(cmd =>
        cmd
          .setName("role")
          .setDescription("ロール")
          .addStringOption(option => option.setName("name").setRequired(true).setDescription("商品名"))

          .addIntegerOption(option =>
            option
              .setName("price")
              .setRequired(true)
              .setDescription("商品の価格")
          )
          .addStringOption(option => option.setName("unit").setRequired(true).setDescription("支払いを受け付ける通貨単位"))
          .addRoleOption(option => option.setName("product").setRequired(true).setDescription("付与するロール"))
      )
    })
];

const res = await rest.put(Routes.applicationCommands(appId), {
  body: commands
});
console.log(res);