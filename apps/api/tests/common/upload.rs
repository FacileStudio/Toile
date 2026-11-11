use axum::body::Body;
use http::{Request, header};

use super::http::{Answer, raw};
use super::{App, Person};

pub const PNG: &[u8] = &[
    0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 13, b'I', b'H', b'D', b'R', 0, 0, 0, 3, 0, 0, 0, 2, 8, 6,
    0, 0, 0,
];

pub fn png(padding: usize) -> Vec<u8> {
    let mut bytes = PNG.to_vec();
    bytes.resize(PNG.len() + padding, 0);
    bytes
}

pub async fn upload(app: &App, path: &str, who: Option<&Person>, bytes: &[u8]) -> Answer {
    let mut body = b"--XyZ\r\nContent-Disposition: form-data; name=\"file\"; filename=\"a.png\"\r\nContent-Type: image/png\r\n\r\n".to_vec();
    body.extend_from_slice(bytes);
    body.extend_from_slice(b"\r\n--XyZ--\r\n");
    let mut request = Request::post(path)
        .header(header::HOST, "toile.test")
        .header(header::CONTENT_TYPE, "multipart/form-data; boundary=XyZ")
        .header("X-Facile-CSRF", "1");
    if let Some(who) = who {
        request = request.header(header::AUTHORIZATION, format!("Bearer {}", who.token));
    }
    raw(&app.router, request.body(Body::from(body)).unwrap()).await
}

pub async fn get(app: &App, path: &str, who: Option<&Person>, etag: Option<&str>) -> Answer {
    let mut request = Request::get(path).header(header::HOST, "toile.test");
    if let Some(who) = who {
        request = request.header(header::AUTHORIZATION, format!("Bearer {}", who.token));
    }
    if let Some(etag) = etag {
        request = request.header(header::IF_NONE_MATCH, etag);
    }
    raw(&app.router, request.body(Body::empty()).unwrap()).await
}
