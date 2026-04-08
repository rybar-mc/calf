mod middleware;
mod models;
mod mojang;
mod routes;
mod utils;

use models::*;
use utoipa::OpenApi;
use worker::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::players::profile::get_player_profile,
        routes::players::head::get_player_head,
        routes::players::skin::get_player_skin,
        routes::players::cape::get_player_cape,
        routes::players::batch::get_player_batch
    ),
    components(schemas(CalfResponse, TextureData, ErrorResponse))
)]
pub struct ApiDoc;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    if let Some(auth_rejection) = middleware::check_auth(&req, &env)? {
        return Ok(auth_rejection);
    }

    let router = Router::new();
    let router = routes::register_routes(router);

    router.run(req, env).await
}
