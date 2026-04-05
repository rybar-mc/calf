use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct CalfResponse {
    pub uuid: String,
    pub username: String,
    pub textures: TextureData,
}

#[derive(Serialize, ToSchema)]
pub struct TextureData {
    pub value: String,
    pub signature: Option<String>,
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
    pub id: String,
    pub name: String,
    pub properties: Vec<ProfileProperty>,
}

#[derive(Deserialize)]
pub struct ProfileProperty {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}
