use menv::require_envs;

require_envs! {
    (assert_env_vars, any_set, gen_help);

    database_url, "DATABASE_URL", String,
    "Please set DATABASE_URL to a postgres db URL";

    discord_token, "DISCORD_TOKEN", String,
    "Please set DISCORD_TOKEN to a discord bot token";
}
