use serde::Serialize;

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Serialize)]
pub struct HealthResponseBody {
    pub status: String,
    pub bind_to: String,
}
