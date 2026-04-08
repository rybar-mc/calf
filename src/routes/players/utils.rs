use base64::prelude::*;
use bytes::{BufMut, BytesMut};
use worker::*;

use crate::models::{self, CalfResponse, TextureData};
use crate::mojang;
use crate::utils::json_error;

pub async fn resolve_uuid(identifier: &str) -> Result<std::result::Result<String, Response>> {
    if identifier.len() > 16 {
        Ok(Ok(identifier.replace("-", "")))
    } else {
        mojang::fetch_mojang_uuid(identifier).await
    }
}

pub async fn fetch_profile_and_textures(
    uuid: &str,
) -> Result<std::result::Result<(models::MojangSessionResponse, models::ProfileProperty), Response>>
{
    let mut profile = match mojang::fetch_mojang_profile(uuid).await? {
        Ok(p) => p,
        Err(e) => return Ok(Err(e)),
    };

    let tex_idx = profile.properties.iter().position(|p| p.name == "textures");
    let tex = match tex_idx {
        Some(idx) => profile.properties.remove(idx),
        None => return Ok(Err(json_error("texture data missing on profile", 404)?)),
    };

    Ok(Ok((profile, tex)))
}

pub async fn get_decoded_textures(
    uuid: &str,
) -> Result<std::result::Result<models::TexturesDecoded, Response>> {
    let (_, tex) = match fetch_profile_and_textures(uuid).await? {
        Ok(res) => res,
        Err(e) => return Ok(Err(e)),
    };

    decode_textures_b64(&tex.value)
}

pub fn decode_textures_b64(
    base64_value: &str,
) -> Result<std::result::Result<models::TexturesDecoded, Response>> {
    let decoded = match BASE64_STANDARD.decode(base64_value) {
        Ok(d) => d,
        Err(_) => return Ok(Err(json_error("failed to decode textures base64", 502)?)),
    };

    let tex_json: models::TexturesDecoded = match serde_json::from_slice(&decoded) {
        Ok(j) => j,
        Err(_) => return Ok(Err(json_error("failed to parse textures json", 502)?)),
    };

    Ok(Ok(tex_json))
}

pub async fn fetch_image_bytes(
    url: &str,
    error_msg: &str,
) -> Result<std::result::Result<Vec<u8>, Response>> {
    let mut res = Fetch::Url(url.parse()?).send().await?;
    if res.status_code() != 200 {
        return Ok(Err(json_error(error_msg, 502)?));
    }
    Ok(Ok(res.bytes().await?))
}

pub fn extract_head_png(skin_bytes: &[u8]) -> Result<std::result::Result<Vec<u8>, Response>> {
    let img = match image::load_from_memory_with_format(skin_bytes, image::ImageFormat::Png) {
        Ok(i) => i,
        Err(_) => return Ok(Err(json_error("failed to parse skin image", 502)?)),
    };

    let mut head = img.crop_imm(8, 8, 8, 8);
    let hat = img.crop_imm(40, 8, 8, 8);

    image::imageops::overlay(&mut head, &hat, 0, 0);

    let mut out_bytes = std::io::Cursor::new(Vec::new());
    if head
        .write_to(&mut out_bytes, image::ImageFormat::Png)
        .is_err()
    {
        return Ok(Err(json_error("failed to encode head image", 500)?));
    }

    Ok(Ok(out_bytes.into_inner()))
}

pub fn extract_head_rgb(skin_bytes: &[u8]) -> Result<std::result::Result<Vec<u8>, Response>> {
    let img = match image::load_from_memory_with_format(skin_bytes, image::ImageFormat::Png) {
        Ok(i) => i,
        Err(_) => return Ok(Err(json_error("failed to parse skin image", 502)?)),
    };

    let mut head = img.crop_imm(8, 8, 8, 8);
    let hat = img.crop_imm(40, 8, 8, 8);

    image::imageops::overlay(&mut head, &hat, 0, 0);

    Ok(Ok(head.into_rgb8().into_raw()))
}

pub fn extract_image_rgb(image_bytes: &[u8]) -> Result<std::result::Result<Vec<u8>, Response>> {
    let img = match image::load_from_memory_with_format(image_bytes, image::ImageFormat::Png) {
        Ok(i) => i,
        Err(_) => return Ok(Err(json_error("failed to parse image", 502)?)),
    };

    Ok(Ok(img.into_rgb8().into_raw()))
}

/// A builder to construct raw binary responses with length-prefixed parts.
pub struct BatchBuilder {
    buffer: BytesMut,
}

impl BatchBuilder {
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::new(),
        }
    }

    pub fn add_string(mut self, s: &str) -> Self {
        self.buffer.put_u16(s.len() as u16);
        self.buffer.put_slice(s.as_bytes());
        self
    }

    pub fn add_bytes(mut self, b: &[u8]) -> Self {
        self.buffer.put_u16(b.len() as u16);
        self.buffer.put_slice(b);
        self
    }

    pub fn build(self) -> Result<Response> {
        let mut res = Response::from_bytes(self.buffer.to_vec())?;
        res.headers_mut()
            .set("Content-Type", "application/octet-stream")?;
        Ok(res)
    }
}

pub fn build_json_response(
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
