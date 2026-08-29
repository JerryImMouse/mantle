# Mantle

Mantle is an identity/authorization service that links a user's account on your service to their Discord account, and gives you cached access to their Discord data afterward.

You bring your own user IDs (from your game server, backend, whatever) - Mantle doesn't need to know anything about your service beforehand. It just needs an ID to link against.

## How it works

1.  **Check** - send your user's ID to `/api/auth/check`. If they haven't linked Discord yet, Mantle responds with `discord_required` and creates a Mantle identity for that ID under the hood.
2.  **Link** - call `/api/auth/link` with the same ID to get an OAuth authorization URL. Hand that URL to the user.
3.  **Callback** - the user authorizes with Discord and is redirected to `/api/auth/callback`, where Mantle links the Discord identity to the identity from your service.
4.  **Use** - from then on, query endpoints like `/api/discord/user` with your service's user ID to get their Discord data.

```
Your service                         Mantle                          Discord
     |                                  |                                |
     |--- POST /api/auth/check -------->|                                |
     |<-- discord_required -------------|                                |
     |                                  |                                |
     |--- POST /api/auth/link --------->|                                |
     |<-- authorization URL ------------|                                |
     |                                  |                                |
     |     (user opens URL) -------------------------------------------->|
     |                                  |<--- OAuth callback ------------|
     |                                  | (identities linked)            |
     |                                  |                                |
     |--- GET /api/discord/user ------->|                                |
     |<-- cached Discord data ----------|                                |
```

## Identity providers

Identities are keyed by a provider + external ID pair. Out of the box this includes `External` and `Discord`. Any service with its own user IDs works the same way.

## Per-user metadata

Each identity can hold arbitrary metadata as key -> value (JSON) pairs, for whatever your service needs to attach to a user (roles, flags, preferences, etc.).

## Caching

Discord API responses (`@me`, `guilds`, `guild_member`, etc.) are cached to avoid hitting Discord's rate limits. Access and refresh tokens are refreshed automatically — you never need to handle the OAuth token lifecycle yourself.

## API

Routes are documented via [utoipa](https://github.com/juhaku/utoipa). Generate an `openapi.json` spec with:

```
cargo run --feature openapi --bin generate-openapi

```

Open the resulting file in any OpenAPI viewer (Swagger UI, Redoc, etc.) to browse the full API.

## Setup

Mantle is configured via a required `config.toml` in the project root — every field must be present. Any field can be overridden via environment variables (e.g. for secrets you don't want committed).

```toml
[database]
url = "postgres://user:password@localhost/mantle"

[server]
host = "127.0.0.1"
port = 5050
api_secret = "..."

[discord]
client_id = "..."
client_secret = "..."
redirect_uri = "https://your-domain/api/auth/callback"
state_secret = "..."

```

| Env var | Overrides |
|---|---|
| `APP_DATABASE_URL` | `database.url` |
| `APP_HOST` | `server.host` |
| `APP_PORT` | `server.port` |
| `APP_API_SECRET` | `server.api_secret` |
| `APP_DISCORD_CLIENT_ID` | `discord.client_id` |
| `APP_DISCORD_CLIENT_SECRET` | `discord.client_secret` |
| `APP_DISCORD_REDIRECT_URI` | `discord.redirect_uri` |
| `APP_DISCORD_STATE_SECRET` | `discord.state_secret` |

Once `config.toml` is in place, it's a regular Rust project:

```
cargo run --bin mantle
```

Build, deploy, and release builds work the same as any other Cargo project — no extra steps beyond the usual `--release`, aside from the other `--bin` targets in the workspace (e.g. `generate-openapi`, mentioned above)

## License

MIT 2026 JerryImMouse - see LICENSE.TXT
