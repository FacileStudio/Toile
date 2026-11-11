mod common;

use axum::body::Body;
use common::fixture::{self, Roles};
use common::http::{Answer, raw};
use common::{App, Person};
use http::{Request, header};

const EXPORTS: [(&str, &str); 5] = [
    ("export.svg", "image/svg+xml"),
    ("export.png", "image/png"),
    ("export.pdf", "application/pdf"),
    ("export.canvas", "application/json"),
    ("thumbnail.png", "image/png"),
];

const EXCALIDRAW: &[u8] = include_bytes!("fixtures/scene.excalidraw");

async fn upload(app: &App, who: &Person, file_name: &str, bytes: &[u8], space: Option<&str>) -> Answer {
    let boundary = "toile-test-boundary";
    let mut body = Vec::new();
    if let Some(space) = space {
        body.extend(
            format!("--{boundary}\r\nContent-Disposition: form-data; name=\"space_id\"\r\n\r\n{space}\r\n").bytes(),
        );
    }
    let head =
        format!("--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\n\r\n");
    body.extend(head.bytes());
    body.extend_from_slice(bytes);
    body.extend(format!("\r\n--{boundary}--\r\n").bytes());
    let request = Request::post("/api/boards/import")
        .header(header::HOST, "toile.test")
        .header(header::AUTHORIZATION, format!("Bearer {}", who.token))
        .header("x-facile-csrf", "1")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .body(Body::from(body))
        .unwrap();
    raw(&app.router, request).await
}

async fn setup(prefix: &str) -> Option<(App, Roles, String)> {
    let app = common::app(common::pool(prefix).await?).await;
    let roles = fixture::roles(&app).await;
    let board = fixture::id(&fixture::board(&app, &roles.owner, Some(&roles.space), "Rétro").await);
    Some((app, roles, board))
}

#[tokio::test]
async fn every_export_answers_members_and_hides_the_board_from_others() {
    let Some((app, roles, board)) = setup("toile_export_authz").await else { return };
    for (path, mime) in EXPORTS {
        let url = format!("/api/boards/{board}/{path}");
        let ok = app.call("GET", &url, Some(&roles.member), None).await;
        assert_eq!(ok.status, 200, "{path}: {}", ok.text());
        assert_eq!(ok.headers[header::CONTENT_TYPE], mime, "{path}");
        assert!(!ok.bytes.is_empty(), "{path}");
        assert_eq!(app.call("GET", &url, Some(&roles.outsider), None).await.status, 404, "{path}");
        assert_eq!(app.call("GET", &url, None, None).await.status, 401, "{path}");
    }
    let missing = format!("/api/boards/{}/export.svg", uuid::Uuid::new_v4());
    assert_eq!(app.call("GET", &missing, Some(&roles.owner), None).await.status, 404);
    let frame = format!("/api/boards/{board}/export.svg?frame=nope");
    assert_eq!(app.call("GET", &frame, Some(&roles.owner), None).await.status, 404);
}

#[tokio::test]
async fn an_import_creates_a_board_in_the_scope_and_reports_what_it_skipped() {
    let Some((app, roles, _)) = setup("toile_import").await else { return };
    let answer = upload(&app, &roles.member, "Atelier.excalidraw", EXCALIDRAW, Some(&roles.space)).await;
    assert_eq!(answer.status, 201, "{}", answer.text());
    let body = answer.json();
    assert_eq!((body["name"].as_str(), body["skipped"].as_u64()), (Some("Atelier"), Some(2)));
    assert_eq!(body["space_id"].as_str(), Some(roles.space.as_str()));
    let board = body["id"].as_str().unwrap();
    let canvas = app.call("GET", &format!("/api/boards/{board}/export.canvas"), Some(&roles.member), None).await;
    let nodes = canvas.json()["nodes"].as_array().unwrap().len();
    assert_eq!(nodes, 5, "{}", canvas.text());
    let other = upload(&app, &roles.outsider, "a.excalidraw", EXCALIDRAW, Some(&roles.space)).await;
    assert_eq!(other.status, 404);
    let junk = upload(&app, &roles.member, "notes.txt", b"hello", None).await;
    assert_eq!(junk.status, 400);
}

#[tokio::test]
async fn an_import_over_five_mib_is_refused() {
    let Some((app, roles, _)) = setup("toile_import_cap").await else { return };
    let big = vec![b' '; 5 * 1024 * 1024 + 1];
    let answer = upload(&app, &roles.owner, "big.canvas", &big, None).await;
    assert_eq!(answer.status, 413, "{}", answer.text());
}
