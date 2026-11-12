//! ilia fork: `CALRS_EMBED_ORIGINS` decides which sites may frame the
//! embedded booking pages (`embed_origins.rs`).

use super::*;
use crate::web::embed_origins::EmbedOrigins;

/// The seeded test app, with the embed policy set as `calrs serve` would
/// from the environment.
async fn app_with(policy: EmbedOrigins) -> (Router, std::sync::Arc<TestDataDir>) {
    let pool = setup_test_db().await;
    seed_test_data(&pool).await;
    let data_dir = std::sync::Arc::new(TestDataDir::new());
    let mut state = build_state(pool, data_dir.to_path_buf(), [0u8; 32]).await;
    Arc::get_mut(&mut state)
        .expect("fresh state has one owner")
        .embed_origins = policy;
    let app = router(state).layer(local_proxy_peer());
    (app, data_dir)
}

async fn framing(app: &Router, uri: &str) -> (String, String) {
    let response = app.clone().oneshot(get(uri)).await.unwrap();
    assert_eq!(response.status(), 200, "{uri}");
    let header = |name| {
        response
            .headers()
            .get(name)
            .map(|v: &axum::http::HeaderValue| v.to_str().unwrap().to_string())
            .unwrap_or_default()
    };
    let csp = header(axum::http::header::CONTENT_SECURITY_POLICY);
    let frame_ancestors = csp
        .split(';')
        .map(str::trim)
        .find(|d| d.starts_with("frame-ancestors"))
        .unwrap_or_default()
        .to_string();
    (frame_ancestors, header(axum::http::header::X_FRAME_OPTIONS))
}

/// Booking surfaces asked for with `?embed=1`, and whether their handler
/// renders through `html_response` (which sets `X-Frame-Options: ALLOWALL`).
/// Upstream's profile and legacy slot pages do not, so they keep
/// `SAMEORIGIN`, which browsers ignore once `frame-ancestors` is set.
fn embedded_pages() -> Vec<(String, bool)> {
    vec![
        ("/u/testuser/test-meeting?embed=1".to_string(), true),
        (format!("{}&embed=1", booking_form_uri(None)), true),
        ("/u/testuser?embed=1".to_string(), false),
        ("/test-meeting?embed=1".to_string(), false),
    ]
}

#[tokio::test]
async fn listed_origins_frame_the_widget_and_nothing_else() {
    let (app, _dir) = app_with(EmbedOrigins::parse("https://ilia.ae").unwrap()).await;
    for (uri, allowall) in embedded_pages() {
        let (fa, xfo) = framing(&app, &uri).await;
        assert_eq!(fa, "frame-ancestors 'self' https://ilia.ae", "{uri}");
        let expected = if allowall { "ALLOWALL" } else { "SAMEORIGIN" };
        assert_eq!(xfo, expected, "{uri}");
    }
    // Without the flag, and on pages that are not booking pages, nothing
    // changes: only calrs itself may frame them.
    for uri in [
        "/u/testuser/test-meeting",
        "/auth/login?embed=1",
        "/embed.js?embed=1",
    ] {
        let (fa, xfo) = framing(&app, uri).await;
        assert_eq!(fa, "frame-ancestors 'self'", "{uri}");
        assert_eq!(xfo, "SAMEORIGIN", "{uri}");
    }
}

#[tokio::test]
async fn none_keeps_every_page_unframeable() {
    let (app, _dir) = app_with(EmbedOrigins::Off).await;
    for (uri, _) in embedded_pages() {
        let (fa, xfo) = framing(&app, &uri).await;
        assert_eq!(fa, "frame-ancestors 'self'", "{uri}");
        assert_eq!(xfo, "SAMEORIGIN", "{uri}");
    }
}

#[tokio::test]
async fn unset_keeps_upstream_any_site() {
    let (app, _dir) = app_with(EmbedOrigins::from_var(None)).await;
    let (fa, xfo) = framing(&app, "/u/testuser/test-meeting?embed=1").await;
    assert_eq!(fa, "frame-ancestors *");
    assert_eq!(xfo, "ALLOWALL");
}
