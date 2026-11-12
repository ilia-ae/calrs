//! ilia fork: `config apply`, the `site.ton` section: the TON tuning keys
//! the admin page also edits (`crate::ton::settings::SPECS`: invoice
//! timing, the rate corridor and rules, the alert email, the demand ladder
//! and the free-step limit), stored in `ton_settings` with the owner
//! `apply`.
//!
//! - **A section left out changes nothing.** Only a present `site.ton`
//!   takes keys over or hands them back.
//! - **A key in the section** is written with the owner `apply`, so the
//!   admin page shows it read-only ("set by config apply"). `null` stores
//!   the default and still owns the key.
//! - **A key the section leaves out** that `apply` owned goes back to the
//!   page: the row keeps its value with the owner `panel` (a `null` row is
//!   removed). This is the one place a left-out key is not "leave as is",
//!   reported as `site.ton.<key>.owner`.
//! - **Values** are the environment's, in the page's units: whole numbers
//!   as JSON integers (`"window_min": 30`), everything else as canonical
//!   strings (`"underpay_tolerance_ton": "0.01"`, `"rate_min_usd": "0.3"`,
//!   `"ladder": "free,20,40"`), checked by the parser the environment and
//!   the page use (`settings::to_env`), then the whole stored result
//!   (`settings::validate`).
//! - **The environment is not seen.** `config apply` is its own process and
//!   cannot read `calrs serve`'s environment, so it validates the stored
//!   values as if no `CALRS_TON_*` were set, and a key the server's
//!   environment sets still wins there (the environment > the database >
//!   the default). The network, the address, the indexer and the API keys
//!   are never keys here.
//! - Every value change sets `restart` (`calrs serve` reads the settings at
//!   startup and on a save of the admin page); an owner change alone does
//!   not. Each change is audited (`ton_settings_audit`, source `apply`).
//! - **Export** prints the keys `apply` owns, so applying an export is no
//!   change; a section with none is left out.
//! - **`debug_booking`** (`settings::DEBUG_BOOKING`, the admin's test
//!   booking page) is a JSON boolean (`null`: off, still owned). `calrs
//!   serve` reads it on every request, so its change sets no `restart`.
//!   It came after the marker below, which is kept as it was: an older
//!   calrs refuses the key as unknown (exit 2, nothing written).

use chrono::Utc;
use serde_json::Value;

use super::{ApplyError, Ctx, TonSection};
use crate::ton::settings::{self, DbValues, Key, Kind, Owner};
use crate::web::ton_pay::{settings as stored, store};

/// The marker of a calrs that knows `site.ton`: the role greps for this
/// literal in the source and in the binary before it sends the section (an
/// older calrs refuses it as an unknown key). Keep it a literal.
pub(crate) const TON_SETTINGS_SHAPE: &str =
    "site.ton: {window_min .. free_per_guest_days}, each a value or null (apply-owned)";

fn path(field: &str) -> String {
    format!("site.ton.{field}")
}

/// No environment: what `config apply` validates against.
fn no_env(_: &str) -> Option<String> {
    None
}

/// A stored value (environment text, `""` the default) in the document.
fn to_doc(key: &Key, text: &str) -> Value {
    let human = settings::to_human(key, text);
    if human.is_empty() {
        return Value::Null;
    }
    match key.kind {
        Kind::Int(_) => human
            .parse::<i64>()
            .map(Value::from)
            .unwrap_or(Value::String(human)),
        Kind::Bool => settings::parse_bool(&human)
            .map(Value::Bool)
            .unwrap_or(Value::String(human)),
        _ => Value::String(human),
    }
}

/// A document value to the stored text (`""` for null), canonical only.
fn from_doc(key: &Key, value: &Value) -> Result<String, ApplyError> {
    let path = path(key.field);
    let human = match (key.kind, value) {
        (_, Value::Null) => return Ok(String::new()),
        (Kind::Bool, Value::Bool(b)) => return Ok(if *b { "1" } else { "0" }.to_string()),
        (Kind::Bool, _) => return Err(ApplyError::invalid(path, "must be true, false or null")),
        (Kind::Int(_), Value::Number(n)) => match n.as_i64() {
            Some(n) => n.to_string(),
            None => return Err(ApplyError::invalid(path, "must be a whole number")),
        },
        (Kind::Int(_), _) => {
            return Err(ApplyError::invalid(path, "must be a whole number or null"))
        }
        (_, Value::String(s)) => s.clone(),
        _ => return Err(ApplyError::invalid(path, "must be a string or null")),
    };
    let canonical = settings::canonical(key, &human)
        .map_err(|e| ApplyError::invalid(&path, e))?
        .ok_or_else(|| ApplyError::invalid(&path, "must not be empty (use null)"))?;
    if canonical != human {
        return Err(ApplyError::invalid(
            path,
            format!("spell it {canonical:?}, as it is stored and exported"),
        ));
    }
    Ok(settings::to_env(key, &human)
        .map_err(|e| ApplyError::invalid(&path, e))?
        .unwrap_or_default())
}

/// The rows after the section: its keys owned by `apply`, the keys `apply`
/// owned that it leaves out handed to the page.
fn planned(before: &DbValues, section: &TonSection) -> Result<DbValues, ApplyError> {
    let mut next = before.clone();
    for (field, value) in section {
        let key = settings::by_field(field).ok_or_else(|| {
            let known: Vec<&str> = settings::SPECS.iter().map(|k| k.field).collect();
            ApplyError::invalid(
                path(field),
                format!(
                    "unknown key ({TON_SETTINGS_SHAPE}); the keys are {}",
                    known.join(", ")
                ),
            )
        })?;
        next.set(key.var, from_doc(key, value)?, Owner::Apply);
    }
    for key in settings::SPECS {
        if before.owner(key.var) != Some(Owner::Apply) || section.contains_key(key.field) {
            continue;
        }
        match before.value(key.var) {
            Some(v) if !v.trim().is_empty() => next.set(key.var, v.to_string(), Owner::Panel),
            _ => next.remove(key.var),
        }
    }
    Ok(next)
}

/// The whole stored result is valid, as the page checks it.
fn validate(next: &DbValues) -> Result<(), ApplyError> {
    settings::validate(&no_env, next).map(|_| ()).map_err(|e| {
        let at =
            settings::key_of_error(&e).map_or_else(|| "site.ton".to_string(), |k| path(k.field));
        ApplyError::invalid(at, e)
    })
}

async fn load(ctx: &Ctx) -> Result<DbValues, ApplyError> {
    let mut conn = ctx.pool().acquire().await?;
    Ok(store::load_settings(&mut conn).await?)
}

/// Reading only: every key and value, and the result as a whole.
pub(super) async fn check(ctx: &Ctx, section: &TonSection) -> Result<(), ApplyError> {
    let before = load(ctx).await?;
    validate(&planned(&before, section)?)
}

/// Write the rows that differ, audited, and record each change.
pub(super) async fn apply(ctx: &mut Ctx, section: &TonSection) -> Result<(), ApplyError> {
    let mut conn = ctx.pool().acquire().await?;
    let before = store::load_settings(&mut conn).await?;
    let next = planned(&before, section)?;
    validate(&next)?;
    let changes = stored::diff(&before, &next);
    if changes.is_empty() {
        return Ok(());
    }
    stored::write(
        &mut conn,
        &changes,
        Some(&ctx.me.id),
        Owner::Apply,
        Utc::now(),
    )
    .await?;
    drop(conn);
    // Logged once the run commits (never on --dry-run or a rollback).
    ctx.ton_audit.extend(changes.iter().cloned());
    for c in &changes {
        let Some(key) = settings::by_var(c.var) else {
            continue;
        };
        let value = |v: &Option<String>| to_doc(key, v.as_deref().unwrap_or(""));
        if key.var == settings::DEBUG_BOOKING {
            // Read per request by `calrs serve`: live, no restart.
            ctx.record(path(key.field), value(&c.before), value(&c.after));
        } else {
            ctx.record_restart(path(key.field), value(&c.before), value(&c.after));
        }
        ctx.record(
            format!("{}.owner", path(key.field)),
            before.owner(c.var).map(Owner::as_str),
            c.owner.map(Owner::as_str),
        );
    }
    Ok(())
}

/// Read back: every key of the section is stored as given and owned by
/// `apply`, and `apply` owns no other.
pub(super) async fn verify(ctx: &Ctx, section: &TonSection) -> Result<(), ApplyError> {
    let stored = load(ctx).await?;
    for key in settings::SPECS {
        let wanted = match section.get(key.field) {
            Some(v) => Some((from_doc(key, v)?, Owner::Apply)),
            None => None,
        };
        let have = stored
            .0
            .get(key.var)
            .filter(|(_, o)| *o == Owner::Apply)
            .cloned();
        if wanted != have {
            let show = |r: &Option<(String, Owner)>| match r {
                Some((v, _)) => to_doc(key, v),
                None => Value::String("(not owned by apply)".into()),
            };
            return Err(ApplyError::stored_differs(
                path(key.field),
                &show(&wanted),
                &show(&have),
            ));
        }
    }
    Ok(())
}

/// The keys `apply` owns; `None` when it owns none. A value the document
/// cannot state (a hand edit the parser refuses) is left out.
pub(super) async fn export(ctx: &Ctx) -> Result<Option<TonSection>, ApplyError> {
    let stored = load(ctx).await?;
    let section: TonSection = settings::SPECS
        .iter()
        .filter_map(|key| {
            let (text, owner) = stored.0.get(key.var)?;
            if *owner != Owner::Apply {
                return None;
            }
            let value = to_doc(key, text);
            from_doc(key, &value).ok()?;
            Some((key.field.to_string(), value))
        })
        .collect();
    Ok((!section.is_empty()).then_some(section))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn section(v: Value) -> TonSection {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn values_are_canonical_and_typed() {
        let k = |f: &str| settings::by_field(f).unwrap();
        assert_eq!(from_doc(k("window_min"), &json!(45)).unwrap(), "45");
        assert_eq!(from_doc(k("window_min"), &json!(null)).unwrap(), "");
        assert!(from_doc(k("window_min"), &json!("45")).is_err());
        assert!(from_doc(k("window_min"), &json!(9)).is_err());
        assert_eq!(
            from_doc(k("underpay_tolerance_ton"), &json!("0.01")).unwrap(),
            "10000000"
        );
        let e = from_doc(k("underpay_tolerance_ton"), &json!("0.010")).unwrap_err();
        assert!(e.to_string().contains("\"0.01\""), "{e}");
        assert!(from_doc(k("rate_min_usd"), &json!("0.30")).is_err());
        assert!(from_doc(k("ladder"), &json!("free, 20")).is_err());
        assert_eq!(
            from_doc(k("ladder"), &json!("free,20,40")).unwrap(),
            "free,20,40"
        );
        assert!(from_doc(k("ladder"), &json!("")).is_err());
        assert!(from_doc(k("alert_email"), &json!(5)).is_err());
        assert_eq!(from_doc(k("debug_booking"), &json!(true)).unwrap(), "1");
        assert_eq!(from_doc(k("debug_booking"), &json!(false)).unwrap(), "0");
        assert_eq!(from_doc(k("debug_booking"), &json!(null)).unwrap(), "");
        for bad in [json!("on"), json!(1), json!("true")] {
            let e = from_doc(k("debug_booking"), &bad).unwrap_err();
            assert!(e.to_string().contains("true, false or null"), "{e}");
        }
        assert_eq!(to_doc(k("debug_booking"), "1"), json!(true));
        assert_eq!(to_doc(k("debug_booking"), "0"), json!(false));
        assert_eq!(to_doc(k("debug_booking"), ""), Value::Null);
        for f in [
            "window_min",
            "underpay_tolerance_ton",
            "ladder",
            "rate_max_jump_pct",
        ] {
            let text = from_doc(k(f), &to_doc(k(f), &k(f).default_text())).unwrap();
            assert_eq!(
                to_doc(k(f), &text),
                to_doc(k(f), &k(f).default_text()),
                "{f}"
            );
        }
    }

    #[test]
    fn keys_left_out_go_back_to_the_page() {
        let mut before = DbValues::default();
        before.set("CALRS_TON_WINDOW_MIN", "45".into(), Owner::Apply);
        before.set("CALRS_TON_GRACE_MIN", "".into(), Owner::Apply);
        before.set("CALRS_TON_FUSE_HOURS", "8".into(), Owner::Panel);
        let next = planned(&before, &section(json!({"fuse_hours": 6}))).unwrap();
        assert_eq!(next.owner("CALRS_TON_WINDOW_MIN"), Some(Owner::Panel));
        assert_eq!(next.value("CALRS_TON_WINDOW_MIN"), Some("45"));
        assert_eq!(next.owner("CALRS_TON_GRACE_MIN"), None);
        assert_eq!(next.value("CALRS_TON_FUSE_HOURS"), Some("6"));
        assert_eq!(next.owner("CALRS_TON_FUSE_HOURS"), Some(Owner::Apply));
        let e = planned(&before, &section(json!({"network": "mainnet"}))).unwrap_err();
        assert!(e.to_string().contains("unknown key"), "{e}");
        // Half a corridor is refused as a whole.
        let half = planned(&before, &section(json!({"rate_min_usd": "0.3"}))).unwrap();
        let e = validate(&half).unwrap_err();
        assert!(
            e.to_string().starts_with("invalid site.ton.rate_min_usd"),
            "{e}"
        );
    }
}
