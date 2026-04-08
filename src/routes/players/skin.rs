use worker::*;

use crate::models::ErrorResponse;
use crate::utils::json_error;

use super::utils::{fetch_image_bytes, get_decoded_textures, resolve_uuid};

#[utoipa::path(
    get,
    path = "/v1/players/{identifier}/skin",
    responses(
        (status = 200, description = "Player skin in png"),
        (status = 400, description = "Identifier is required", body = ErrorResponse),
        (status = 404, description = "Player not found or no skin", body = ErrorResponse),
        (status = 502, description = "Mojang API error", body = ErrorResponse),
    ),
    params(
        ("identifier" = String, Path, description = "Minecraft username or UUID")
    )
)]
pub async fn get_player_skin(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let identifier = match ctx.param("identifier") {
        Some(i) => i,
        None => return json_error("Identifier is required", 400),
    };

    let uuid = match resolve_uuid(identifier).await? {
        Ok(id) => id,
        Err(error_response) => return Ok(error_response),
    };

    let tex_json = match get_decoded_textures(&uuid).await? {
        Ok(t) => t,
        Err(error_response) => return Ok(error_response),
    };

    let skin_url = match tex_json.textures.skin {
        Some(s) => s.url,
        None => return json_error("player has no skin", 404),
    };

    let skin_bytes = match fetch_image_bytes(&skin_url, "failed to fetch skin from mojang").await? {
        Ok(b) => b,
        Err(e) => return Ok(e),
    };

    let mut response = Response::from_bytes(skin_bytes)?;
    response.headers_mut().set("Content-Type", "image/png")?;
    response
        .headers_mut()
        .set("Cache-Control", "public, max-age=300")?;

    Ok(response)
}
