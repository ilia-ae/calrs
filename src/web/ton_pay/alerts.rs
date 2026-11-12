//! The owner's TON emails: the [`AlertSink`] `calrs serve` installs. The
//! watcher claims each alert before handing it here, so every one goes out
//! once; sending runs in the background (`spawn_emails`), never inside a
//! watcher step.
//!
//! Who reads what: payment and invoice alerts go to the owner of the event
//! type (in their saved language and timezone); health alerts, and payments
//! that match no booking, to the alert email ([`alert_email`]: the
//! environment's `CALRS_TON_ALERT_EMAIL`, else the stored one) or else
//! every enabled admin. No email carries the sender's on-chain comment.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use super::{parse_stamp, AlertSink, TonAlert, TARGET};
use crate::email::TonOwnerMail;
use crate::ton::amount::{format_ton, format_usd_cents, from_i64};
use crate::ton::link::tonviewer_tx;
use crate::ton::Network;

pub(crate) struct EmailSink {
    pool: SqlitePool,
    secret_key: [u8; 32],
}

impl EmailSink {
    pub(crate) fn new(pool: SqlitePool, secret_key: [u8; 32]) -> Self {
        Self { pool, secret_key }
    }
}

#[async_trait]
impl AlertSink for EmailSink {
    async fn deliver(&self, alert: TonAlert) {
        let (pool, key) = (self.pool.clone(), self.secret_key);
        crate::web::spawn_emails(async move {
            deliver_now(&pool, &key, alert).await;
        });
    }
}

/// A recipient: (name, email, language).
type Recipient = (String, String, String);

/// Where health alerts and unmatched payments go.
pub(crate) async fn operators(pool: &SqlitePool) -> Vec<Recipient> {
    let lang_of = |l: Option<String>| crate::email::supported_language(l).unwrap_or("en".into());
    let admins: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT name, email, language FROM users WHERE role = 'admin' AND enabled = 1 ORDER BY created_at, id",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    let configured = alert_email(pool).await;
    match configured {
        Some(email) => {
            let lang = admins
                .first()
                .map(|(_, _, l)| lang_of(l.clone()))
                .unwrap_or_else(|| "en".into());
            vec![(String::new(), email, lang)]
        }
        None => admins
            .into_iter()
            .map(|(name, email, l)| (name, email, lang_of(l)))
            .collect(),
    }
}

/// ilia fork: the alert email in effect, through the settings resolver:
/// `CALRS_TON_ALERT_EMAIL` from the environment, else the stored one (the
/// admin page, `config apply`), else none. Read at each alert, so a saved
/// change applies to the next one, with payments on or off. An invalid
/// value (only a hand edit can store one) counts as none: the admins get
/// the alert instead.
pub(crate) async fn alert_email(pool: &SqlitePool) -> Option<String> {
    alert_email_with(pool, &crate::ton::settings::process_env).await
}

pub(crate) async fn alert_email_with(
    pool: &SqlitePool,
    env: &(dyn Fn(&str) -> Option<String> + Sync),
) -> Option<String> {
    use crate::ton::config::{Vars, ALERT_EMAIL};
    let db = match pool.acquire().await {
        Ok(mut conn) => super::store::load_settings(&mut conn).await,
        Err(e) => Err(e),
    }
    .map_err(|e| tracing::error!(target: TARGET, error = %e, "cannot read the TON settings"))
    .unwrap_or_default();
    Vars(|k: &str| crate::ton::settings::lookup(env, &db, k))
        .email(ALERT_EMAIL)
        .map_err(|e| tracing::error!(target: TARGET, error = %e, "the TON alert email is invalid"))
        .ok()
        .flatten()
}

#[derive(sqlx::FromRow)]
struct InvoiceMailRow {
    event_title: String,
    start_at_utc: String,
    amount_nano: i64,
    paid_nano: i64,
    excess_nano: i64,
    currency: String,
    price_usd_cents: i64,
    rate_usd_micro: i64,
    network: String,
    guest_name: Option<String>,
    guest_email: Option<String>,
    start_at: Option<String>,
    time_version: Option<i64>,
    owner_name: Option<String>,
    owner_email: Option<String>,
    owner_language: Option<String>,
    owner_timezone: Option<String>,
}

/// "12.3412" from micro-USD (4 decimals, as the payment page shows it).
pub(crate) fn rate_display(micro: i64) -> String {
    let m = u64::try_from(micro).unwrap_or(0);
    format!("{}.{:04}", m / 1_000_000, (m % 1_000_000) / 100)
}

/// The long date and "HH:MM (Zone)" of `at` in `timezone`, in `lang`.
pub(crate) fn date_and_time(lang: &str, at: DateTime<Utc>, timezone: &str) -> (String, String) {
    let tz = timezone.parse::<chrono_tz::Tz>().unwrap_or(chrono_tz::UTC);
    let local = at.with_timezone(&tz);
    (
        crate::i18n::format_long_date(local.date_naive(), lang),
        format!("{} ({})", local.format("%H:%M"), tz.name()),
    )
}

/// The owner's email about one invoice, and the recipient it goes to.
pub(crate) async fn invoice_mail(pool: &SqlitePool, invoice_id: &str) -> Option<TonOwnerMail> {
    let r: InvoiceMailRow = sqlx::query_as(
        "SELECT i.event_title, i.start_at_utc, i.amount_nano, i.paid_nano, i.excess_nano,
                i.currency, i.price_usd_cents, i.rate_usd_micro, i.network,
                b.guest_name, b.guest_email, b.start_at, b.time_version,
                u.name AS owner_name, COALESCE(u.booking_email, u.email) AS owner_email,
                u.language AS owner_language, u.timezone AS owner_timezone
         FROM ton_invoices i
         LEFT JOIN bookings b ON b.id = i.booking_id
         LEFT JOIN event_types et ON et.id = i.event_type_id
         LEFT JOIN accounts a ON a.id = et.account_id
         LEFT JOIN users u ON u.id = a.user_id
         WHERE i.id = ?",
    )
    .bind(invoice_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;
    let (to_name, to_email, lang) = match (r.owner_email.clone(), r.owner_name.clone()) {
        (Some(email), name) => (
            name.unwrap_or_default(),
            email,
            crate::email::supported_language(r.owner_language.clone()).unwrap_or("en".into()),
        ),
        // The event type is gone (its bookings with it): the operators.
        (None, _) => operators(pool).await.into_iter().next()?,
    };
    let start = match (r.time_version, r.start_at.as_deref()) {
        (Some(1), Some(s)) => parse_stamp(s),
        _ => None,
    }
    .or_else(|| parse_stamp(&r.start_at_utc))
    .unwrap_or_else(Utc::now);
    let (date, time) = date_and_time(&lang, start, r.owner_timezone.as_deref().unwrap_or("UTC"));
    let network = Network::parse(&r.network).unwrap_or(Network::Mainnet);
    let usd = r.currency == "USD";
    let txs: Vec<String> = sqlx::query_scalar(
        "SELECT tx_hex FROM ton_payments WHERE invoice_id = ? AND outcome IN ('credited', 'overpaid', 'late') ORDER BY lt",
    )
    .bind(invoice_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    Some(TonOwnerMail {
        lang,
        to_name,
        to_email,
        event_title: r.event_title,
        date,
        time,
        guest_name: r.guest_name.unwrap_or_default(),
        guest_email: r.guest_email.unwrap_or_default(),
        amount_ton: format_ton(from_i64(r.amount_nano)),
        // A TON price has no dollar value or rate: both stay empty.
        price_usd: if usd {
            format_usd_cents(from_i64(r.price_usd_cents))
        } else {
            String::new()
        },
        rate_usd: if usd {
            rate_display(r.rate_usd_micro)
        } else {
            String::new()
        },
        paid_ton: format_ton(from_i64(r.paid_nano)),
        excess_ton: if r.excess_nano > 0 {
            format_ton(from_i64(r.excess_nano))
        } else {
            String::new()
        },
        value_ton: String::new(),
        txs: txs
            .into_iter()
            .map(|h| {
                let url = tonviewer_tx(network, &h);
                (h, url)
            })
            .collect(),
        sender: None,
        sender_is_wallet: None,
        dashboard_url: dashboard_url(),
    })
}

fn dashboard_url() -> Option<String> {
    crate::settings::base_url().map(|b| format!("{}/dashboard/bookings", b.trim_end_matches('/')))
}

/// The email about one recorded transfer. Never its comment.
async fn payment_mail(pool: &SqlitePool, tx_hex: &str) -> Vec<TonOwnerMail> {
    let row: Option<(String, i64, Option<String>, Option<i64>, Option<String>)> = sqlx::query_as(
        "SELECT network, value_nano, source_friendly, source_is_wallet, invoice_id
         FROM ton_payments WHERE tx_hex = ?",
    )
    .bind(tx_hex)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    let Some((network, value, sender, is_wallet, invoice_id)) = row else {
        return vec![];
    };
    let network = Network::parse(&network).unwrap_or(Network::Mainnet);
    let tx = (tx_hex.to_string(), tonviewer_tx(network, tx_hex));
    let with_transfer = |mut m: TonOwnerMail| {
        m.value_ton = format_ton(from_i64(value));
        m.sender = sender.clone();
        m.sender_is_wallet = is_wallet.map(|v| v != 0);
        m.txs = vec![tx.clone()];
        m
    };
    if let Some(id) = invoice_id {
        if let Some(m) = invoice_mail(pool, &id).await {
            return vec![with_transfer(m)];
        }
    }
    operators(pool)
        .await
        .into_iter()
        .map(|(to_name, to_email, lang)| {
            with_transfer(TonOwnerMail {
                lang,
                to_name,
                to_email,
                dashboard_url: dashboard_url(),
                ..Default::default()
            })
        })
        .collect()
}

pub(crate) async fn deliver_now(pool: &SqlitePool, secret_key: &[u8; 32], alert: TonAlert) {
    let smtp = match crate::email::load_smtp_config(pool, secret_key).await {
        Ok(Some(c)) => c,
        _ => {
            tracing::warn!(target: TARGET, ?alert, "ton alert not emailed: SMTP is not configured");
            return;
        }
    };
    let result = match &alert {
        TonAlert::Payment { tx_hex, kind } => {
            let mut last = Ok(());
            for mail in payment_mail(pool, tx_hex).await {
                last = crate::email::send_host_ton_payment_alert(&smtp, &mail, kind).await;
            }
            last
        }
        TonAlert::InvoiceProblem {
            invoice_id,
            problem,
        } => match invoice_mail(pool, invoice_id).await {
            Some(mail) => crate::email::send_host_ton_invoice_problem(&smtp, &mail, problem).await,
            None => Ok(()),
        },
        TonAlert::PaymentReceived { invoice_id } => match invoice_mail(pool, invoice_id).await {
            Some(mail) => crate::email::send_host_ton_payment_received(&smtp, &mail).await,
            None => Ok(()),
        },
        TonAlert::Renewed { invoice_id } => {
            super::page::email_renewal(pool, secret_key, &smtp, invoice_id).await
        }
        TonAlert::Health { kind, detail } => {
            let mut last = Ok(());
            for (name, email, lang) in operators(pool).await {
                last =
                    crate::email::send_host_ton_health(&smtp, &name, &email, &lang, kind, detail)
                        .await;
            }
            last
        }
    };
    if let Err(e) = result {
        tracing::error!(target: TARGET, error = %e, ?alert, "ton alert email failed");
    }
}
