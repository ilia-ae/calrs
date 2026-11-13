# TON Payments (ilia fork)

The ilia fork can ask a guest to pay for a booking in TON before it is confirmed. calrs never holds a wallet key: it watches payments that arrive at the owner's address and confirms the booking once one is final on chain. The full operator guide (setting up a wallet, the role, the rollout) lives in the ilia-booking repository (`docs/ton.md`); this page covers what calrs itself does.

## Prices

Each personal event type has one of three price modes, set in the **Payment in TON** section of the event type form, by the MCP tool `set_event_type_price`, or by `config apply` (`ton_price`):

| Mode | What the guest pays |
|---|---|
| Free (no price) | Nothing; the booking follows the type's usual flow |
| Fixed | One amount in TON (the invoice amount itself) or in USD (converted at the rate of the moment the guest books, inside the server's corridor) |
| By demand | The step of the site's demand ladder for the owner's current demand, in TON |

### The demand ladder

The ladder is one list of steps for the whole site, `free,20,40,60,80,100,200,300,400,500` by default (TON). With `n` the owner's live bookings ahead, the price is step `n`; the last step is the ceiling.

`n` counts the owner's bookings that start between now and the end of the booking horizon (`CALRS_MAX_HORIZON_DAYS`), on every event type:

- confirmed bookings (paid, waived or free);
- free requests waiting for the owner's approval;
- paid bookings not yet confirmed.

It does not count unpaid payment holds, cancelled or declined bookings, bookings in the past, or the owner's calendar busy time.

Rules of the ladder: 1 to 30 steps, comma-separated; `free` only as a run at the start; every other step 1 to 100000 TON with at most two decimals; never decreasing.

**Shown before booking, fixed at booking.** The public profile lists a priced personal event type with its price now after the duration (the ladder price, or the TON or USD price of a fixed one; team profiles show none). The slot page and the booking form show the price now, that it is fixed when the guest books, the ladder in short and that the network fee is paid by the wallet. The form carries the price shown. Inside the booking transaction, after the booking's row is inserted, calrs counts `n` again on the same connection:

- a lower price is charged without asking;
- a higher price, or a switch between free and priced, rolls the booking back and shows a "the price has changed" page that books at the new price with one click.

Two guests booking at once therefore never both get the free step: a pending free request counts toward `n`. At a priced step, guests who open invoices at the same `n` each pay that step, because unpaid holds do not count toward `n` (the price rises once a payment confirms). The invoice stores the amount, the mode, `n` and the quote; a later ladder change never touches an open invoice.

**A free step** opens no invoice: the booking waits for the owner's approval and holds the time. One guest (by email, typed Telegram username or client network) gets at most one free step per `CALRS_TON_FREE_PER_GUEST_DAYS` days (default 30, `0` for no limit), a cancelled or declined one included. A guest past the limit is never shut out: while the step is free, the ladder's first price is theirs (the "the price has changed" page says why and books at that price with one click). Only a ladder that is free all the way refuses them. The client network is an IPv4 /24 or an IPv6 /64, so guests sharing one also share the limit. The limit is a second check, not a protection: those identities are easy to change, which is why the owner approves every free booking.

The public price reveals roughly how full the owner's calendar is.

### "Any time, 1000 TON"

A price for meetings outside the usual hours is an ordinary event type with a fixed price, not a ladder step. Configure it like any type, for example:

- a fixed price of 1000 TON;
- its own weekly hours (00:00 to 23:59 every day, or whatever the owner chooses; 24:00 is not accepted, so no slot ending at midnight is offered: with 30-minute meetings the last one starts at 23:00 and 23:30 to 00:00 is lost);
- a minimum notice of 120 minutes or more, so the payment has time to settle;
- a frequency limit of 1 per day;
- public, or `private` behind invite links.

It never books over another booking or the owner's calendar busy time: "any time" means outside the working hours, not on top of another meeting. Its bookings count toward `n`. The TON settings page marks a fixed TON price at or below the ladder's ceiling.

## The payment page

A priced booking is held and the guest is sent to `/booking/pay/{token}`. This page never opens inside an iframe: from the embedded widget, a short page links to it with `target="_top"`. The layout follows BTCPay Server's checkout v2, one 30rem column on every screen:

- the meeting and its date; the amount in TON, plus the USD price and the rate with its sources and time when the price is in USD;
- one status box with the state (waiting, payment seen, partly paid, ...), "held until" with the time zone, and a countdown that turns amber at 5 minutes or less and red at 1 minute or less; after a renewal it also shows "renewed n of max";
- **Open in Tonkeeper**, the main button: `https://app.tonkeeper.com/transfer/<address>?amount=<nano>&text=<code>&exp=<unix>&asset_id=ton/<network>/coin`. The `asset_id` lets a multichain Tonkeeper keep the amount and the comment instead of asking for the token;
- **Another wallet**: `ton://transfer/<address>?amount=<nano>&text=<code>&exp=<unix>`, without `asset_id`, which MyTonWallet refuses;
- the wallet buttons come before the QR code (a phone cannot scan its own screen): a QR code of the Tonkeeper link, for Tonkeeper's scanner or the phone's camera, and under the fold-out "QR code for another wallet (MyTonWallet etc.)" a QR code of the `ton://` link;
- the address (shown on one line, cut in the middle, the full address on hover) and the comment code (marked amber), each copied whole by its icon button, which turns into a check, or by a tap on the value. With JavaScript off, the value is still selectable;
- the rules right under them: the comment is required, must not be encrypted, pay in TON and not in tokens, from an exchange put the code in the Memo field, and the deadline with its time zone ("Send the payment before 23:58 (UTC).");
- **I have paid, check now**, which wakes the watcher; and **Email me this link**, offered until the link has been emailed once, only when SMTP and the base URL are set.

While the payment is open, the page asks `/booking/pay/{token}/status` every 5 seconds and reloads when the state, the deadline or the renewal count changes. A finished payment shows only the result: a centred card with the meeting, a round icon and the state. A confirmed booking adds **Add to calendar**: `GET /booking/pay/{token}/ics`, the meeting's invite with the confirmation email's UID and SEQUENCE, so a calendar updates the event the email added. It is served over the pay token, so the page never carries the cancel token behind `/booking/ics/…` (whoever holds that can cancel), and since the pay link may have been passed to whoever pays, the file carries no one's identity: the title, the time and the place only, no ATTENDEE or ORGANIZER line, no guest name, email or notes, and the summary "<title> — <host's first name>". Any other state, a waived invoice still waiting for the host included, answers 404. Expired, late, closed, revoked, taken and refunded bookings add **Book another time**, and the states where the guest may need the host (busy, expired, late, taken, refund) add "Questions? Write to <host> at <reply address>". The language menu sits in the footer under the card, as on every public page (see [page language](booking-flow.md#page-language)).

## The TON settings page

**Admin → TON payments** (`/dashboard/admin/ton`, admins only) has:

- **Status** (read-only): payments on or off and why (an unset `CALRS_TON_NETWORK` and the reasons the watcher is not ready are in the admin's language; a configuration error is shown as calrs reports it, in English, as in the log and the agent API), the network, the wallet address in both forms, the indexer host, whether the watcher is ready and its last step, the open invoices, and whether the indexer and CoinGecko API keys are set (from systemd credentials, from the environment, or not set; never the value). "From systemd credentials" means calrs really read the credential: a variable already set in the environment wins, its credential is left unread, and the page says "from the environment".
- **Price by demand:** the ladder (a text field, with a row editor when JavaScript is on: one row per step, "Bookings: N", its value field and **Remove** named by that row for screen readers), the free-step limit, a preview of the price by `n`, the admin's own `n` and price now, and every priced event type with its mode, its price and the ladder price now. **Preview the ladder** shows the price table for the ladder entered without saving it.
- **Invoice timing:** the payment window, the grace after it, the safety fuse, renewals and the accepted underpayment (in TON).
- **Exchange rate (USD prices):** the corridor (both bounds or neither), the largest spread between the sources, the oldest accepted rate and the largest jump.
- **Alerts:** the alert email (empty: every admin).
- **Recent changes:** the last 20, with who, when and the values.

The network, the wallet address, the indexer URL and the API keys are not fields: they are set only in the server's environment and systemd credentials.

### Who wins

For each key: the server's environment (`CALRS_TON_*`), then the one value stored in the database (with its owner, the page or `config apply`), then the default.

- A field set by the environment is read-only and says which variable sets it; to change it on the page, remove the variable and restart calrs.
- A field set by `config apply` is read-only too; change it in the document, or leave it out of the document's `site.ton` section to hand it to the page.
- A POST of a read-only field is ignored.

Saving checks every field with the same parser the environment goes through, then the whole result (for example, half a corridor is refused). Anything invalid stores nothing and is shown next to its field. A valid save writes the values and one audit row per change (rows are kept for a year; once the save has committed, also a `calrs::audit` log event), then applies them at once: the running payments switch to the new settings, and the watcher restarts on them after its current step. Open invoices keep the amounts and deadlines stored with them. With payments off the values are stored for later.

A stored setting that later becomes invalid (after an environment change, say) leaves payments off, as an invalid environment always did: priced types are then not bookable, never free.

## Debug booking (ilia fork)

To test the guest side of a priced booking again and again without filling in the guest form each time, switch on **Debug** at the top of this page (**Admin → TON payments**) and press **Open test booking** (`/dashboard/admin/debug-booking`). Switched off, the address answers 404 to everyone and no page links to it; switch it off again once debugging is done.

- **The switch** is stored and audited like the settings above (its row is `CALRS_DEBUG_BOOKING`, shown as `debug_booking` in the last changes) and resolved on every dashboard request, so it takes effect at once, with no restart. Precedence: the server's environment `CALRS_DEBUG_BOOKING` (`1`, `true`, `yes`, `on` or `0`, `false`, `no`, `off`; anything else counts as off) wins and locks the switch, shown disabled with the variable and its value; then the stored value, which `config apply` may own (`site.ton.debug_booking`, locked here too); then off. The ilia-booking role leaves the variable unset unless `calrs_debug_booking` is `true` or `false`.
- **Marked as debug:** the page has a striped "DEBUG · test booking" banner and a yellow-and-black frame, readable in light and dark themes; each booking it made has a TEST badge in the bookings dashboard and on its payment page (the guest of a test booking is the admin).

- **Who:** admins only, never while impersonating another user; every form is CSRF-protected.
- **What it books:** one of the admin's own enabled personal event types (a slug starting with `test` is chosen first, then a priced one), at one of the next free times (computed like the slot page, shown in the admin's profile zone). The guest is the admin: the name is prefilled and editable, the email is a choice between the admin's sign-in and booking addresses only. The guest Telegram username (for a type that asks for one) is prefilled: the username of the chat the admin linked in Profile & Settings → Telegram, else **Guest Telegram username for test bookings** in the Debug block of this page (stored and audited like the switch, `CALRS_DEBUG_BOOKING_TELEGRAM`, checked like the booking form's field and stored without `@`; the environment wins and locks it, `config apply` owns it as `site.ton.debug_booking_telegram`), else empty; the page says where it came from.
- **How:** the form is handed to the public booking handler with the browser's own request headers, so every guest check applies: captcha (the page cannot solve it, so keep captcha off while using it), the per-IP booking limit, validation, the booking window and frequency caps, the price (the ladder's price now for this guest is sent as the quote, priced as the booking path prices it: the owner's demand, then the first price when this address, Telegram username or network already had a free booking within the free-step limit, which the page then says; it is counted again inside the booking transaction), the invoice and its limits (one open invoice per email address), the hold and the emails. The answer is the guest's own: the payment page, the confirmation page, or the refusal. The one exception is a price that changes between the page's quote and the booking transaction (a booking, a cancellation or a ladder change in between): the guest's "the price has changed" page would repost outside this page, so the page books once more at the price now. If the price changes again, it refuses (409, nothing booked, its invite removed) and says so. A private or internal type gets a single-use invite for the admin's address first. Payment is never skipped and never marked as received: pay from a wallet, or cancel.
- **Timing for this test booking** (priced types): **Normal** keeps this page's settings. **Fast test** gives the invoice 3 minutes to pay, one renewal of 3 minutes (the guest is asked to pay again, by Telegram and email), a minute of grace and a minute of expiry margin and a 10-minute fuse, then it expires and the booking is cancelled automatically: about 9 minutes from booking to a free slot, unpaid. **Custom** takes the window, the renewed window and the number of renewals in minutes (window 2–240, renewed window 2–1440, renewals 0–5; an empty field takes the fast value). The timing applies to that one invoice only (stored as `ton_invoices.debug_timing`), may go below this page's minimums (never under 2 minutes) and keeps every other rule: the window still ends 5 minutes plus the grace before the meeting's minimum notice, the hard deadline never passes the start, a renewal never shortens a deadline, and payment is never marked as received. It reaches the invoice only from this page (a task-local around its own call of the booking handler; a guest's form cannot set it), and the watcher honours it only for a booking marked as a test. While such an invoice is open the watcher steps at least every 10 seconds, whatever `CALRS_TON_POLL_SECS` says (its default is already 10), and also after a transient indexer error (5xx, network, unreadable answer); only a rate limit (429), a rejected API key or the wrong network keep their longer backoff (up to 2 and 5 minutes); calrs's sweep releases the booking within a minute of the expiry. The list shows each booking's timing, the renewals used and, while its invoice is open, a live countdown, and refreshes itself every 10 seconds while a booking is pending (also once its invoice has expired, until calrs declines it).
- **Telegram:** the box "deliver guest messages to my own linked Telegram chat" (ticked by default whenever Telegram is on and a chat is linked; untick it to test the guest's own deep link) links the booking's guest side to the chat linked in Profile & Settings → Telegram, as if the guest's deep link had been opened from it; the bot then sends what to pay (or the welcome) there, and later the confirmation, reminder or cancellation. Unticked, the payment page shows the guest's deep link as usual. When the link cannot be made after the booking (the chat was unlinked or the bot stopped meanwhile, or the link expired), the booking stands, is listed without "my chat", and the page shows a warning instead of the payment page. Without a linked chat, or with Telegram off, the box is disabled and the page says why.
- **Guest language:** the language the test guest books in, chosen from every shipped language (by its own name). By default it is the admin's own interface language (Profile & Settings), else the page's language. The booking stores it as its language, exactly as if the guest had picked it with the language switcher of the public pages, so the guest's confirmation page, emails and Telegram messages, and every later notice (reminder, cancellation, payment), are in that language; emails to hosts stay in each host's own language. Only the page's own booking is affected: a guest's booking keeps the language of the guest's browser or switcher pick. The list shows each test booking's language.
- **No linked Telegram chat:** while Telegram is on and the admin has not linked a chat, a notice at the top of the page says that the own-chat option is unavailable and links to **Profile & Settings → Telegram** to link it.
- **Test bookings:** every booking the page made is marked (`debug_bookings`) and listed with its status, invoice and payment page. **Cancel all my test bookings** cancels the admin's standing test bookings through the bookings dashboard's cancel (pending ones are declined, invoices closed, the usual notices sent, their reason in each test booking's own language, else the admin's); nothing else is touched. While a test booking stands it is a real booking: it holds the time and counts toward the demand price.
- **Audit:** each attempt and each cancel-all is logged under `calrs::audit` (who, the type, the time, created or refused).

## `config apply`

`site.ton` sets the same keys, `debug_booking` (`true`, `false` or `null`) for the Debug switch and `debug_booking_telegram` (a username without `@`, or `null`) for the guest Telegram username it prefills (see [CLI reference](cli-reference.md)). Left out, nothing changes. `config apply` runs as its own process and cannot see `calrs serve`'s environment: it validates what it stores as if no `CALRS_TON_*` were set, and a variable in the server's environment still wins at run time. It reports `restart: true` when a value changes, since `calrs serve` reads the stored settings at startup (except `debug_booking` and `debug_booking_telegram`, which are read on every request and take effect at once). Each change gets a `ton_settings_audit` row, the durable trail; the `calrs::audit` log event follows only a committed run (never a `--dry-run`), and a run with `RUST_LOG=off`, as the ilia-booking role runs it, logs none. Its export lists only the keys it owns.

## Environment

| Variable | Default | Page / apply |
|---|---|---|
| `CALRS_TON_NETWORK` | unset (payments off) | no |
| `CALRS_TON_ADDRESS` | | no |
| `CALRS_TON_API_URL`, `CALRS_TON_API_KEY`, `CALRS_TON_COINGECKO_KEY` | | no (systemd credentials) |
| `CALRS_TON_POLL_SECS`, `CALRS_TON_EXPIRY_MARGIN_MIN`, `CALRS_TON_RECORD_MIN_NANO`, the `CALRS_TON_MAX_*` limits, the rate sources and their URLs | | no |
| `CALRS_TON_WINDOW_MIN` | 30 | yes |
| `CALRS_TON_GRACE_MIN` | 15 | yes |
| `CALRS_TON_FUSE_HOURS` | 6 | yes |
| `CALRS_TON_RENEW_MIN_LEAD_HOURS`, `CALRS_TON_RENEW_WINDOW_MIN`, `CALRS_TON_MAX_RENEWALS` | 24, 360, 2 | yes |
| `CALRS_TON_UNDERPAY_TOLERANCE_NANO` | 0 | yes (in TON) |
| `CALRS_TON_RATE_MIN_USD`, `CALRS_TON_RATE_MAX_USD` | unset | yes |
| `CALRS_TON_RATE_MAX_SPREAD_PCT`, `CALRS_TON_RATE_MAX_AGE_MIN`, `CALRS_TON_RATE_MAX_JUMP_PCT` | 3, 10, 10 | yes |
| `CALRS_TON_ALERT_EMAIL` | unset (every admin) | yes |
| `CALRS_TON_LADDER` | `free,20,40,60,80,100,200,300,400,500` | yes |
| `CALRS_TON_FREE_PER_GUEST_DAYS` | 30 | yes |
| `CALRS_DEBUG_BOOKING` | off | yes: the Debug switch at the top (the debug booking page) |
| `CALRS_DEBUG_BOOKING_TELEGRAM` | unset | yes: the guest Telegram username the debug booking page prefills |
