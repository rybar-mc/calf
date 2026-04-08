use worker::*;

use crate::models::{ErrorResponse, TextureData};
use crate::utils::json_error;

use super::utils::{build_json_response, fetch_profile_and_textures, resolve_uuid, BatchBuilder};

#[utoipa::path(
    get,
    path = "/v1/players/{identifier}",
    responses(
        (status = 200, description = "Player textures (json or raw binary)"),
        (status = 400, description = "Identifier is required", body = ErrorResponse),
        (status = 404, description = "Player not found", body = ErrorResponse),
        (status = 502, description = "Mojang API error", body = ErrorResponse),
    ),
    params(
        ("identifier" = String, Path, description = "Minecraft username or UUID"),
        ("format" = Option<String>, Query, description = "Response format: 'json' (default) or 'raw'"),
        ("exclude" = Option<String>, Query, description = "Comma-separated fields to exclude, e.g. 'uuid,username'")
    )
)]
pub async fn get_player_profile(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let identifier = match ctx.param("identifier") {
        Some(i) => i,
        None => return json_error("Identifier is required", 400),
    };

    let req_url = req.url()?;

    let is_raw = req_url
        .query_pairs()
        .any(|(k, v)| k == "format" && v == "raw");

    let exclude_param = req_url
        .query_pairs()
        .find(|(k, _)| k == "exclude")
        .map(|(_, v)| v.to_string())
        .unwrap_or_default();

    let exclude_uuid = exclude_param.contains("uuid");
    let exclude_username = exclude_param.contains("username");

    let uuid = match resolve_uuid(identifier).await? {
        Ok(id) => id,
        Err(error_response) => return Ok(error_response),
    };

    let (profile, tex) = match fetch_profile_and_textures(&uuid).await? {
        Ok(res) => res,
        Err(error_response) => return Ok(error_response),
    };

    let clean_textures = TextureData {
        value: tex.value,
        signature: tex.signature,
    };

    let opt_uuid = if exclude_uuid {
        None
    } else {
        Some(uuid.clone())
    };
    let opt_username = if exclude_username {
        None
    } else {
        Some(profile.name.clone())
    };

    let mut response = if is_raw {
        let mut builder = BatchBuilder::new();
        if let Some(u) = &opt_uuid {
            builder = builder.add_string(u);
        }
        if let Some(name) = &opt_username {
            builder = builder.add_string(name);
        }
        builder = builder.add_string(&clean_textures.value);
        builder = builder.add_string(&clean_textures.signature);
        builder.build()?
    } else {
        build_json_response(opt_uuid, opt_username, clean_textures)?
    };

    response
        .headers_mut()
        .set("Cache-Control", "public, max-age=300")?;

    Ok(response)
}
