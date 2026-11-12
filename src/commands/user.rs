use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use sqlx::SqlitePool;
use std::path::Path;
use tabled::{Table, Tabled};
use uuid::Uuid;

use crate::auth;
use crate::models::User;
use crate::utils::{prompt, read_secret};

#[derive(Debug, Subcommand)]
pub enum UserCommands {
    /// Create a new user
    Create {
        /// User's email
        #[arg(long)]
        email: Option<String>,
        /// User's name
        #[arg(long)]
        name: Option<String>,
        /// Grant admin role
        #[arg(long)]
        admin: bool,
        /// Read the password from stdin instead of the terminal (for scripts).
        /// Needs --email and --name, since stdin then holds the password.
        #[arg(long)]
        password_stdin: bool,
        /// Profile timezone, an IANA name such as Europe/Paris (default: UTC)
        #[arg(long)]
        timezone: Option<String>,
    },
    /// Reset local MFA (authenticator app, passkeys, recovery codes) and revoke sessions (server-operator recovery)
    ResetMfa { email: String },
    /// List all users
    List,
    /// Disable a user
    Disable {
        /// User email
        email: String,
    },
    /// Enable a user
    Enable {
        /// User email
        email: String,
    },
    /// Promote a user to admin
    Promote {
        /// User email
        email: String,
    },
    /// Demote an admin to regular user
    Demote {
        /// User email
        email: String,
    },
    /// Set or reset a user's password
    SetPassword {
        /// User email
        email: String,
        /// Read the new password from stdin instead of the terminal (for scripts)
        #[arg(long)]
        password_stdin: bool,
    },
    /// Set a user's profile timezone (the one Settings → Profile shows)
    SetTimezone {
        /// User email
        email: String,
        /// IANA timezone name, such as Europe/Paris
        timezone: String,
    },
    /// Permanently delete a user and all data uniquely owned by them
    Delete {
        /// User email
        email: String,
        /// Skip the interactive confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },
}

pub async fn run(pool: &SqlitePool, data_dir: &Path, cmd: UserCommands) -> Result<()> {
    match cmd {
        UserCommands::ResetMfa { email } => {
            crate::mfa::reset(pool, &email).await?;
            println!("MFA reset for {email}; all sessions revoked.");
        }
        UserCommands::Create {
            email,
            name,
            admin,
            password_stdin,
            timezone,
        } => {
            if password_stdin && (email.is_none() || name.is_none()) {
                anyhow::bail!(
                    "--password-stdin needs --email and --name: stdin holds the password"
                );
            }
            // Checked before any prompt, so a typo costs no password entry.
            let timezone = timezone.as_deref().map(parse_timezone).transpose()?;
            let email = email.unwrap_or_else(|| prompt("Email"));
            let name = name.unwrap_or_else(|| prompt("Name"));
            let password = read_secret("Password", password_stdin)?;
            let role =
                create_user(pool, &email, &name, &password, admin, timezone.as_deref()).await?;

            println!(
                "{} User created: {} <{}> (role: {})",
                "✓".green(),
                name,
                email,
                role
            );

            if role == "admin" && !admin {
                println!(
                    "{}",
                    "  First user, automatically granted admin role.".dimmed()
                );
            }
        }
        UserCommands::List => {
            let users: Vec<User> = sqlx::query_as("SELECT * FROM users ORDER BY created_at")
                .fetch_all(pool)
                .await?;

            if users.is_empty() {
                println!("No users. Create one with `calrs user create`.");
                return Ok(());
            }

            #[derive(Tabled)]
            struct UserRow {
                email: String,
                name: String,
                role: String,
                provider: String,
                enabled: String,
                created: String,
            }

            let rows: Vec<UserRow> = users
                .iter()
                .map(|u| UserRow {
                    email: u.email.clone(),
                    name: u.name.clone(),
                    role: u.role.clone(),
                    provider: u.auth_provider.clone(),
                    enabled: if u.enabled {
                        "✓".to_string()
                    } else {
                        "✗".to_string()
                    },
                    created: u.created_at.clone(),
                })
                .collect();

            println!("{}", Table::new(rows));
        }
        UserCommands::Disable { email } => {
            let id = user_id_by_email(pool, &email).await?;
            match auth::set_user_enabled(pool, &id, false).await? {
                auth::AdminChange::NotFound => {
                    println!("{} User not found: {}", "✗".red(), email);
                }
                auth::AdminChange::LastAdmin => {
                    println!(
                        "{} Cannot disable the last enabled admin. Promote or enable another admin first.",
                        "✗".red()
                    );
                }
                auth::AdminChange::Changed => {
                    println!("{} User disabled: {}", "✓".green(), email);
                }
            }
        }
        UserCommands::Enable { email } => {
            let result = sqlx::query(
                "UPDATE users SET enabled = 1, updated_at = datetime('now') WHERE email = ?",
            )
            .bind(&email)
            .execute(pool)
            .await?;
            if result.rows_affected() == 0 {
                println!("{} User not found: {}", "✗".red(), email);
            } else {
                println!("{} User enabled: {}", "✓".green(), email);
            }
        }
        UserCommands::Promote { email } => {
            let result = sqlx::query(
                "UPDATE users SET role = 'admin', updated_at = datetime('now') WHERE email = ?",
            )
            .bind(&email)
            .execute(pool)
            .await?;
            if result.rows_affected() == 0 {
                println!("{} User not found: {}", "✗".red(), email);
            } else {
                println!("{} User promoted to admin: {}", "✓".green(), email);
            }
        }
        UserCommands::Demote { email } => {
            // Refuses to demote the last enabled admin, atomically.
            let id = user_id_by_email(pool, &email).await?;
            match auth::set_user_admin(pool, &id, false).await? {
                auth::AdminChange::NotFound => {
                    println!("{} User not found: {}", "✗".red(), email);
                }
                auth::AdminChange::LastAdmin => {
                    println!(
                        "{} Cannot demote the last admin. Promote another user first.",
                        "✗".red()
                    );
                }
                auth::AdminChange::Changed => {
                    println!("{} User demoted to user: {}", "✓".green(), email);
                }
            }
        }
        UserCommands::SetPassword {
            email,
            password_stdin,
        } => {
            let existing: Option<(String,)> =
                sqlx::query_as("SELECT id FROM users WHERE email = ?")
                    .bind(&email)
                    .fetch_optional(pool)
                    .await?;

            if existing.is_none() {
                println!("{} User not found: {}", "✗".red(), email);
                return Ok(());
            }

            let password = read_secret("New password", password_stdin)?;
            if password.len() < 12 {
                anyhow::bail!("Password must be at least 12 characters");
            }

            let password_hash = auth::hash_password(&password)?;
            // A reset password usually means the old one leaked: sign out
            // every session it opened, in the same transaction.
            let mut tx = pool.begin().await?;
            sqlx::query(
                "UPDATE users SET password_hash = ?, updated_at = datetime('now') WHERE email = ?",
            )
            .bind(&password_hash)
            .bind(&email)
            .execute(&mut *tx)
            .await?;
            if let Some((id,)) = &existing {
                crate::mfa::revoke_access(&mut tx, id).await?;
            }
            tx.commit().await?;

            println!(
                "{} Password updated for {}; all sessions signed out.",
                "✓".green(),
                email
            );
        }
        UserCommands::SetTimezone { email, timezone } => {
            let tz = parse_timezone(&timezone)?;
            match set_timezone(pool, &email, &tz).await? {
                TimezoneChange::NotFound => anyhow::bail!("User not found: {email}"),
                TimezoneChange::Unchanged => {
                    println!("Timezone of {email} is already {tz}; nothing to do.");
                }
                TimezoneChange::Changed { from } => {
                    println!(
                        "{} Timezone of {} set to {} (was {}).",
                        "✓".green(),
                        email,
                        tz,
                        from
                    );
                }
            }
        }
        UserCommands::Delete { email, yes } => {
            let user: Option<(String, String, String, String)> =
                sqlx::query_as("SELECT id, name, role, auth_provider FROM users WHERE email = ?")
                    .bind(&email)
                    .fetch_optional(pool)
                    .await?;
            let (target_id, target_name, _role, auth_provider) = match user {
                Some(u) => u,
                None => {
                    println!("{} User not found: {}", "✗".red(), email);
                    return Ok(());
                }
            };

            if !yes {
                println!("{} About to permanently delete:", "⚠".yellow());
                println!("    {} <{}>", target_name, email);
                println!(
                    "{}",
                    "  This removes their user record, scheduling account, calendar sources,"
                        .dimmed()
                );
                println!(
                    "{}",
                    "  event types, and all data uniquely owned by them.".dimmed()
                );
                if auth_provider == "oidc" {
                    println!(
                        "{}",
                        "  This is an OIDC/SSO user; if auto-register is enabled they will be"
                            .dimmed()
                    );
                    println!("{}", "  re-created on their next login.".dimmed());
                }
                let confirm = prompt("Type 'delete' to confirm");
                if confirm.trim() != "delete" {
                    println!("{} Aborted.", "✗".red());
                    return Ok(());
                }
            }

            let avatars_dir = data_dir.join("avatars");
            match auth::delete_user(pool, &target_id, None, Some(&avatars_dir)).await {
                Ok(()) => {
                    println!("{} User deleted: {}", "✓".green(), email);
                }
                Err(e) => {
                    println!("{} {}", "✗".red(), e);
                }
            }
        }
    }

    Ok(())
}

/// An IANA timezone name as the web settings accept it (`chrono_tz`), in
/// its canonical spelling; anything else is an error naming the value.
fn parse_timezone(input: &str) -> Result<String> {
    let input = input.trim();
    input
        .parse::<chrono_tz::Tz>()
        .map(|tz| tz.name().to_string())
        .map_err(|_| {
            anyhow::anyhow!("Invalid timezone: {input:?} (an IANA name such as Europe/Paris)")
        })
}

#[derive(Debug, PartialEq)]
enum TimezoneChange {
    NotFound,
    Unchanged,
    Changed { from: String },
}

/// Store `tz` (already validated) as the profile timezone of the user with
/// this email: the `users.timezone` column the profile form and the
/// timezone banner write. Nothing is written when it is already set.
async fn set_timezone(pool: &SqlitePool, email: &str, tz: &str) -> Result<TimezoneChange> {
    let current: Option<String> = sqlx::query_scalar("SELECT timezone FROM users WHERE email = ?")
        .bind(email)
        .fetch_optional(pool)
        .await?;
    let Some(current) = current else {
        return Ok(TimezoneChange::NotFound);
    };
    if current == tz {
        return Ok(TimezoneChange::Unchanged);
    }
    sqlx::query("UPDATE users SET timezone = ?, updated_at = datetime('now') WHERE email = ?")
        .bind(tz)
        .bind(email)
        .execute(pool)
        .await?;
    Ok(TimezoneChange::Changed { from: current })
}

/// Store a new local user and their scheduling account in one transaction,
/// with the profile timezone when given (`timezone` already validated by
/// `parse_timezone`; `None` keeps the column default, UTC). Returns the role
/// the user got: the first user becomes admin even without `admin`.
async fn create_user(
    pool: &SqlitePool,
    email: &str,
    name: &str,
    password: &str,
    admin: bool,
    timezone: Option<&str>,
) -> Result<String> {
    if password.len() < 12 {
        anyhow::bail!("Password must be at least 12 characters");
    }

    let password_hash = auth::hash_password(password)?;
    let user_id = Uuid::new_v4().to_string();
    let username = auth::generate_username(pool, email).await?;

    let mut tx = pool.begin().await?;
    let role = auth::create_local_user(
        &mut *tx,
        &user_id,
        email,
        name,
        &password_hash,
        &username,
        admin,
    )
    .await?;
    auth::create_user_account(&mut tx, &user_id, name, email).await?;
    if let Some(tz) = timezone {
        sqlx::query("UPDATE users SET timezone = ? WHERE id = ?")
            .bind(tz)
            .bind(&user_id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(role)
}

/// The id of the user with this email, or an empty string when there is
/// none (the guarded updates then report `NotFound`).
async fn user_id_by_email(pool: &SqlitePool, email: &str) -> Result<String> {
    let id: Option<String> = sqlx::query_scalar("SELECT id FROM users WHERE email = ?")
        .bind(email)
        .fetch_optional(pool)
        .await?;
    Ok(id.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;

    async fn setup_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                SqliteConnectOptions::from_str("sqlite::memory:")
                    .unwrap()
                    .foreign_keys(true),
            )
            .await
            .unwrap();
        crate::db::migrate(&pool).await.unwrap();
        pool
    }

    /// Insert a user directly (bypasses interactive prompts in UserCommands::Create)
    async fn insert_user(pool: &SqlitePool, email: &str, name: &str, role: &str) -> String {
        let user_id = Uuid::new_v4().to_string();
        let password_hash = crate::auth::hash_password("testpass123").unwrap();
        let username = crate::auth::generate_username(pool, email).await.unwrap();
        sqlx::query(
            "INSERT INTO users (id, email, name, timezone, password_hash, role, auth_provider, username, enabled)
             VALUES (?, ?, ?, 'UTC', ?, ?, 'local', ?, 1)",
        )
        .bind(&user_id)
        .bind(email)
        .bind(name)
        .bind(&password_hash)
        .bind(role)
        .bind(&username)
        .execute(pool)
        .await
        .unwrap();

        // Create a linked account
        let account_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO accounts (id, name, email, timezone, user_id) VALUES (?, ?, ?, 'UTC', ?)",
        )
        .bind(&account_id)
        .bind(name)
        .bind(email)
        .bind(&user_id)
        .execute(pool)
        .await
        .unwrap();

        user_id
    }

    /// Wrapper around `auth::create_local_user` that handles the bookkeeping
    /// the production handler also handles (UUID, password hash, username).
    /// Calls the same helper the `Create` command uses, so the atomic CASE
    /// in the production code path is what's actually under test.
    async fn create_user_returning_role(pool: &SqlitePool, email: &str, admin: bool) -> String {
        let user_id = Uuid::new_v4().to_string();
        let password_hash = crate::auth::hash_password("testpass123").unwrap();
        let username = crate::auth::generate_username(pool, email).await.unwrap();
        crate::auth::create_local_user(
            pool,
            &user_id,
            email,
            "Test",
            &password_hash,
            &username,
            admin,
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn first_user_create_gets_admin_role_atomically() {
        let pool = setup_db().await;
        let role = create_user_returning_role(&pool, "first@test.com", false).await;
        assert_eq!(role, "admin");
    }

    #[tokio::test]
    async fn subsequent_user_create_gets_user_role() {
        let pool = setup_db().await;
        insert_user(&pool, "first@test.com", "First", "admin").await;
        let role = create_user_returning_role(&pool, "second@test.com", false).await;
        assert_eq!(role, "user");
    }

    #[tokio::test]
    async fn explicit_admin_flag_overrides_user_role() {
        // `--admin` flag set + existing users: the operator's choice wins.
        let pool = setup_db().await;
        insert_user(&pool, "first@test.com", "First", "admin").await;
        let role = create_user_returning_role(&pool, "second@test.com", true).await;
        assert_eq!(role, "admin");
    }

    #[tokio::test]
    async fn test_list_users_empty() {
        let pool = setup_db().await;
        let result = run(&pool, &std::env::temp_dir(), UserCommands::List).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_users_with_data() {
        let pool = setup_db().await;
        insert_user(&pool, "alice@test.com", "Alice", "admin").await;
        insert_user(&pool, "bob@test.com", "Bob", "user").await;

        let result = run(&pool, &std::env::temp_dir(), UserCommands::List).await;
        assert!(result.is_ok());

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 2);
    }

    #[tokio::test]
    async fn test_promote_user() {
        let pool = setup_db().await;
        insert_user(&pool, "alice@test.com", "Alice", "admin").await;
        insert_user(&pool, "bob@test.com", "Bob", "user").await;

        let result = run(
            &pool,
            &std::env::temp_dir(),
            UserCommands::Promote {
                email: "bob@test.com".to_string(),
            },
        )
        .await;
        assert!(result.is_ok());

        let role: (String,) = sqlx::query_as("SELECT role FROM users WHERE email = 'bob@test.com'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(role.0, "admin");
    }

    #[tokio::test]
    async fn test_demote_user() {
        let pool = setup_db().await;
        insert_user(&pool, "alice@test.com", "Alice", "admin").await;
        insert_user(&pool, "bob@test.com", "Bob", "admin").await;

        let result = run(
            &pool,
            &std::env::temp_dir(),
            UserCommands::Demote {
                email: "bob@test.com".to_string(),
            },
        )
        .await;
        assert!(result.is_ok());

        let role: (String,) = sqlx::query_as("SELECT role FROM users WHERE email = 'bob@test.com'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(role.0, "user");
    }

    #[tokio::test]
    async fn test_demote_last_admin_prevented() {
        let pool = setup_db().await;
        insert_user(&pool, "alice@test.com", "Alice", "admin").await;

        // Attempting to demote the only admin should not change the role
        let result = run(
            &pool,
            &std::env::temp_dir(),
            UserCommands::Demote {
                email: "alice@test.com".to_string(),
            },
        )
        .await;
        assert!(result.is_ok());

        let role: (String,) =
            sqlx::query_as("SELECT role FROM users WHERE email = 'alice@test.com'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(role.0, "admin", "Last admin should not be demoted");
    }

    #[tokio::test]
    async fn test_disable_user() {
        let pool = setup_db().await;
        insert_user(&pool, "bob@test.com", "Bob", "user").await;

        let result = run(
            &pool,
            &std::env::temp_dir(),
            UserCommands::Disable {
                email: "bob@test.com".to_string(),
            },
        )
        .await;
        assert!(result.is_ok());

        let enabled: (bool,) =
            sqlx::query_as("SELECT enabled FROM users WHERE email = 'bob@test.com'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(!enabled.0, "User should be disabled");
    }

    #[tokio::test]
    async fn test_enable_user() {
        let pool = setup_db().await;
        insert_user(&pool, "bob@test.com", "Bob", "user").await;

        // Disable first
        run(
            &pool,
            &std::env::temp_dir(),
            UserCommands::Disable {
                email: "bob@test.com".to_string(),
            },
        )
        .await
        .unwrap();

        // Then re-enable
        let result = run(
            &pool,
            &std::env::temp_dir(),
            UserCommands::Enable {
                email: "bob@test.com".to_string(),
            },
        )
        .await;
        assert!(result.is_ok());

        let enabled: (bool,) =
            sqlx::query_as("SELECT enabled FROM users WHERE email = 'bob@test.com'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(enabled.0, "User should be enabled");
    }

    #[tokio::test]
    async fn test_disable_nonexistent_user() {
        let pool = setup_db().await;
        // Should succeed (no error), just print "not found"
        let result = run(
            &pool,
            &std::env::temp_dir(),
            UserCommands::Disable {
                email: "nobody@test.com".to_string(),
            },
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_promote_nonexistent_user() {
        let pool = setup_db().await;
        let result = run(
            &pool,
            &std::env::temp_dir(),
            UserCommands::Promote {
                email: "nobody@test.com".to_string(),
            },
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_user_with_yes_flag() {
        let pool = setup_db().await;
        // Two admins so the LastAdmin guard doesn't fire on Bob.
        insert_user(&pool, "alice@test.com", "Alice", "admin").await;
        insert_user(&pool, "bob@test.com", "Bob", "admin").await;

        let result = run(
            &pool,
            &std::env::temp_dir(),
            UserCommands::Delete {
                email: "bob@test.com".to_string(),
                yes: true,
            },
        )
        .await;
        assert!(result.is_ok());

        let bob_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = 'bob@test.com'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(bob_count.0, 0, "Bob should be gone");
    }

    #[tokio::test]
    async fn test_delete_nonexistent_user() {
        let pool = setup_db().await;
        // Should not error; just print "not found".
        let result = run(
            &pool,
            &std::env::temp_dir(),
            UserCommands::Delete {
                email: "ghost@test.com".to_string(),
                yes: true,
            },
        )
        .await;
        assert!(result.is_ok());
    }

    async fn timezone_of(pool: &SqlitePool, email: &str) -> String {
        sqlx::query_scalar("SELECT timezone FROM users WHERE email = ?")
            .bind(email)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    fn set_tz_cmd(email: &str, timezone: &str) -> UserCommands {
        UserCommands::SetTimezone {
            email: email.to_string(),
            timezone: timezone.to_string(),
        }
    }

    #[test]
    fn parse_timezone_accepts_iana_names_only() {
        assert_eq!(parse_timezone("Asia/Almaty").unwrap(), "Asia/Almaty");
        assert_eq!(parse_timezone(" Europe/Paris\n").unwrap(), "Europe/Paris");
        assert_eq!(parse_timezone("UTC").unwrap(), "UTC");
        for bad in [
            "",
            "  ",
            "Mars/Olympus",
            "Paris",
            "+05:00",
            "Asia/Almaty; DROP",
        ] {
            let err = parse_timezone(bad).unwrap_err().to_string();
            assert!(err.contains("Invalid timezone"), "{bad:?}: {err}");
        }
    }

    #[tokio::test]
    async fn set_timezone_writes_the_profile_column() {
        let pool = setup_db().await;
        insert_user(&pool, "alice@test.com", "Alice", "admin").await;
        insert_user(&pool, "bob@test.com", "Bob", "user").await;

        run(
            &pool,
            &std::env::temp_dir(),
            set_tz_cmd("alice@test.com", " Asia/Almaty "),
        )
        .await
        .unwrap();

        assert_eq!(timezone_of(&pool, "alice@test.com").await, "Asia/Almaty");
        assert_eq!(
            timezone_of(&pool, "bob@test.com").await,
            "UTC",
            "only Alice"
        );
    }

    #[tokio::test]
    async fn set_timezone_reports_no_change_when_already_set() {
        let pool = setup_db().await;
        insert_user(&pool, "alice@test.com", "Alice", "admin").await;

        assert_eq!(
            set_timezone(&pool, "alice@test.com", "Asia/Almaty")
                .await
                .unwrap(),
            TimezoneChange::Changed {
                from: "UTC".to_string()
            }
        );
        assert_eq!(
            set_timezone(&pool, "alice@test.com", "Asia/Almaty")
                .await
                .unwrap(),
            TimezoneChange::Unchanged
        );
        // The command itself succeeds on a no-op, so scripts can rerun it.
        run(
            &pool,
            &std::env::temp_dir(),
            set_tz_cmd("alice@test.com", "Asia/Almaty"),
        )
        .await
        .unwrap();
        assert_eq!(timezone_of(&pool, "alice@test.com").await, "Asia/Almaty");
    }

    #[tokio::test]
    async fn set_timezone_refuses_an_invalid_name_or_unknown_user() {
        let pool = setup_db().await;
        insert_user(&pool, "alice@test.com", "Alice", "admin").await;

        let err = run(
            &pool,
            &std::env::temp_dir(),
            set_tz_cmd("alice@test.com", "Mars/Olympus"),
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("Invalid timezone"), "{err}");
        assert_eq!(timezone_of(&pool, "alice@test.com").await, "UTC");

        // Unlike promote/enable, an unknown user is an error: a script
        // setting a timezone must not pass silently.
        let err = run(
            &pool,
            &std::env::temp_dir(),
            set_tz_cmd("ghost@test.com", "Asia/Almaty"),
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("User not found"), "{err}");
        assert_eq!(
            set_timezone(&pool, "ghost@test.com", "Asia/Almaty")
                .await
                .unwrap(),
            TimezoneChange::NotFound
        );
    }

    #[tokio::test]
    async fn create_user_stores_the_given_timezone() {
        let pool = setup_db().await;
        let role = create_user(
            &pool,
            "alice@test.com",
            "Alice",
            "long-enough-password",
            true,
            Some("Asia/Almaty"),
        )
        .await
        .unwrap();
        assert_eq!(role, "admin");
        assert_eq!(timezone_of(&pool, "alice@test.com").await, "Asia/Almaty");
        // The scheduling account is created in the same transaction.
        let accounts: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM accounts a JOIN users u ON u.id = a.user_id
             WHERE u.email = 'alice@test.com'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(accounts, 1);
    }

    #[tokio::test]
    async fn create_user_without_a_timezone_keeps_utc() {
        let pool = setup_db().await;
        create_user(
            &pool,
            "alice@test.com",
            "Alice",
            "long-enough-password",
            false,
            None,
        )
        .await
        .unwrap();
        assert_eq!(timezone_of(&pool, "alice@test.com").await, "UTC");
    }

    #[tokio::test]
    async fn create_user_refuses_a_short_password_and_writes_nothing() {
        let pool = setup_db().await;
        let err = create_user(
            &pool,
            "alice@test.com",
            "Alice",
            "short",
            true,
            Some("Asia/Almaty"),
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("at least 12"), "{err}");
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn create_refuses_an_invalid_timezone_before_reading_the_password() {
        let pool = setup_db().await;
        // stdin is never read: the timezone is checked first.
        let err = run(
            &pool,
            &std::env::temp_dir(),
            UserCommands::Create {
                email: Some("alice@test.com".to_string()),
                name: Some("Alice".to_string()),
                admin: true,
                password_stdin: true,
                timezone: Some("Mars/Olympus".to_string()),
            },
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("Invalid timezone"), "{err}");
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_disable_invalidates_sessions() {
        let pool = setup_db().await;
        let user_id = insert_user(&pool, "bob@test.com", "Bob", "user").await;

        // Create a session for Bob
        let session_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO sessions (id, user_id, expires_at) VALUES (?, ?, datetime('now', '+30 days'))",
        )
        .bind(&session_id)
        .bind(&user_id)
        .execute(&pool)
        .await
        .unwrap();

        // Disable Bob
        run(
            &pool,
            &std::env::temp_dir(),
            UserCommands::Disable {
                email: "bob@test.com".to_string(),
            },
        )
        .await
        .unwrap();

        // Session should be deleted
        let session_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE user_id = ?")
                .bind(&user_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(session_count.0, 0, "Sessions should be deleted on disable");
    }
}
