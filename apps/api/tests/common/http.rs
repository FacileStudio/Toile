use axum::Router;
use axum::body::Body;
use http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use tower::ServiceExt;

pub struct Answer {
    pub status: StatusCode,
    pub headers: http::HeaderMap,
    pub bytes: Vec<u8>,
}

impl Answer {
    pub fn json(&self) -> serde_json::Value {
        serde_json::from_slice(&self.bytes)
            .unwrap_or_else(|_| panic!("not json: {}", String::from_utf8_lossy(&self.bytes)))
    }

    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.bytes).into_owned()
    }
}

pub async fn send(
    router: &Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Option<serde_json::Value>,
) -> Answer {
    let mut request = Request::builder().method(method).uri(path).header(header::HOST, "toile.test");
    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let body = match body {
        Some(value) => {
            request = request.header(header::CONTENT_TYPE, "application/json");
            Body::from(value.to_string())
        }
        None => Body::empty(),
    };
    raw(router, request.body(body).unwrap()).await
}

/// A browser login's cookie for `user_id`, `name=token`, as the OIDC callback sets it.
pub async fn browser_cookie(sessions: &porte::session::Manager, user_id: i64) -> String {
    let template = porte::Session { user_id, ..porte::Session::default() };
    let (token, _, cookie) = sessions.issue_cookie_for(&Request::new(()), template).await.unwrap();
    let name = cookie.to_str().unwrap().split_once('=').unwrap().0.to_owned();
    format!("{name}={token}")
}

/// `method path` with `cookie` (`name=token`) and the CSRF header, as the browser client sends.
pub async fn with_cookie(router: &Router, method: &str, path: &str, cookie: &str) -> Answer {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::HOST, "toile.test")
        .header(header::COOKIE, cookie)
        .header(porte::CSRF_HEADER_NAME, "1")
        .body(Body::empty())
        .unwrap();
    raw(router, request).await
}

pub async fn raw(router: &Router, request: Request<Body>) -> Answer {
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes().to_vec();
    Answer { status, headers, bytes }
}
