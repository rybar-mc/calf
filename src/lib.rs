mod middleware;
mod models;
mod utils;

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

#[utoipa::path(
    get,
    path = "/v1/{username}",
    responses(
        (status = 200, description = "Player textures", body = CalfResponse),
        (status = 400, description = "Username is required", body = ErrorResponse),
        (status = 404, description = "Player not found", body = ErrorResponse),
        (status = 502, description = "Mojang API error", body = ErrorResponse),
    ),
    params(
        ("username" = String, Path, description = "Minecraft username")
    )
)]
async fn get_player(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let username = match ctx.param("username") {
        Some(u) => u,
        None => return json_error("Username is required", 400),
    };

    let name_url = format!(
        "https://api.mojang.com/users/profiles/minecraft/{}",
        username
    );
    let mut name_res = Fetch::Url(name_url.parse()?).send().await?;

    if name_res.status_code() == 204 || name_res.status_code() == 404 {
        return json_error("player not found", 404);
    }

    let name_data = match name_res.json::<MojangUuidResponse>().await {
        Ok(data) => data,
        Err(_) => return json_error("failed to parse mojang uuid response", 502),
    };
    let uuid = name_data.id;

    let session_url = format!(
        "https://sessionserver.mojang.com/session/minecraft/profile/{}?unsigned=false",
        uuid
    );

    let mut session_res = Fetch::Url(session_url.parse()?).send().await?;
    if session_res.status_code() != 200 {
        return json_error("failed to fetch profile from mojang", 502);
    }

    let profile = match session_res.json::<MojangSessionResponse>().await {
        Ok(data) => data,
        Err(_) => return json_error("failed to parse mojang profile data", 502),
    };

    let textures_prop = profile
        .properties
        .into_iter()
        .find(|p| p.name == "textures");

    if let Some(tex) = textures_prop {
        let clean_textures = TextureData {
            value: tex.value,
            signature: tex.signature,
        };

        let mut response = Response::from_json(&CalfResponse {
            uuid,
            username: profile.name,
            textures: clean_textures,
        })?;

        response
            .headers_mut()
            .set("Cache-Control", "public, max-age=300")?;

        Ok(response)
    } else {
        json_error("texture data missing on profile", 404)
    }
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
