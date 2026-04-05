use crate::utils::json_error;
use worker::*;

pub fn check_auth(req: &Request, env: &Env) -> Result<Option<Response>> {
    let auth_header = req.headers().get("Authorization")?.unwrap_or_default();

    let expected_key = match env.secret("AUTH_KEY") {
        Ok(k) => k.to_string(),
        Err(_) => return Ok(Some(json_error("missing auth key in secrets", 500)?)),
    };

    if auth_header == format!("Bearer {}", expected_key) {
        Ok(None)
    } else {
        Ok(Some(json_error("invalid api key", 401)?))
    }
}
