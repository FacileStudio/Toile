//! WebSocket auth for clients that cannot set headers: the installed apps.

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use http::header::{AUTHORIZATION, COOKIE, SEC_WEBSOCKET_PROTOCOL};
use http::{HeaderValue, StatusCode};

/// The WebSocket subprotocol a native client offers beside its `bearer.<token>` entry.
pub const WS_PROTOCOL: &str = "toile";

const WS_BEARER_PREFIX: &str = "bearer.";

fn offered(request: &Request) -> Vec<String> {
    request
        .headers()
        .get_all(SEC_WEBSOCKET_PROTOCOL)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .map(|entry| entry.trim().to_owned())
        .filter(|entry| !entry.is_empty())
        .collect()
}

fn lift_bearer(request: &mut Request, protocols: &[String]) {
    if request.headers().contains_key(AUTHORIZATION) || request.headers().contains_key(COOKIE) {
        return;
    }
    let token = protocols.iter().find_map(|entry| entry.strip_prefix(WS_BEARER_PREFIX)).unwrap_or_default();
    if let Ok(value) = HeaderValue::from_str(&format!("Bearer {token}"))
        && !token.is_empty()
    {
        request.headers_mut().insert(AUTHORIZATION, value);
    }
}

/// Lets a WebSocket client that cannot set headers authenticate: a browser-less app opens the
/// board socket with the subprotocols `toile` and `bearer.<session token>`. With no cookie and
/// no `Authorization`, the token becomes `Authorization: Bearer`, and a 101 answer that names no
/// subprotocol names `toile`, which the client requires.
pub async fn ws_bearer(mut request: Request, next: Next) -> Response {
    let protocols = offered(&request);
    if protocols.is_empty() {
        return next.run(request).await;
    }
    lift_bearer(&mut request, &protocols);
    let mut response = next.run(request).await;
    let named = protocols.iter().any(|entry| entry == WS_PROTOCOL);
    if named
        && response.status() == StatusCode::SWITCHING_PROTOCOLS
        && !response.headers().contains_key(SEC_WEBSOCKET_PROTOCOL)
    {
        response.headers_mut().insert(SEC_WEBSOCKET_PROTOCOL, HeaderValue::from_static(WS_PROTOCOL));
    }
    response
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum::body::Body;
    use axum::routing::get;
    use tower::ServiceExt;

    use super::*;

    fn app() -> Router {
        let echo = |request: Request| async move {
            let auth = request.headers().get(AUTHORIZATION).map(|v| v.to_str().unwrap_or("").to_owned());
            (StatusCode::SWITCHING_PROTOCOLS, auth.unwrap_or_default())
        };
        Router::new().route("/ws", get(echo)).layer(axum::middleware::from_fn(ws_bearer))
    }

    async fn call(headers: &[(&str, &str)]) -> (String, Option<String>) {
        let mut request = Request::builder().uri("/ws");
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        let response = app().oneshot(request.body(Body::empty()).unwrap()).await.unwrap();
        let protocol = response.headers().get(SEC_WEBSOCKET_PROTOCOL).map(|v| v.to_str().unwrap().to_owned());
        let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
        (String::from_utf8(body.to_vec()).unwrap(), protocol)
    }

    #[tokio::test]
    async fn a_subprotocol_token_becomes_the_bearer() {
        let (auth, protocol) = call(&[("sec-websocket-protocol", "toile, bearer.abc_-1")]).await;
        assert_eq!(auth, "Bearer abc_-1");
        assert_eq!(protocol.as_deref(), Some("toile"));
    }

    #[tokio::test]
    async fn a_cookie_or_header_wins_over_the_subprotocol() {
        let (auth, _) = call(&[("sec-websocket-protocol", "toile, bearer.abc"), ("cookie", "session=x")]).await;
        assert_eq!(auth, "");
        let (auth, _) =
            call(&[("sec-websocket-protocol", "toile, bearer.abc"), ("authorization", "Bearer real")]).await;
        assert_eq!(auth, "Bearer real");
    }

    #[tokio::test]
    async fn no_subprotocol_changes_nothing() {
        assert_eq!(call(&[]).await, (String::new(), None));
    }
}
