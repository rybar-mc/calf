mod middleware;
mod models;
mod utils;

use bytes::{BufMut, BytesMut};
use models::*;
use utils::json_error;
use utoipa::OpenApi;
use worker::*;

#[derive(OpenApi)]
#[openapi(
    paths(get_player),
    components(schemas(CalfResponse, TextureData, ErrorResponse))
)]
pub struct ApiDoc;

/// Fetches the uuid for a given minecraft username.
/// Returns `Ok(Ok(uuid))` on success, or `Ok(Err(Response))` if an error occurs.
async fn fetch_mojang_uuid(username: &str) -> Result<std::result::Result<String, Response>> {
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
async fn fetch_mojang_profile(
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

fn build_raw_response(
    uuid: Option<&str>,
    username: Option<&str>,
    textures: &TextureData,
) -> Result<Response> {
    let mut buffer = BytesMut::new();

    let write_str = |buf: &mut BytesMut, s: &str| {
        buf.put_u16(s.len() as u16);
        buf.put_slice(s.as_bytes());
    };

    if let Some(u) = uuid {
        write_str(&mut buffer, u);
    }

    if let Some(name) = username {
        write_str(&mut buffer, name);
    }

    write_str(&mut buffer, &textures.value);
    write_str(&mut buffer, &textures.signature);

    let mut res = Response::from_bytes(buffer.to_vec())?;
    res.headers_mut()
        .set("Content-Type", "application/octet-stream")?;
    Ok(res)
}

fn build_json_response(
    uuid: Option<String>,
    username: Option<String>,
    textures: TextureData,
) -> Result<Response> {
    Response::from_json(&CalfResponse {
        uuid,
        username,
        textures,
    })
}

#[utoipa::path(
    get,
    path = "/v1/{username}",
    responses(
        (status = 200, description = "Player textures (JSON or Raw Binary)"),
        (status = 400, description = "Username is required", body = ErrorResponse),
        (status = 404, description = "Player not found", body = ErrorResponse),
        (status = 502, description = "Mojang API error", body = ErrorResponse),
    ),
    params(
        ("username" = String, Path, description = "Minecraft username"),
        ("format" = Option<String>, Query, description = "Response format: 'json' (default) or 'raw'"),
        ("exclude" = Option<String>, Query, description = "Comma-separated fields to exclude, e.g. 'uuid,username'")
    )
)]
async fn get_player(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let username = match ctx.param("username") {
        Some(u) => u,
        None => return json_error("Username is required", 400),
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

    let uuid = match fetch_mojang_uuid(username).await? {
        Ok(id) => id,
        Err(error_response) => return Ok(error_response),
    };

    let profile = match fetch_mojang_profile(&uuid).await? {
        Ok(p) => p,
        Err(error_response) => return Ok(error_response),
    };

    let textures_prop = profile
        .properties
        .into_iter()
        .find(|p| p.name == "textures");
    let tex = match textures_prop {
        Some(t) => t,
        None => return json_error("texture data missing on profile", 404),
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
        build_raw_response(
            opt_uuid.as_deref(),
            opt_username.as_deref(),
            &clean_textures,
        )?
    } else {
        build_json_response(opt_uuid, opt_username, clean_textures)?
    };

    response
        .headers_mut()
        .set("Cache-Control", "public, max-age=300")?;

    Ok(response)
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    if let Some(auth_rejection) = middleware::check_auth(&req, &env)? {
        return Ok(auth_rejection);
    }

    let router = Router::new();

    router
        .get_async("/v1/:username", get_player)
        .run(req, env)
        .await
}
