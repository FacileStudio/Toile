//! The API reference at `/docs`: one row per route, checked against the router by a test.

use utoipa::openapi::path::{HttpMethod, OperationBuilder, PathItem};
use utoipa::openapi::response::ResponseBuilder;
use utoipa::openapi::{InfoBuilder, OpenApi, OpenApiBuilder, PathsBuilder};

/// One documented route: method, path under `/api` (or the root), tag, summary, success status.
type Row = (&'static str, &'static str, &'static str, &'static str, &'static str);

const ROUTES: &[Row] = &[
    ("GET", "/version", "meta", "The release this binary was built from", "200"),
    ("GET", "/auth/config", "auth", "Whether sign-in is SSO only and OIDC is on", "200"),
    ("GET", "/auth/oidc", "auth", "Starts Registre sign-in (302)", "302"),
    ("GET", "/auth/oidc/callback", "auth", "Registre's redirect back, sets the session cookie (302)", "302"),
    ("POST", "/auth/oidc/exchange", "auth", "Trades a CLI one-time code for a session token", "200"),
    ("POST", "/auth/oidc/device/exchange", "auth", "Trades a facile-cli device token for a session token", "200"),
    ("POST", "/auth/sync-profile", "auth", "Refreshes the profile from Registre", "200"),
    ("POST", "/auth/backchannel-logout", "auth", "Registre's back-channel logout", "200"),
    ("POST", "/auth/logout", "auth", "Ends the session", "200"),
    ("GET", "/auth/me", "auth", "The signed-in person, or 401", "200"),
    ("GET", "/auth/session", "auth", "The signed-in person, or {\"user\": null}", "200"),
    ("POST", "/registre/sync", "registre", "Registre's directory nudge", "202"),
    ("POST", "/registre/deletions", "registre", "Registre's signed account deletion", "202"),
    ("GET", "/spaces", "spaces", "The caller's spaces and roles", "200"),
    ("GET", "/boards", "boards", "The live boards of a scope (?space_id=)", "200"),
    ("POST", "/boards", "boards", "Creates a board in a scope", "201"),
    ("GET", "/boards/{id}", "boards", "One board", "200"),
    ("PATCH", "/boards/{id}", "boards", "Renames a board", "200"),
    ("DELETE", "/boards/{id}", "boards", "Trashes a board, or deletes a trashed one (?permanent=true)", "204"),
    ("POST", "/boards/{id}/duplicate", "boards", "Copies a board into a new board of its scope", "201"),
    ("POST", "/boards/{id}/restore", "boards", "Takes a board out of the trash", "200"),
    ("GET", "/boards/{id}/thumbnail.png", "boards", "The board's last thumbnail", "200"),
    ("GET", "/boards/{id}/ws", "rooms", "The board's y-websocket socket (101)", "101"),
    ("GET", "/board-schema", "rooms", "The board schema version, shape types and limits", "200"),
    ("GET", "/boards/{id}/shares", "share", "The board's live share links", "200"),
    ("POST", "/boards/{id}/shares", "share", "Creates a view, comment or edit link", "201"),
    ("DELETE", "/boards/{id}/shares/{share}", "share", "Revokes a link and closes its sockets", "204"),
    ("GET", "/share/{token}", "share", "The shared board and the link's role", "200"),
    ("GET", "/share/{token}/ws", "share", "A guest's y-websocket socket (?name=, 101)", "101"),
    ("GET", "/boards/{id}/comments", "comments", "The board's comments, oldest first", "200"),
    ("POST", "/boards/{id}/comments", "comments", "Adds a comment or a reply", "201"),
    ("PATCH", "/boards/{id}/comments/{comment}", "comments", "Edits or resolves a comment", "200"),
    ("DELETE", "/boards/{id}/comments/{comment}", "comments", "Deletes a comment and its replies", "204"),
    ("GET", "/share/{token}/comments", "comments", "The comments a guest sees", "200"),
    ("POST", "/share/{token}/comments", "comments", "A guest's comment, signed with its name", "201"),
    (
        "POST",
        "/boards/{id}/assets",
        "assets",
        "Uploads an image (multipart `file`) into the uploader's Nuage app folder",
        "201",
    ),
    ("GET", "/boards/{id}/assets/{asset}", "assets", "An image of the board, cached by ETag", "200"),
    ("GET", "/share/{token}/assets/{asset}", "assets", "An image of the board, for a link holder", "200"),
    ("POST", "/share/{token}/assets", "assets", "Always 403: guests cannot upload", "403"),
    ("POST", "/cards/resolve", "cards", "Resolves a link into a live card for the caller", "200"),
    ("POST", "/boards/{id}/shapes/{shape}/task", "tasks", "Turns a sticky into an Opus task and adds its card", "201"),
    ("GET", "/me/entitlements", "suite", "The plan values of the personal scope or of a space", "200"),
];

fn method(name: &str) -> HttpMethod {
    match name {
        "POST" => HttpMethod::Post,
        "PATCH" => HttpMethod::Patch,
        "DELETE" => HttpMethod::Delete,
        _ => HttpMethod::Get,
    }
}

/// The OpenAPI document of every route in [`ROUTES`].
pub fn document() -> OpenApi {
    let paths = ROUTES.iter().fold(PathsBuilder::new(), |paths, (verb, path, tag, summary, status)| {
        let operation = OperationBuilder::new()
            .tag(*tag)
            .summary(Some(*summary))
            .response(*status, ResponseBuilder::new().description(*summary).build());
        paths.path(*path, PathItem::new(method(verb), operation))
    });
    let info = InfoBuilder::new().title("Toile API").version(crate::version::VERSION).build();
    OpenApiBuilder::new().info(info).paths(paths.build()).build()
}

/// The page's title, description and servers.
pub fn config() -> tronc::apiref::Config {
    tronc::apiref::Config {
        title: "Toile API".to_owned(),
        version: crate::version::VERSION.trim_start_matches('v').to_owned(),
        description: "Collaborative infinite canvas: boards per scope, personal or a Registre space.".to_owned(),
        servers: vec!["/api".to_owned()],
        script_url: String::new(),
    }
}
