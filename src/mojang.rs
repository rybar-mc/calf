use crate::models::{MojangSessionResponse, MojangUuidResponse};
use crate::utils::json_error;
use worker::*;

/// Fetches the uuid for a given minecraft username.
/// Returns `Ok(Ok(uuid))` on success, or `Ok(Err(Response))` if an error occurs.
pub async fn fetch_mojang_uuid(username: &str) -> Result<std::result::Result<String, Response>> {
    let url = format!(
        "https://api.mojang.com/users/profiles/minecraft/{}",
        username
    );
    let mut res = Fetch::Url(url.parse()?).send().await?;

    if res.status_code() == 204 || res.status_code() == 404 {
        return Ok(Err(json_error("player not found", 404)?));
    }

    match res.json::<MojangUuidResponse>().await {
        Ok(data) => Ok(Ok(data.id)),
        Err(_) => Ok(Err(json_error(
            "failed to parse mojang uuid response",
            502,
        )?)),
    }
}

/// Fetches the profile data (including textures) for a given uuid.
pub async fn fetch_mojang_profile(
    uuid: &str,
) -> Result<std::result::Result<MojangSessionResponse, Response>> {
    let url = format!(
        "https://sessionserver.mojang.com/session/minecraft/profile/{}?unsigned=false",
        uuid
    );
    let mut res = Fetch::Url(url.parse()?).send().await?;

    if res.status_code() != 200 {
        return Ok(Err(json_error("failed to fetch profile from mojang", 502)?));
    }

    match res.json::<MojangSessionResponse>().await {
        Ok(data) => Ok(Ok(data)),
        Err(_) => Ok(Err(json_error("failed to parse mojang profile data", 502)?)),
    }
}
