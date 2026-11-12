//! ilia fork: the TON settings page, `/dashboard/admin/ton` (admin only;
//! ilia-booking pricing design §2.4 with its critique).
//!
//! What it shows: the payments' status (network, wallet in both forms,
//! watcher, open invoices, the indexer host and whether the API keys are
//! set), every tuning key of `crate::ton::settings::SPECS` with where its
//! value comes from, the demand ladder with a preview (the price by `n`, the
//! admin's own `n` now, and every priced event type with its mode), and the
//! last changes.
//!
//! What it changes: only the tuning keys, and of those only the ones that
//! neither the environment nor `config apply` owns (env > DB > default; a
//! row owned by `apply` is read-only here). A locked field is shown
//! disabled with the reason, and a forced POST of it is ignored. The
//! network, the wallet address, the indexer and the API keys are never
//! fields: they stay in the server's environment and systemd credentials.
//!
//! Saving checks every field with the environment's own parser
//! (`settings::to_env`), then the whole merged result
//! (`settings::validate`); anything invalid stores nothing and is shown next
//! to its field. A valid save writes the rows and their audit trail in one
//! transaction (`super::settings::write`), then puts the settings into
//! effect without a restart (`TonSlot::apply_settings`).
//!
//! At the top, the Debug block: the test booking page's switch
//! (`settings::DEBUG_BOOKING`, stored and audited with the keys above but
//! never part of this form: its own POST is `debug_booking::toggle`) and,
//! while on, the button to the page (`crate::web::debug_booking`); and the
//! guest Telegram username that page prefills
//! (`settings::DEBUG_BOOKING_TELEGRAM`, its own POST
//! `debug_booking::save_telegram`).

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Form, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use chrono::Utc;
use minijinja::context;

use super::{settings as stored, store, Applied, TARGET};
use crate::auth::AdminUser;
use crate::ton::amount::{format_ton, format_usd_cents, from_i64};
use crate::ton::ladder::{Ladder, Step};
use crate::ton::settings::{self, DbValues, Kind, Owner, Source};
use crate::web::{
    admin_sidebar_context, internal_error_body, internal_error_html, verify_csrf_token, AppState,
};

/// The page's address.
pub(crate) const PATH: &str = "/dashboard/admin/ton";

/// Changes listed on the page.
const AUDIT_ROWS: i64 = 20;

/// The page's sections, each a list of `settings::SPECS` fields, in order.
pub(crate) const GROUPS: &[(&str, &[&str])] = &[
    ("pricing", &["ladder", "free_per_guest_days"]),
    (
        "timing",
        &[
            "window_min",
            "grace_min",
            "fuse_hours",
            "renew_min_lead_hours",
            "renew_window_min",
            "max_renewals",
            "underpay_tolerance_ton",
        ],
    ),
    (
        "rate",
        &[
            "rate_min_usd",
            "rate_max_usd",
            "rate_max_spread_pct",
            "rate_max_age_min",
            "rate_max_jump_pct",
        ],
    ),
    ("alerts", &["alert_email"]),
];

/// The posted form: every field by name (`_csrf`, `action` and the
/// `settings::SPECS` fields).
type Posted = HashMap<String, String>;

/// Errors by field; `""` is an error of the whole form.
type Errors = HashMap<&'static str, String>;

fn tr(lang: &str, key: &str) -> String {
    crate::i18n::translate(lang, key, None)
}

/// The rows the save would leave: `before` with every posted field that is
/// not locked (owned by the environment or by `config apply`) on top. A
/// field the form did not carry is left as it is; a blank one goes back to
/// the default (its row is removed). Errors are the parser's own messages.
pub(crate) fn merged(
    env: &dyn Fn(&str) -> Option<String>,
    before: &DbValues,
    posted: &Posted,
) -> (DbValues, Errors) {
    let mut next = before.clone();
    let mut errors = Errors::new();
    for key in settings::SPECS {
        // The debug keys have their own forms (`debug_booking::toggle`,
        // `debug_booking::save_telegram`).
        if settings::is_debug(key.var) {
            continue;
        }
        if matches!(
            settings::source(env, before, key.var),
            Source::Env | Source::Apply
        ) {
            continue;
        }
        let Some(human) = posted.get(key.field) else {
            continue;
        };
        match settings::to_env(key, human) {
            Ok(Some(text)) => next.set(key.var, text, Owner::Panel),
            Ok(None) => next.remove(key.var),
            Err(e) => {
                errors.insert(key.field, e.0);
            }
        }
    }
    (next, errors)
}

/// `GET /dashboard/admin/ton`.
pub(crate) async fn page(
    State(state): State<Arc<AppState>>,
    admin: AdminUser,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let notice = query
        .get("saved")
        .map(String::as_str)
        .filter(|s| matches!(*s, "live" | "off" | "restart" | "invalid" | "none"));
    let mut view = View::stored(notice);
    view.debug_notice = query.get("debug").and_then(|d| {
        [
            "on",
            "off",
            "locked",
            "tg-saved",
            "tg-same",
            "tg-invalid",
            "tg-locked",
        ]
        .into_iter()
        .find(|k| *k == d.as_str())
    });
    render(&state, &admin, view).await
}

/// `POST /dashboard/admin/ton`: save (or, with `action=preview`, only
/// check and show the ladder posted).
pub(crate) async fn save(
    State(state): State<Arc<AppState>>,
    admin: AdminUser,
    headers: HeaderMap,
    Form(posted): Form<Posted>,
) -> Response {
    if let Err(resp) = verify_csrf_token(&headers, &posted.get("_csrf").cloned()) {
        return resp;
    }
    let preview = posted.get("action").map(String::as_str) == Some("preview");
    let env = state.ton.env();
    let env = |k: &str| env.get(k);
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return internal_error_html("ton settings: begin", &e).into_response(),
    };
    let before = match store::load_settings(&mut tx).await {
        Ok(db) => db,
        Err(e) => return internal_error_html("ton settings: read", &e).into_response(),
    };
    let (next, mut errors) = merged(&env, &before, &posted);
    if errors.is_empty() {
        if let Err(e) = settings::validate(&env, &next) {
            let field = settings::key_of_error(&e).map_or("", |k| k.field);
            errors.insert(field, e.0);
        }
    }
    if !errors.is_empty() || preview {
        let _ = tx.rollback().await;
        let ladder = settings::ladder(&env, &next).ok().map(|r| r.value);
        let status = if errors.is_empty() {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        };
        let view = View {
            posted: Some(posted),
            errors,
            notice: None,
            debug_notice: None,
            preview: if preview { ladder } else { None },
        };
        let mut resp = render(&state, &admin, view).await;
        if resp.status() == StatusCode::OK {
            *resp.status_mut() = status;
        }
        return resp;
    }
    let changes = stored::diff(&before, &next);
    if changes.is_empty() {
        let _ = tx.rollback().await;
        return Redirect::to(&format!("{PATH}?saved=none")).into_response();
    }
    if let Err(e) = stored::write(
        &mut tx,
        &changes,
        Some(&admin.user.id),
        Owner::Panel,
        Utc::now(),
    )
    .await
    {
        return internal_error_html("ton settings: write", &e).into_response();
    }
    if let Err(e) = tx.commit().await {
        return internal_error_html("ton settings: commit", &e).into_response();
    }
    stored::log_changes(&changes, Some(&admin.user.id), Owner::Panel);
    let applied = state.ton.apply_settings(&next).await;
    tracing::info!(
        target: TARGET,
        admin = %admin.user.email,
        keys = changes.len(),
        ?applied,
        "admin: ton settings saved"
    );
    let saved = match applied {
        Applied::Live => "live",
        Applied::Off => "off",
        Applied::RestartNeeded => "restart",
        Applied::Invalid(_) => "invalid",
    };
    Redirect::to(&format!("{PATH}?saved={saved}")).into_response()
}

/// What the page shows on top of the stored state.
struct View {
    /// The form as posted (a refused save, a preview): its values are shown
    /// again in the fields that are not locked.
    posted: Option<Posted>,
    errors: Errors,
    notice: Option<&'static str>,
    /// What the debug switch's POST did (`on`, `off`, `locked`).
    debug_notice: Option<&'static str>,
    /// The ladder to preview instead of the one in effect.
    preview: Option<Ladder>,
}

impl View {
    fn stored(notice: Option<&str>) -> Self {
        Self {
            posted: None,
            errors: Errors::new(),
            notice: notice.and_then(|n| {
                ["live", "off", "restart", "invalid", "none"]
                    .into_iter()
                    .find(|k| *k == n)
            }),
            debug_notice: None,
            preview: None,
        }
    }
}

fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Int(_) => "int",
        Kind::Nano(_) => "ton",
        Kind::Usd => "usd",
        Kind::Pct(_) => "pct",
        Kind::Email => "email",
        Kind::Ladder => "ladder",
        Kind::Bool => "bool",
        Kind::Telegram => "telegram",
    }
}

fn step_text(lang: &str, step: Step) -> String {
    match step {
        Step::Free => tr(lang, "ton-admin-free"),
        Step::Ton(n) => format!("{} TON", crate::ton::ladder::ton_text(n)),
    }
}

/// The price table: one row per step, the last one "n or more".
fn preview_rows(lang: &str, ladder: &Ladder) -> Vec<minijinja::Value> {
    let rows = settings::ladder_preview(ladder);
    let last = rows.len().saturating_sub(2);
    rows.into_iter()
        .take(last + 1)
        .map(|(n, _)| {
            context! {
                n => n,
                or_more => n == last,
                price => step_text(lang, ladder.price(n as i64)),
            }
        })
        .collect()
}

async fn render(state: &AppState, admin: &AdminUser, view: View) -> Response {
    let lang = admin.lang;
    let env_snapshot = state.ton.env();
    let env = |k: &str| env_snapshot.get(k);
    let db = match state.pool.acquire().await {
        Ok(mut conn) => store::load_settings(&mut conn).await,
        Err(e) => Err(e),
    };
    let db = match db {
        Ok(db) => db,
        Err(e) => return internal_error_html("ton settings: read", &e).into_response(),
    };

    // The fields, by section.
    let debug = crate::web::debug_booking::block(state).await;
    let groups: Vec<minijinja::Value> = GROUPS
        .iter()
        .map(|(group, fields)| {
            let fields: Vec<minijinja::Value> = fields
                .iter()
                .filter_map(|f| settings::by_field(f))
                .map(|key| {
                    let source = settings::source(&env, &db, key.var);
                    let locked = matches!(source, Source::Env | Source::Apply);
                    let stored_text = match source {
                        Source::Env => env(key.var).unwrap_or_default(),
                        _ => db.value(key.var).unwrap_or_default().to_string(),
                    };
                    let value = match (&view.posted, locked) {
                        (Some(posted), false) => posted.get(key.field).cloned().unwrap_or_default(),
                        _ => settings::to_human(key, &stored_text),
                    };
                    let (min, max) = match key.kind {
                        Kind::Int(s) => (Some(s.min), Some(s.max)),
                        _ => (None, None),
                    };
                    context! {
                        field => key.field,
                        var => key.var,
                        kind => kind_name(key.kind),
                        label => tr(lang, &format!("ton-admin-f-{}", key.field)),
                        value => value,
                        default => key.default_text(),
                        source => source.as_str(),
                        locked => locked,
                        error => view.errors.get(key.field),
                        min => min,
                        max => max,
                    }
                })
                .collect();
            context! {
                id => group,
                title => tr(lang, &format!("ton-admin-group-{group}")),
                fields => fields,
            }
        })
        .collect();

    // The ladder: in effect, or the one posted for a preview.
    let effective = settings::ladder(&env, &db);
    let ladder_error = effective.as_ref().err().map(|e| e.0.clone());
    let ladder = view
        .preview
        .clone()
        .or_else(|| effective.ok().map(|r| r.value))
        .unwrap_or_default();
    let mut conn = match state.pool.acquire().await {
        Ok(c) => c,
        Err(e) => return internal_error_html("ton settings: read", &e).into_response(),
    };
    let now = Utc::now();
    let my_n = store::demand_n(&mut conn, &admin.user.id, now, None)
        .await
        .unwrap_or(0);

    // Every priced event type, with its mode and its price now.
    let rows: Vec<PricedType> = sqlx::query_as(
        "SELECT et.title, et.slug, COALESCE(u.name, ''), u.id, tp.mode, tp.currency,
                tp.price_nano, tp.price_usd_cents, tp.enabled
         FROM ton_prices tp
         JOIN event_types et ON et.id = tp.event_type_id
         LEFT JOIN accounts a ON a.id = et.account_id
         LEFT JOIN users u ON u.id = a.user_id
         ORDER BY u.name, et.title",
    )
    .fetch_all(&mut *conn)
    .await
    .unwrap_or_default();
    let mut demand: HashMap<String, i64> = HashMap::new();
    let ceiling = ladder.ceiling().nano();
    let mut types = Vec::new();
    for (title, slug, owner, owner_id, mode, currency, nano, cents, enabled) in rows {
        let (price, now_price, at_or_below_ceiling) = if mode == "ladder" {
            let n = match owner_id.as_deref() {
                Some(id) => match demand.get(id) {
                    Some(n) => *n,
                    None => {
                        let n = store::demand_n(&mut conn, id, now, None).await.unwrap_or(0);
                        demand.insert(id.to_string(), n);
                        n
                    }
                },
                None => 0,
            };
            (
                tr(lang, "ton-admin-mode-ladder"),
                Some(step_text(lang, ladder.price(n))),
                false,
            )
        } else if currency == "USD" {
            let cents = u128::try_from(cents.unwrap_or(0)).unwrap_or(0);
            (format!("{} USD", format_usd_cents(cents)), None, false)
        } else {
            let nano = from_i64(nano.unwrap_or(0));
            let below = enabled != 0 && ceiling.is_some_and(|c| nano <= c);
            (format!("{} TON", format_ton(nano)), None, below)
        };
        types.push(context! {
            title => title,
            slug => slug,
            owner => owner,
            mode => mode,
            price => price,
            now_price => now_price,
            enabled => enabled != 0,
            at_or_below_ceiling => at_or_below_ceiling,
        });
    }
    let open_invoices: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM ton_invoices WHERE status IN ('open', 'paid')")
            .fetch_one(&mut *conn)
            .await
            .unwrap_or(0);
    drop(conn);

    // The payments' status: read-only, from the running runtime.
    let rt = state.ton.get();
    let status = context! {
        on => rt.is_some(),
        network => env("CALRS_TON_NETWORK").map(|v| v.trim().to_string()).filter(|v| !v.is_empty()),
        disabled_reason => rt.is_none().then(|| state.ton.disabled_reason()).flatten(),
        address => rt.as_ref().map(|r| r.cfg.address_friendly.clone()),
        address_raw => rt.as_ref().map(|r| r.cfg.address_raw.clone()),
        indexer_host => rt.as_ref().map(|r| r.cfg.indexer_host()),
        ready => rt.as_ref().map(|r| r.ready_on_demand(now).err().map(|e| e.to_string())),
        last_ok => rt.as_ref().and_then(|r| r.health().last_ok).map(super::stamp),
        api_key => env_snapshot.secret_state("CALRS_TON_API_KEY"),
        coingecko_key => env_snapshot.secret_state("CALRS_TON_COINGECKO_KEY"),
        open_invoices => open_invoices,
    };

    let tmpl = match state.templates.get_template("ton_settings.html") {
        Ok(t) => t,
        Err(e) => return internal_error_html("template render", &e).into_response(),
    };
    Html(
        tmpl.render(context! {
            sidebar => admin_sidebar_context(&admin.user, "admin_ton"),
            lang => lang,
            impersonating => false,
            impersonating_name => "",
            status => status,
            debug => debug,
            debug_notice => view.debug_notice,
            groups => groups,
            general_error => view.errors.get(""),
            has_errors => !view.errors.is_empty(),
            notice => view.notice,
            previewing => view.preview.is_some(),
            ladder_steps => ladder.steps().iter().map(|s| s.text()).collect::<Vec<_>>(),
            ladder_error => ladder_error,
            preview => preview_rows(lang, &ladder),
            my_n => my_n,
            my_price => step_text(lang, ladder.price(my_n)),
            types => types,
            audit => stored::recent(&state.pool, AUDIT_ROWS).await,
        })
        .unwrap_or_else(|e| internal_error_body("template render", &e)),
    )
    .into_response()
}

/// One `ton_prices` row of the preview's list: title, slug, owner name and
/// id, mode, currency, nanotons, cents, enabled.
type PricedType = (
    String,
    String,
    String,
    Option<String>,
    String,
    String,
    Option<i64>,
    Option<i64>,
    i64,
);

#[cfg(test)]
mod tests {
    use super::*;

    /// Every stored key is on the page exactly once, with a label in
    /// English.
    #[test]
    fn every_key_has_one_field_and_a_label() {
        let mut seen: Vec<&str> = GROUPS.iter().flat_map(|(_, f)| f.iter().copied()).collect();
        seen.sort_unstable();
        // The debug switch is the Debug block's, not a field of the form.
        let mut all: Vec<&str> = settings::SPECS
            .iter()
            .filter(|k| !settings::is_debug(k.var))
            .map(|k| k.field)
            .collect();
        all.sort_unstable();
        assert_eq!(seen, all);
        for key in settings::SPECS {
            let id = format!("ton-admin-f-{}", key.field);
            assert_ne!(tr("en", &id), id, "{id} has no English text");
        }
        for (group, _) in GROUPS {
            let id = format!("ton-admin-group-{group}");
            assert_ne!(tr("en", &id), id, "{id} has no English text");
        }
    }

    #[test]
    fn locked_fields_are_ignored_and_blank_is_the_default() {
        let env = |k: &str| (k == "CALRS_TON_GRACE_MIN").then(|| "20".to_string());
        let mut before = DbValues::default();
        before.set("CALRS_TON_WINDOW_MIN", "45".into(), Owner::Apply);
        before.set("CALRS_TON_FUSE_HOURS", "8".into(), Owner::Panel);
        let posted: Posted = [
            ("grace_min", "30"),
            ("window_min", "60"),
            ("fuse_hours", ""),
            ("max_renewals", "3"),
            ("ladder", "free, 10,20"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        let (next, errors) = merged(&env, &before, &posted);
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(next.value("CALRS_TON_GRACE_MIN"), None, "env-owned");
        assert_eq!(
            next.value("CALRS_TON_WINDOW_MIN"),
            Some("45"),
            "apply-owned"
        );
        assert_eq!(next.owner("CALRS_TON_WINDOW_MIN"), Some(Owner::Apply));
        assert_eq!(next.value("CALRS_TON_FUSE_HOURS"), None, "blank: default");
        assert_eq!(next.value("CALRS_TON_MAX_RENEWALS"), Some("3"));
        assert_eq!(next.value("CALRS_TON_LADDER"), Some("free,10,20"));
        // The debug switch is never this form's, even forced.
        let (next, _) = merged(
            &env,
            &before,
            &[("debug_booking".to_string(), "on".to_string())].into(),
        );
        assert_eq!(next.value(settings::DEBUG_BOOKING), None);
        let (next, _) = merged(
            &env,
            &before,
            &[(
                "debug_booking_telegram".to_string(),
                "getmonolith".to_string(),
            )]
            .into(),
        );
        assert_eq!(next.value(settings::DEBUG_BOOKING_TELEGRAM), None);
        let (_, errors) = merged(
            &env,
            &before,
            &[("max_renewals".to_string(), "9".to_string())].into(),
        );
        assert!(errors["max_renewals"].starts_with("CALRS_TON_MAX_RENEWALS"));
    }

    #[test]
    fn the_preview_ends_at_the_ceiling() {
        let l = Ladder::parse("free,20,40").unwrap();
        let rows = preview_rows("en", &l);
        assert_eq!(rows.len(), 3);
        let last = &rows[2];
        assert_eq!(last.get_attr("n").unwrap().as_usize(), Some(2));
        assert!(last.get_attr("or_more").unwrap().is_true());
        assert_eq!(last.get_attr("price").unwrap().as_str(), Some("40 TON"));
    }
}
