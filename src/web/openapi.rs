use serde::Serialize;
use utoipa::{
    Modify, OpenApi, ToSchema,
    openapi::security::{Http, SecurityScheme},
};

use crate::db::identities::IdentityProvider;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "BearerAuth",
                SecurityScheme::Http(
                    Http::builder()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .description(Some("API token"))
                        .bearer_format("API")
                        .build(),
                ),
            );
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(OpenApi)]
#[openapi(
    info(
        contact(
            name = "JerryImMouse",
            email = "jerryimmouse.dev@gmail.com",
            url = "https://github.com/JerryImMouse",
        ),
    ),
    modifiers(&SecurityAddon),
    security(
        ("BearerAuth" = [])
    ),
    servers(
        (
            url = "http://localhost:{port}",
            variables(
                ("port" = (default = "5050", description = "Default port of mantle"))
            )
        ),
    ),
    tags(
        (name = "auth", description = "Authorization related routes"),
        (name = "discord", description = "Discord related routes"),
        (name = "health", description = "Service health related routes"),
        (name = "account", description = "Account related routes"),
        (name = "metadata", description = "Metadata related routes"),
    ),
    components(schemas(ErrorResponse, IdentityProvider))
)]
pub struct ApiDoc;
