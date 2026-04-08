use worker::*;

use crate::models::ErrorResponse;
use crate::utils::json_error;

use super::utils::{
    decode_textures_b64, extract_head_rgb, extract_image_rgb, fetch_image_bytes,
    fetch_profile_and_textures, resolve_uuid, BatchBuilder,
};

#[utoipa::path(
    get,
    path = "/v1/players/{identifier}/batch",
    responses(
        (status = 200, description = "Dynamically batched player data (profile, head, skin, cape)"),
        (status = 400, description = "Identifier is required", body = ErrorResponse),
        (status = 404, description = "Player not found or missing requested data", body = ErrorResponse),
        (status = 502, description = "Mojang API error", body = ErrorResponse),
    ),
    params(
        ("identifier" = String, Path, description = "Minecraft username or UUID"),
        ("parts" = Option<String>, Query, description = "Comma-separated parts to include: profile, texture, head, skin, cape. Default: profile,head")
    )
)]
pub async fn get_player_batch(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let identifier = match ctx.param("identifier") {
        Some(i) => i,
        None => return json_error("Identifier is required", 400),
    };

    let req_url = req.url()?;
    let parts_param = req_url
        .query_pairs()
        .find(|(k, _)| k == "parts")
        .map(|(_, v)| v.to_string())
        .unwrap_or_else(|| "profile,head".to_string());

    let parts: Vec<&str> = parts_param.split(',').map(|s| s.trim()).collect();

    let uuid = match resolve_uuid(identifier).await? {
        Ok(id) => id,
        Err(error_response) => return Ok(error_response),
    };

    let (profile, tex) = match fetch_profile_and_textures(&uuid).await? {
        Ok(res) => res,
        Err(error_response) => return Ok(error_response),
    };

    let needs_images =
        parts.contains(&"head") || parts.contains(&"skin") || parts.contains(&"cape");
    let tex_json = if needs_images {
        match decode_textures_b64(&tex.value)? {
            Ok(t) => Some(t),
            Err(error_response) => return Ok(error_response),
        }
    } else {
        None
    };

    let mut builder = BatchBuilder::new();
    let mut cached_skin_bytes: Option<Vec<u8>> = None;

    for part in parts {
        match part {
            "profile" => {
                builder = builder
                    .add_string(&uuid)
                    .add_string(&profile.name)
                    .add_string(&tex.value)
                    .add_string(&tex.signature);
            }
            "texture" => {
                builder = builder.add_string(&tex.value).add_string(&tex.signature);
            }
            "head" => {
                let textures = tex_json.as_ref().unwrap();
                let skin_url = match &textures.textures.skin {
                    Some(s) => &s.url,
                    None => return json_error("player has no skin", 404),
                };

                let skin_bytes = if let Some(bytes) = &cached_skin_bytes {
                    bytes.clone()
                } else {
                    let bytes =
                        match fetch_image_bytes(skin_url, "failed to fetch skin from mojang")
                            .await?
                        {
                            Ok(b) => b,
                            Err(e) => return Ok(e),
                        };
                    cached_skin_bytes = Some(bytes.clone());
                    bytes
                };

                let head_rgb = match extract_head_rgb(&skin_bytes)? {
                    Ok(h) => h,
                    Err(e) => return Ok(e),
                };
                builder = builder.add_bytes(&head_rgb);
            }
            "skin" => {
                let textures = tex_json.as_ref().unwrap();
                let skin_url = match &textures.textures.skin {
                    Some(s) => &s.url,
                    None => return json_error("player has no skin", 404),
                };

                let skin_bytes = if let Some(bytes) = &cached_skin_bytes {
                    bytes.clone()
                } else {
                    let bytes =
                        match fetch_image_bytes(skin_url, "failed to fetch skin from mojang")
                            .await?
                        {
                            Ok(b) => b,
                            Err(e) => return Ok(e),
                        };
                    cached_skin_bytes = Some(bytes.clone());
                    bytes
                };

                let skin_rgb = match extract_image_rgb(&skin_bytes)? {
                    Ok(s) => s,
                    Err(e) => return Ok(e),
                };
                builder = builder.add_bytes(&skin_rgb);
            }
            "cape" => {
                let textures = tex_json.as_ref().unwrap();
                let cape_url = match &textures.textures.cape {
                    Some(c) => &c.url,
                    None => return json_error("player has no cape", 404),
                };

                let cape_bytes =
                    match fetch_image_bytes(cape_url, "failed to fetch cape from mojang").await? {
                        Ok(b) => b,
                        Err(e) => return Ok(e),
                    };
                let cape_rgb = match extract_image_rgb(&cape_bytes)? {
                    Ok(c) => c,
                    Err(e) => return Ok(e),
                };
                builder = builder.add_bytes(&cape_rgb);
            }
            _ => {
                // ignore
            }
        }
    }

    let mut res = builder.build()?;
    res.headers_mut()
        .set("Cache-Control", "public, max-age=300")?;

    Ok(res)
}
