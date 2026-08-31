use serde::Deserialize;

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Deserialize)]
pub struct MetadataRequestBody {
    pub value: serde_json::Value,
    pub private: Option<bool>,
}
