# Admin Dashboard

The admin dashboard is available at `/dashboard/admin` for users with the `admin` role.

![Admin dashboard](images/admin.png)

## User management

Lists all registered users with:

- Name, email, username
- Role (admin/user)
- Status (enabled/disabled)
- Teams and groups

Actions per user:

- **Promote/Demote** — toggle admin role
- **Enable/Disable** — disabled users cannot log in or receive bookings
- **Impersonate** — view the dashboard as that user (for troubleshooting)

## Impersonation

Admins can impersonate any user to troubleshoot their view:

1. Click **Impersonate** next to a user in the admin panel
2. You are redirected to the dashboard, viewing it as that user
3. A yellow banner at the top shows who you're impersonating
4. Click **Stop impersonating** to return to your own view

Impersonation uses a separate `calrs_impersonate` cookie (24-hour TTL). The real admin session is preserved.

## Availability troubleshoot

The dashboard sidebar's **Troubleshoot** link opens a visual timeline at `/dashboard/troubleshoot?event_type=<slug>&date=<YYYY-MM-DD>` (pick the event type and the day on the page):

- Shows the candidate slots of one day (today by default), with arrows to the previous and next day
- Displays why each slot is blocked (calendar event name, existing booking, buffer overlap)
- Helps debug availability issues when users report incorrect free/busy status
- Times already gone on the day shown are marked **Already passed** (their own shade and legend entry, **Past**), not blamed on the minimum notice, and the date is shown in your dashboard language (ilia fork)

![Availability troubleshoot](images/troubleshoot.png)

## Authentication settings

- **Registration** — toggle open registration on/off
- **Allowed domains** — restrict registration to specific email domains (comma-separated) or allow any

## OIDC configuration

- **Enabled** — toggle SSO login on/off
- **Issuer URL** — your OIDC provider's base URL
- **Client ID** — the client ID registered with your provider
- **Client secret** — update the secret (current value is never displayed)
- **Auto-register** — automatically create users on first OIDC login

## Google OAuth2

Admin → Auth → Google OAuth2 holds the instance-wide Google client ID and secret used when users connect Google Calendar. The same credentials are used to attach a Google Meet conference to a confirmed booking when an event type uses the Google Meet location. There is no extra Meet admin form. See [Google Calendar](./google-calendar.md).

## Resources

The **Resources** card manages [shared bookable resources](./resources.md) (demo lab, meeting rooms): add a resource from its ICS feed URL, edit its optional CalDAV write-back settings and team allowlist, force a sync with **Sync now**, and verify write access with **Test write**. A failed feed sync is flagged here with the last error. See [Shared Resources](./resources.md) for details.

## SMS settings

The **SMS settings** card configures the instance's [SMS gateway](./sms.md): pick Twilio, GatewayAPI, seven.io, or a generic webhook, and the form relabels its fields for that vendor. The same card holds the daily message limit, today's usage and cost, and the policy controlling whether non-admins may enable SMS on their own event types.

**Test gateway** sends a real message to a number you give it, or verifies the credentials for free when you leave the field empty and the gateway supports it.

SMS spends real money on a public form, so the card also warns when SMS is configured without a captcha. See [SMS Notifications](./sms.md) for the setup steps and for keeping the bill bounded.

## SMTP status

Shows whether SMTP is configured and the current sender address. SMTP is configured via CLI (`calrs config smtp`) or by editing the database directly.

## TON payments (ilia fork)

**Admin → TON payments** (`/dashboard/admin/ton`, admins only) shows the state of TON payments and edits their tuning: the demand ladder and the free-step limit, invoice timing and renewals, the exchange-rate corridor and rules, and the alert email. The network, the wallet address, the indexer and the API keys are never edited here: they stay in the server's environment and systemd credentials, and the page only shows them (the address in both forms, the keys as set or not set). A field set in the environment or by `config apply` is read-only, with the reason. See [TON payments](ton.md).

## Debug booking (ilia fork)

**Admin → Debug booking** (`/dashboard/admin/debug-booking`) exists only while its switch is on: the Debug block at the top of **Admin → TON payments** (on/off, live at once, audited), unless the server's environment sets `CALRS_DEBUG_BOOKING` (which then wins and locks the switch) or `config apply` owns `site.ton.debug_booking`. Off, the address answers 404 and nothing links to it. The page is marked as debug (a striped banner and a hazard frame), and its bookings carry a TEST badge in the bookings list and on their payment page. It books a test meeting through the public booking path with the admin's own details filled in (the guest Telegram username from the admin's linked chat, else from the Debug block), books in the guest language the admin chooses (by default the admin's own interface language), routes the guest's Telegram messages to the admin's own linked chat (ticked by default; without a linked chat the page says so at the top and links to Settings → Telegram), can give the test booking's invoice its own fast or custom timing (minutes instead of the site's window and renewals, then automatic cancellation) and cancels all of the admin's test bookings at once. See [TON payments](ton.md#debug-booking-ilia-fork).
