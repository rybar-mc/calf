use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct CalfResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    pub textures: TextureData,
}

#[derive(Serialize, ToSchema)]
pub struct TextureData {
    pub value: String,
    pub signature: String,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub status: u16,
}

#[derive(Deserialize)]
pub struct MojangUuidResponse {
    pub id: String,
}

#[derive(Deserialize)]
pub struct MojangSessionResponse {
    #[allow(dead_code)]
    pub id: String,
    pub name: String,
    pub properties: Vec<ProfileProperty>,
}

#[derive(Deserialize)]
pub struct ProfileProperty {
    pub name: String,
    pub value: String,
    pub signature: String,
}

#[derive(Deserialize)]
pub struct TexturesDecoded {
    pub textures: TexturesMap,
}

#[derive(Deserialize)]
pub struct TexturesMap {
    #[serde(rename = "SKIN")]
    pub skin: Option<SkinInfo>,
    #[serde(rename = "CAPE")]
    pub cape: Option<SkinInfo>,
}

#[derive(Deserialize)]
pub struct SkinInfo {
    pub url: String,
}
