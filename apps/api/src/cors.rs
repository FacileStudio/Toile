//! CORS for `POST /api/boards/meeting` alone: Echo's page calls it with the person's cookie.
//! It wraps the app outside tronc's layers, so tronc's own CORS list (native apps, no
//! credentials) neither answers nor refuses Echo's preflight.

use axum::Router;
use axum::extract::Request;
use axum::middleware::{Next, from_fn};
use axum::response::{IntoResponse, Response};
use http::{HeaderMap, HeaderValue, Method, StatusCode, header};

/// The origins allowed to call the meeting route with credentials.
pub const MEETING_ORIGINS: &[&str] = &["https://echo.facile.studio"];

/// The path the CORS rule covers.
pub const MEETING_PATH: &str = "/api/boards/meeting";

/// `scheme://host[:port]` of `url`; empty when `url` has no scheme.
pub fn origin_of(url: &str) -> &str {
    let Some(start) = url.find("://").map(|i| i + 3) else { return "" };
    url[start..].find('/').map_or(url, |end| &url[..start + end])
}

fn decorate(headers: &mut HeaderMap, origin: HeaderValue) {
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
    headers.insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_static("true"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static("POST, OPTIONS"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("content-type, x-facile-csrf"));
    headers.insert(header::ACCESS_CONTROL_MAX_AGE, HeaderValue::from_static("600"));
    headers.append(header::VARY, HeaderValue::from_static("Origin"));
}

async fn guard(request: Request, next: Next) -> Response {
    let origin = request.headers().get(header::ORIGIN).cloned();
    let allowed = origin.filter(|o| MEETING_ORIGINS.iter().any(|m| o.as_bytes() == m.as_bytes()));
    let Some(origin) = allowed.filter(|_| request.uri().path() == MEETING_PATH) else {
        return next.run(request).await;
    };
    let mut response = match *request.method() {
        Method::OPTIONS => StatusCode::NO_CONTENT.into_response(),
        _ => next.run(request).await,
    };
    decorate(response.headers_mut(), origin);
    response
}

/// `app` with the meeting route's CORS rule outside everything else.
pub fn meetings(app: Router) -> Router {
    app.layer(from_fn(guard))
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_origin_keeps_scheme_and_host_only() {
        assert_eq!(
            super::origin_of("https://toile.facile.studio/api/auth/oidc/callback"),
            "https://toile.facile.studio"
        );
        assert_eq!(super::origin_of("http://localhost:4000"), "http://localhost:4000");
        assert_eq!(super::origin_of(""), "");
    }
}
