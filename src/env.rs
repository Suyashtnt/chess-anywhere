use menv::require_envs;

require_envs! {
    (assert_env_vars, any_set, gen_help);

    database_url, "DATABASE_URL", String,
    "Please set DATABASE_URL to a postgres db URL";

    discord_token, "DISCORD_TOKEN", String,
    "Please set DISCORD_TOKEN to a discord bot token";

    resend_api_key, "RESEND_API_KEY", String,
    "Please set RESEND_API_KEY to a resend.com api key";
}
