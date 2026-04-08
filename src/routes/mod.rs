pub mod players;

use worker::Router;

pub fn register_routes<'a>(router: Router<'a, ()>) -> Router<'a, ()> {
    router
        .get_async("/v1/players/:identifier", players::get_player_profile)
        .get_async("/v1/players/:identifier/head", players::get_player_head)
        .get_async("/v1/players/:identifier/skin", players::get_player_skin)
        .get_async("/v1/players/:identifier/cape", players::get_player_cape)
        .get_async("/v1/players/:identifier/batch", players::get_player_batch)
}
