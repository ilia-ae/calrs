# Booking confirmation page (templates/confirmed.html)

confirmed-page-title-pending = Booking pending
confirmed-page-title-booked = Booking confirmed

confirmed-heading-reschedule-requested = Reschedule requested
confirmed-heading-rescheduled = Rescheduled!
confirmed-heading-pending = Pending confirmation
confirmed-heading-booked = You're booked!

confirmed-subtitle-reschedule-requested = Your reschedule request has been sent to { $host }. You'll receive an email at { $email } once it's approved.
confirmed-subtitle-rescheduled = Your booking has been rescheduled. A confirmation email has been sent to { $email }.
confirmed-subtitle-pending = Your booking request has been sent to { $host }. You'll receive an email at { $email } once it's confirmed.
confirmed-subtitle-booked = A confirmation email has been sent to { $email }.

confirmed-detail-event = Event:
confirmed-detail-date = Date:
confirmed-detail-time = Time:
confirmed-detail-with = With:
confirmed-detail-location = Location:
confirmed-detail-notes = Notes:
confirmed-detail-additional-guests = Additional guests:

confirmed-book-another = Book another time

confirmed-add-to-calendar = Add to calendar

# Slot picker (templates/slots.html)

## ilia-ov-slots
slots-time-format-24 = 24h
slots-time-format-12 = 12h
slots-prev-month = Previous month
slots-next-month = Next month
slots-prev-week = Previous week
slots-next-week = Next week
book-remove-guest = Remove guest
ilia-error-back-to-site = Back to the site

slots-location-video = Video call
slots-location-phone = Phone call
slots-location-google-meet = Google Meet

slots-tz-label = Your timezone
slots-time-format-label = Time format

## ilia-ov5-guest
tz-city-midway = Midway
tz-city-hawaii = Hawaii
tz-city-alaska = Alaska
tz-city-los-angeles = Los Angeles
tz-city-denver = Denver
tz-city-chicago = Chicago
tz-city-new-york = New York
tz-city-sao-paulo = São Paulo
tz-city-cape-verde = Cape Verde
tz-city-london = London
tz-city-paris = Paris, Brussels
tz-city-helsinki = Helsinki, Kyiv
tz-city-moscow = Moscow
tz-city-dubai = Dubai
tz-city-india = India
tz-city-bangkok = Bangkok
tz-city-shanghai = Shanghai
tz-city-tokyo = Tokyo
tz-city-sydney = Sydney
tz-city-auckland = Auckland

tz-group-common = Common timezones
tz-region-africa = Africa
tz-region-america = Americas
tz-region-antarctica = Antarctica
tz-region-arctic = Arctic
tz-region-asia = Asia
tz-region-atlantic = Atlantic Ocean
tz-region-australia = Australia
tz-region-europe = Europe
tz-region-indian = Indian Ocean
tz-region-pacific = Pacific Ocean
slots-offset-note-day = Times on %1 are %2 (clocks change on %3).
slots-offset-note-range = Clocks change on %1: from then on, times are %2.

slots-view-month = Month view
slots-view-week = Week view
slots-view-column = Column view

slots-weekday-mon = Mon
slots-weekday-tue = Tue
slots-weekday-wed = Wed
slots-weekday-thu = Thu
slots-weekday-fri = Fri
slots-weekday-sat = Sat
slots-weekday-sun = Sun

slots-weekday-mon-short = M
slots-weekday-tue-short = T
slots-weekday-wed-short = W
slots-weekday-thu-short = T
slots-weekday-fri-short = F
slots-weekday-sat-short = S
slots-weekday-sun-short = S

slots-select-date = Select a date

## ilia-ov3-guest
# The deferred slot load failed (slots.html); the booking form's TON price note (partials/ton_price.html).
slots-load-failed = Failed to load availability. Please refresh the page.
ton-book-price-note = paid in TON after you book

slots-loading-availability = Loading availability...
slots-click-highlighted = Click a highlighted date to see available times
slots-bookable-range-both = Bookable from { $from } to { $until }
slots-bookable-range-from = Bookable from { $from }
slots-bookable-range-until = Bookable until { $until }
slots-no-longer-bookable = This event is no longer bookable.
slots-no-times-month = No available times this month
slots-no-times-day = No available times this day
slots-no-availability-participants = No availability found for all participants this month
slots-week-more = more

# Booking form (templates/book.html)

book-page-title = Book { $title }
book-back-to-times = Back to times
book-name-label = Your name
book-name-placeholder = Jane Doe
book-email-label = Email
book-email-placeholder = jane@example.com
book-email-invalid = Please enter a complete email address, including the domain (e.g. jane@example.com).
book-notes-label = Notes
book-notes-optional = (optional)
book-notes-placeholder = Anything you'd like to discuss?
book-additional-guests-label = Additional guests
book-additional-guests-hint = (optional, up to { $max })
book-add-guest-btn = + Add guest email
book-guest-email-placeholder = colleague@example.com
book-phone-label = Phone number
book-phone-placeholder = 06 12 34 56 78
book-phone-help = Local numbers are fine; { $country } is assumed unless you start with +.
book-phone-optional-consequence = Leave it empty if you would rather not get text messages about this booking.
book-phone-required = A phone number is required for this booking.
book-phone-invalid-title = Invalid phone number
book-phone-invalid = Please enter a phone number we can text, or leave the field empty.
book-phone-country-search = Search
book-phone-country-label = Select country
book-phone-country-none = No country selected
book-phone-country-no-results = No countries match that search
captcha-label = Security verification
captcha-initial-state = Verify you're human
captcha-verifying = Verifying...
captcha-solved = You're human
captcha-error = Error
captcha-troubleshooting = Troubleshooting
captcha-wasm-disabled = Enable WASM for significantly faster solving
captcha-verify-aria = Click to verify you're a human
captcha-verifying-aria = Verifying, please wait
captcha-verified-aria = Verified
captcha-required = Please verify you're human
captcha-error-aria = An error occurred, please try again
book-confirm-button = Confirm booking

# SMS notifications (src/sms/message.rs).
#
# These are text messages, billed per 160-character segment (70 if the text
# contains any character outside the GSM-7 alphabet, which includes most
# accented letters). Keep them short and plain.

sms-confirmed = Booking confirmed: { $event }, { $date } at { $time } ({ $tz }).
sms-cancelled = Booking cancelled: { $event }, { $date } at { $time } ({ $tz }).
sms-rescheduled = Booking moved: { $event } is now { $date } at { $time } ({ $tz }).
sms-reminder = Reminder: { $event } starts { $date } at { $time } ({ $tz }).

# Shared labels used across the cancel / decline / approve / reschedule / claim flows

common-detail-guest = Guest:
common-detail-reason = Reason:
common-reason-optional = (optional)
common-close-page = You can close this page.

# Cancel flow (booking_cancel_form.html, booking_cancelled_guest.html)

cancel-page-title = Cancel booking
cancel-heading = Cancel booking
cancel-subtitle = You are about to cancel your booking.
cancel-reason-label = Reason
cancel-reason-placeholder-host = Let the host know why...
cancel-button = Cancel booking
cancelled-heading = Booking cancelled
cancelled-subtitle = Your booking has been cancelled and the host has been notified.

# Decline flow (booking_decline_form.html, booking_declined.html)

decline-page-title = Decline booking
decline-heading = Decline booking
decline-subtitle = You are about to decline this booking request.
decline-reason-placeholder-guest = Let the guest know why...
decline-button = Decline booking
declined-heading = Booking declined
declined-subtitle = The booking has been declined and the guest has been notified.

# Approve flow (booking_approve_form.html, booking_approved.html)

approve-page-title = Approve booking
approve-heading = Approve booking
approve-subtitle = You are about to approve this booking request.
approve-button = Approve booking
approved-heading = Booking approved
approved-subtitle = The booking has been confirmed and a confirmation email has been sent to { $email }.

# Claim flow (booking_claim_form.html, booking_claimed.html, booking_already_claimed.html)

claim-page-title = Claim booking
claim-heading = Claim booking
claim-subtitle = You are about to claim this booking. You will be added as an attendee.
claim-assigned-to = Assigned to:
claim-button = Claim this booking
claimed-page-title = Booking claimed
claimed-heading = Booking claimed
claimed-subtitle = You have claimed this booking. A calendar invite has been sent to your email.
already-claimed-page-title = Already claimed
already-claimed-heading = Already claimed
already-claimed-subtitle = This booking has already been claimed by { $name }.

# Generic error page (booking_action_error.html)

action-error-page-title = Booking action error

# Host-initiated reschedule (booking_host_reschedule.html)

host-resched-page-title = Reschedule booking — calrs
host-resched-heading = Reschedule booking
host-resched-subtitle = This will send { $guest } an email asking them to pick a new time.
host-resched-currently = Currently:
host-resched-button = Send reschedule request
host-resched-cancel-link = Cancel

# Guest reschedule confirmation (booking_reschedule_confirm.html)

resched-confirm-page-title = Confirm reschedule
resched-confirm-heading = Confirm reschedule
resched-confirm-subtitle = You are about to move your booking to a new time.
resched-was = Was:
resched-new = New:
resched-button = Confirm reschedule
resched-back-to-picker = Back to time picker

# Base layout chrome (templates/base.html)

base-loader-checking = Checking availability
base-loader-please-wait = Please wait, loading the latest calendar data...
base-stop-impersonating = Stop impersonating
base-theme-toggle = Toggle theme
base-language-choose = Choose your language
base-powered-by = Powered by

# Profile (templates/profile.html)

profile-pick-event-type-invite = Pick an event type to book a time.
profile-no-event-type = No event types available yet.

# Month and weekday names + per-locale date format patterns.
# Used by server-side date formatters in src/i18n.rs.

common-month-1 = January
common-month-2 = February
common-month-3 = March
common-month-4 = April
common-month-5 = May
common-month-6 = June
common-month-7 = July
common-month-8 = August
common-month-9 = September
common-month-10 = October
common-month-11 = November
common-month-12 = December

common-weekday-long-mon = Monday
common-weekday-long-tue = Tuesday
common-weekday-long-wed = Wednesday
common-weekday-long-thu = Thursday
common-weekday-long-fri = Friday
common-weekday-long-sat = Saturday
common-weekday-long-sun = Sunday

# Format patterns are parametric per locale to handle word order. Translators
# pick where each placeholder lands. Example outputs:
#   EN: April 2026  /  Tuesday, March 12, 2026
#   FR: avril 2026  /  mardi 12 mars 2026
#   ES: abril 2026  /  martes, 12 de marzo de 2026
common-format-month-year = { $month } { $year }
common-format-long-date = { $weekday }, { $month } { $day }, { $year }
common-format-date = { $month } { $day }, { $year }

# Email signatures and shared bits (src/email.rs)

email-signature = — calrs
email-action-reschedule = Reschedule
email-action-cancel-booking = Cancel booking

# Email: guest booking confirmation

# Kept to "event — date": Exchange titles the guest appointment after the
# email Subject header, not the ICS SUMMARY (#157).
email-confirm-subject = { $event } — { $date }
email-confirm-greeting = Hi { $name },
email-confirm-headline = Your booking has been confirmed!
email-confirm-ics-attached-plain = A calendar invite is attached.
email-confirm-ics-attached-html = A calendar invite is attached to this email.
email-confirm-need-to-cancel = Need to cancel? { $url }

# Email: guest reminder

email-reminder-subject = Reminder: { $event } at { $time }
email-reminder-headline = Your meeting is coming up.

# Email: guest cancellation

email-cancel-subject = Cancelled: { $event } — { $date }
email-cancel-headline-by-host = Your booking has been cancelled by { $host }.
email-cancel-headline-by-guest = Your booking has been cancelled.
email-cancel-ics-attached-plain = A calendar cancellation is attached.
email-cancel-ics-attached-html = A calendar cancellation is attached to this email.

# Confirmation email: notice-window policy lines (src/email.rs)

email-confirm-cancel-notice = Note: cancellation requires at least { $minutes } minutes notice.
email-confirm-reschedule-notice = Note: rescheduling requires at least { $minutes } minutes notice.

# Event type form: Google Meet location + cancel/reschedule minimum notice
# (templates/event_type_form.html, src/google_meet.rs)

## ilia-ov-eventform
# The collapsed "Payment in TON" section's summary: no payment, or the price comes from the demand ladder.
ton-etf-hint-off = Free
ton-etf-hint-ladder = By demand

event-type-form-location-google-meet = Google Meet (auto-generated link)
event-type-form-location-google-meet-hint = A unique Google Meet link is created on confirmation, owned by the assigned host. Every host (you, or every eligible team member) must have Google Calendar connected with a write-back calendar selected.

google-meet-prereq-no-host = Google Meet requires a host with Google Calendar connected.
google-meet-prereq-no-eligible = Google Meet requires at least one eligible team member with Google Calendar connected.
google-meet-prereq-missing = Google Meet requires every host to have Google Calendar connected with a write-back calendar selected. Still missing: { $names }. Connect them at Dashboard → Calendar sources.
google-meet-unavailable-title = Google Meet is not available
google-meet-dynamic-group-unavailable = The host needs Google Calendar connected with a write-back calendar selected.

event-type-form-cancel-notice-label = Minimum notice to cancel
event-type-form-reschedule-notice-label = Minimum notice to reschedule
event-type-form-notice-help = Leave empty for no restriction.
event-type-form-resources-label = Required resources
event-type-form-resources-hint = Slots are offered only when the selected resources are available, according to the mode below.
event-type-form-resources-mode-all = All selected resources must be free
event-type-form-resources-mode-round-robin = Any one free resource is enough (it gets assigned to the booking)
event-type-form-notice-unit-minutes = minutes
event-type-form-notice-unit-hours = hours
event-type-form-notice-unit-days = days
event-type-form-booking-horizon-label = Booking horizon
event-type-form-booking-horizon-help = How many days ahead guests can book. Leave empty for no limit, 0 for today only.
event-type-form-bookable-range-label = Bookable date range
event-type-form-bookable-from = from
event-type-form-bookable-until = until
event-type-form-bookable-range-help = Only these days can be booked. Leave empty for no limit. Use overrides for different hours on single days.
event-type-form-bookable-range-past-warning = This date range has already ended. The event type is no longer listed publicly.
event-type-form-bookable-range-too-far-warning = This date range starts more than a year ahead. Guests can book at most one year in advance, so booking opens on { $date }.
event-type-form-bookable-range-horizon-warning = The booking horizon ends before this date range starts. Guests can book from { $date }.
event-type-form-bookable-range-bookings-outside =
    { $count ->
        [one] 1 existing booking falls outside the bookable date range.
       *[other] { $count } existing bookings fall outside the bookable date range.
    }
event-type-form-bookable-range-bookings-link = View bookings
event-type-form-saved = Saved.

# Booking confirmation: cancel/reschedule policy notices (templates/confirmed.html)

confirmed-cancel-notice-info = Cancellation requires at least { $minutes } minutes notice before the meeting.
confirmed-reschedule-notice-info = Rescheduling requires at least { $minutes } minutes notice before the meeting.

# Booking action blocked page (templates/booking_action_blocked.html)

booking-blocked-title-cancel = This booking can no longer be cancelled online
booking-blocked-title-reschedule = This booking can no longer be rescheduled online
booking-blocked-body = The host requires at least { $minutes } minutes of notice. If you cannot attend, please email <a href="mailto:{ $host_email }">{ $host_email }</a> directly.

# Dashboard event types listing (templates/dashboard_event_types.html)

## ilia-ov-dates
# Booking times on the dashboard lists: the day ("Today", "Tomorrow", a weekday
# or a date), then the start and end times and the host timezone.
dashboard-when-today = Today
dashboard-when-tomorrow = Tomorrow
dashboard-when-range = { $date } at { $start } — { $end } { $tz }

dashboard-event-types-copy = Copy
dashboard-event-types-copied = Copied!
dashboard-event-types-copy-title = Copy booking link
dashboard-event-types-copy-failed = Copy failed

# Dashboard sidebar and shared chrome (templates/dashboard_base.html)

## ilia-ov-shell
nav-menu = Menu
event-types-more-actions = More actions

nav-section-scheduling = Scheduling
nav-overview = Overview
nav-event-types = Event Types
nav-bookings = Bookings
nav-teams = Teams
nav-section-shared-links = Shared Links
nav-invite-links = Invite Links
nav-section-calendars = Calendars
nav-sources = Sources
nav-section-personal = Personal
nav-settings = Profile & Settings
nav-troubleshoot = Troubleshoot
nav-section-admin = Admin
nav-admin-panel = Admin Panel
nav-sign-out = Sign out
nav-release-notes = View release notes

# Timezone mismatch banner (templates/dashboard_base.html)

tz-banner-text = Your browser timezone is { $detected } but your booking timezone is set to { $current }.
tz-banner-update = Update
tz-banner-dismiss = Dismiss

# Markdown editor toolbar (templates/dashboard_base.html)

editor-link-prompt = Enter URL:
editor-link-default-label = link text
editor-placeholder-text = text
editor-nothing-to-preview = Nothing to preview

# Dashboard overview (templates/dashboard_overview.html)

overview-page-title = Dashboard
overview-welcome = Welcome, { $name }
overview-public-page = Public page:
overview-avail-banner-title = Default availability
overview-avail-banner-body = Your default working hours have been set to Mon–Fri, 9:00–17:00. These are used when others include you in dynamic group meetings.
overview-avail-banner-cta = Review your availability
overview-dismiss = Dismiss
overview-getting-started = Getting started
overview-getting-started-help = Follow these steps to start accepting bookings.
overview-step-connect-calendar = Connect a calendar
overview-step-first-event-type = Create your first event type
overview-step-share-link = Share your booking link
overview-pending-approval = Pending approval
overview-booking-with = { $title } with { $guest }
overview-badge-pending = pending
overview-guest-booked = Guest booked:
overview-confirm = Confirm
overview-decline = Decline
overview-stat-event-types = Event Types
overview-stat-upcoming = Upcoming Bookings
overview-stat-pending = Pending Approval
overview-stat-sources = Calendar Sources
overview-quick-actions = Create a new event type
overview-action-public-title = Public booking page
overview-action-public-desc = Share a link — anyone can pick a slot and book time with you.
overview-action-team-title = Team scheduling
overview-action-team-desc = Distribute bookings across team members or find a time everyone's free.
overview-action-team-desc-empty = Create a team first, then set up shared event types.
overview-action-private-title = Private invite-only
overview-action-private-desc = Generate single-use links for specific contacts. No one else can book.
overview-action-shared-title = Shared invite links
overview-action-shared-desc = Any colleague in the team can generate booking links to share externally.
overview-action-reason-calendar = Connect a calendar first
overview-action-reason-ask-admin = Ask an admin to create a team
overview-action-reason-team-admin = Requires a team — create one first
overview-action-reason-team-member = Requires a team — ask an admin

# Dashboard bookings (templates/dashboard_bookings.html)

## ilia-ov-lists
team-form-slug-placeholder = engineering
troubleshoot-label-past = Already passed
troubleshoot-legend-beyond-horizon = Beyond horizon or date range

bookings-page-title = Bookings
bookings-pending-approval = Pending approval
bookings-available-to-claim = Available to claim
bookings-upcoming = Upcoming bookings
bookings-with = { $title } with { $guest }
bookings-guest-booked = Guest booked:
bookings-resource = Resource:
bookings-confirm = Confirm
bookings-reschedule = Reschedule
bookings-decline = Decline
bookings-claim = Claim
bookings-badge-awaiting-reschedule = awaiting reschedule
bookings-badge-writeback-failed = not in your calendar
bookings-writeback-failed-detail = Calendar write-back failed: { $error }
bookings-cancel = Cancel
bookings-reason-placeholder = Reason (optional)
bookings-confirm-cancel = Confirm cancel
bookings-back = Back
bookings-empty = No upcoming bookings yet.<br>Share your { $link } so people can book time with you.
bookings-empty-link-label = event type links

# Dashboard teams listing (templates/dashboard_teams.html)

teams-page-title = Teams
teams-heading = Teams
teams-new = New
teams-badge-public = public
teams-badge-private = private
teams-settings = Settings
teams-view = View
teams-empty = No teams yet.
teams-empty-admin = { $link } to collaborate with your team.
teams-empty-admin-link-label = Create one
teams-empty-member = Teams are created by administrators. Ask your admin to create one and add you as a member.

# Dashboard invite links (templates/dashboard_internal.html)

invite-links-page-title = Invite Links
invite-links-heading = Invite Links
invite-links-new = New internal event
invite-links-help = Generate single-use booking links for internal event types. Any authenticated colleague can create and share links here.
invite-links-duration = { $minutes }min
invite-links-hosted-by = Hosted by { $host }
invite-links-get-link = Get link
invite-links-invites = Invites
invite-links-empty = No internal event types yet.<br>{ $link } with "Internal" visibility to let any colleague generate booking links.
invite-links-empty-link-label = Create an event type
invite-links-js-generating = Generating...
invite-links-js-copied = Copied!
invite-links-js-error = Error

teams-member-count =
    { $count ->
        [one] { $count } member
       *[other] { $count } members
    }

# Dashboard calendar sources (templates/dashboard_sources.html)

sources-page-title = Calendar Sources
sources-heading = Calendar sources
sources-add = Add
sources-last-sync = Last sync:
sources-sync-failing = Sync failing since { $since } UTC: { $error }
sources-sync = Sync
sources-full-resync = Full resync
sources-full-resync-title = Clear cache and re-fetch all events from server
sources-test = Test
sources-reconnect = Reconnect
sources-reconnect-title = Re-run the Google consent flow
sources-edit = Edit
sources-remove = Remove
sources-remove-confirm = Remove source '{ $name }'? This will delete all synced events from this source.
sources-no-write-calendar = No write calendar selected. Confirmed bookings stay in calrs and are not pushed to this calendar. Pick one below to enable write-back.
sources-write-bookings-to = Write bookings to:
sources-write-none = None (don't write)
sources-empty = No calendar sources connected. { $link } to check availability.
sources-empty-link-label = Add one
sources-calendars = Calendars
sources-calendars-synced = Calendars synced: { $synced } of { $total }
source-calendars-page-title = Calendars to sync
source-calendars-heading = Calendars to sync
source-calendars-back = Back to calendar sources
source-calendars-help = Pick the calendars of { $name } that calrs syncs. Unchecked calendars are never fetched and do not block your availability.
source-calendars-select-all = Select all
source-calendars-select-none = Select none
source-calendars-write-badge = Bookings written here
source-calendars-write-hint = Confirmed bookings are written to this calendar whether or not it is synced.
source-calendars-save = Save
source-calendars-saved = Saved. Newly checked calendars are fetched at the next sync; use Sync on the sources page to fetch them now.
source-calendars-empty = No calendars found for this source yet. Run a sync to discover them.

# Dashboard event types listing (templates/dashboard_event_types.html)

event-types-page-title = Event Types
event-types-heading = Event types
event-types-new = New
event-types-badge-disabled = disabled
event-types-badge-internal = internal
event-types-badge-private = private
event-types-badge-resources = resources
event-types-badge-active = active
event-types-badge-expired = expired
event-types-badge-range-from = from { $date }
event-types-badge-range-until = until { $date }
event-types-send-invites = Send invites
event-types-duration = { $minutes }min
event-types-mode-collective = collective
event-types-mode-round-robin = round robin
event-types-edit = Edit
event-types-disable = Disable
event-types-enable = Enable
event-types-embed = Embed
event-types-overrides = Overrides
event-types-team-settings = Team settings
event-types-invites = Invites
event-types-view-public = View public page
event-types-view-page = View page
event-types-delete = Delete
event-types-delete-confirm = Delete event type '{ $title }'? This cannot be undone.
event-types-duplicate = Duplicate
event-types-duplicate-title = { $title } (copy)
event-types-empty = No event types yet. { $link } to start accepting bookings.
event-types-empty-link-label = Create one

# Markdown editor toolbar (templates/settings.html, templates/team_form.html)

editor-bold = Bold (Ctrl+B)
editor-italic = Italic (Ctrl+I)
editor-strikethrough = Strikethrough
editor-code = Inline code
editor-link = Insert link (Ctrl+K)
editor-toggle-preview = Toggle preview
editor-preview = Preview

# Profile and settings (templates/settings.html)

## ilia-ov-settings
settings-choose-file = Choose file
settings-no-file = No file chosen
admin-provider-local = Password
admin-provider-oidc = SSO
admin-theme-vates-desc = Red and space blue

settings-page-title = Settings
settings-heading = Profile & Settings
settings-public-page-label = Your public booking page
settings-copy = Copy
settings-copied = Copied!
settings-open = Open
settings-avatar = Avatar
settings-upload = Upload
settings-remove = Remove
settings-display-name = Display name
settings-display-name-placeholder = Your name
settings-username = Username
settings-username-hint = (used in your booking URL)
settings-username-pattern-title = Lowercase letters, numbers, and dashes only
settings-username-help = Your public booking page:
settings-title = Title
settings-title-placeholder = e.g. Software Engineer, Product Manager
settings-title-help = Shown on your public profile and in the sidebar.
settings-bio = Bio
settings-bio-placeholder = Tell people a bit about yourself...
settings-bio-help = Shown on your public booking page. Supports **bold**, *italic*, ~~strikethrough~~, `code`, and [links](url).
settings-booking-email = Booking email
settings-booking-email-help = This email will appear on your public booking pages and in email notifications. Leave empty to use your login email.
settings-booking-email-warning = Make sure this email exists on your mail provider. If it doesn't, notifications won't be delivered.
settings-timezone = Timezone
settings-timezone-help = Your availability rules and booking times are computed in this timezone.
settings-language = Language
settings-language-auto = Auto (browser default)
settings-language-help = Pick a UI language, or leave on Auto to follow your browser's setting.
settings-public-page-language = Public booking page language
settings-public-page-language-visitor = Each visitor's language
settings-public-page-language-host = Always my language
settings-public-page-language-help = Language of your public profile and booking pages, and of the emails your guests receive. "Always my language" uses the language chosen above.
settings-error-public-page-language = Pick a valid public booking page language.
settings-dynamic-group = Allow others to include me in dynamic group links
settings-dynamic-group-help = When enabled, other users can create ad-hoc collective meeting URLs that include you (e.g. { $example }).
settings-lend-resource = Lend my calendar access for resource reservations
settings-lend-resource-help = When a booking needs to reserve a shared resource (demo lab, meeting room) that your calendar account can write to, allow calrs to use your stored calendar credentials for that write.
settings-default-availability = Default availability
settings-default-availability-help = Your default working hours. Used for dynamic group links when others include you in a meeting.
settings-copy-to-all = Copy to all days
settings-copy-to-all-title = Copy the first enabled day's windows to all other enabled days
settings-add-window = Add time window
settings-remove-window = Remove window
settings-save = Save settings
settings-appearance = Appearance
settings-theme-system = System
settings-theme-light = Light
settings-theme-dark = Dark

# Sign in (templates/auth/login.html)

## ilia-ov-auth
ilia-mfa-recovery-copy = Copy the codes
ilia-mfa-recovery-copied = Codes copied
ilia-mfa-recovery-copy-manual = Copying did not work. The codes are selected: copy them by hand.

login-page-title = Sign in
login-heading = Sign in
login-subtitle = Sign in to your calrs account
login-sso = Sign in with SSO
login-or = or
login-email = Email
login-password = Password
login-submit = Sign in with email
login-no-account = Don't have an account? { $link }
login-register-link = Register

# Registration (templates/auth/register.html)

register-page-title = Register
register-heading = Create account
register-subtitle = Register for a new calrs account
register-domains-limited = Registration is limited to: { $domains }
register-name = Name
register-name-placeholder = Your name
register-email = Email
register-password = Password
register-password-hint = (min. 12 characters)
register-submit = Create account
register-have-account = Already have an account? { $link }
register-signin-link = Sign in

# Authentication errors (src/auth.rs)

auth-error-rate-limited = Too many login attempts. Please try again later.
auth-error-invalid-credentials = Invalid email or password
auth-error-internal = Internal error
auth-error-registration-disabled = Registration is disabled.
auth-error-name-length = Name must be between 1 and 255 characters
auth-error-email-length = Email must be between 1 and 255 characters
auth-error-email-invalid = Please enter a valid email address
auth-error-email-domain = Email domain not allowed
auth-error-password-length = Password must be at least 12 characters
auth-error-email-taken = Email already registered
auth-error-create-failed = Failed to create account

# Calendar source test and write-back setup (templates/source_test.html, templates/source_write_setup.html)

source-test-page-title = Calendar source
source-test-sync-heading = Sync: { $name }
source-test-heading = Connection test
source-write-page-title = Set up calendar write-back
source-write-back = Back to dashboard
source-write-heading = Where should bookings go?
source-write-help = When someone books a meeting with you, calrs can automatically create the event in your calendar. Pick which calendar to write bookings to for { $name }.
source-write-save = Save
source-write-skip = Skip for now
source-write-sync-results = Sync results

source-write-event-count =
    { $count ->
        [one] { $count } event
       *[other] { $count } events
    }

# Date overrides (templates/overrides.html)

overrides-page-title = Date overrides
overrides-heading = Date overrides
overrides-back-teams = Back to teams
overrides-back-event-types = Back to event types
overrides-intro = Add date-specific exceptions for { $title }
overrides-add-heading = Add new override
overrides-date = Date
overrides-type = Override type
overrides-type-blocked = Block entire day
overrides-type-custom = Custom hours
overrides-start-time = Start time
overrides-end-time = End time
overrides-add-submit = Add override
overrides-existing = Existing overrides
overrides-badge-blocked = blocked
overrides-badge-custom = custom hours
overrides-delete = Delete
overrides-delete-confirm = Delete this override?
overrides-empty = No date overrides yet.<br>Use the form above to block specific dates (holidays, days off) or set custom hours.

# Public team page (templates/team_profile.html)

team-profile-subtitle = Pick an event type to book a time.
team-profile-empty = No event types available yet.

# Availability troubleshoot (templates/troubleshoot.html, src/web/mod.rs)

## ilia-ov2-lists
# The troubleshoot timeline's legend entry for time already gone today.
troubleshoot-legend-past = Past
# Accessible names of the troubleshoot page's day arrows.
troubleshoot-prev-day = Previous day
troubleshoot-next-day = Next day
# Accessible name of a member's role select on the team settings page.
team-settings-role-of = Role of { $name }

troubleshoot-page-title = Troubleshoot
troubleshoot-empty = No event types found. { $link } to start troubleshooting availability.
troubleshoot-empty-link-label = Create one
troubleshoot-subtitle = See why time slots are available or blocked for { $title }
troubleshoot-duration = { $minutes }min
troubleshoot-buffer-before = { $minutes }min buffer before
troubleshoot-buffer-after = { $minutes }min buffer after
troubleshoot-min-notice = { $minutes }min notice
troubleshoot-blocked-override = Blocked by date override (day off)
troubleshoot-custom-hours-active = Custom hours override active (replaces weekly rules)
troubleshoot-legend-available = Available
troubleshoot-legend-calendar-event = Calendar event
troubleshoot-legend-booking = Booking
troubleshoot-legend-resource = Resource busy
troubleshoot-legend-outside = Outside hours
troubleshoot-legend-buffer = Buffer / Min. notice
troubleshoot-blocked-slots = Blocked slots
troubleshoot-none-date-blocked = This date is blocked by an availability override (day off). No slots available.
troubleshoot-none-custom-hours = Custom hours override active but no matching windows. Check your override settings.
troubleshoot-none-no-rules = No availability rules for this day of the week. This event type is not bookable on { $date }.
troubleshoot-none-all-bookable = No blocked slots during availability hours. All times are bookable.
troubleshoot-label-outside = Outside availability
troubleshoot-label-available = Available
troubleshoot-label-min-notice = Min. notice ({ $minutes }min)
troubleshoot-label-beyond-horizon = Beyond booking horizon ({ $days } days)
troubleshoot-label-outside-date-range = Outside the bookable date range
troubleshoot-label-buffer = Buffer ({ $minutes }min)
troubleshoot-label-resource-busy = Resource busy: { $names }
troubleshoot-detail-around = Around: { $label }
troubleshoot-detail-around-booking = Around: { $guest } booking
troubleshoot-reason-calendar-event = Calendar event: { $label }
troubleshoot-reason-booking = Booking: { $label }

# Invite management (templates/invite_form.html)

invites-heading = Invites
invites-back-teams = Back to teams
invites-back-event-types = Back to event types
invites-intro = Send invite links for { $title }
invites-capped = <strong>Input was capped at { $max } recipients per submission.</strong> Submit the rest in another batch.
invites-failed-hint = — check server logs for details.
invites-quick-link = Quick link
invites-quick-link-help = Generate a single-use link and copy it to your clipboard.
invites-get-link = Get link
invites-or-email = Or send via email
invites-recipients = Recipients
invites-recipients-hint = (one email per line, max { $max })
invites-message = Personal message
invites-message-hint = (optional, sent to every recipient)
invites-message-placeholder = Looking forward to showing you a demo...
invites-expires-in = Expires in
invites-expires-days = { $days } days
invites-expires-never = Never
invites-allow-multiple = Allow multiple bookings per recipient
invites-send = Send invites
invites-sent-heading = Sent invites
invites-badge-expired = expired
invites-badge-used = used
invites-badge-active = active
invites-sent-by = Sent by { $name }
invites-uses = { $used }/{ $max } uses
invites-expires-at = Expires { $date }
invites-copy-link = Copy link
invites-delete = Delete
invites-delete-confirm = Delete this invite?
invites-empty = No invites sent yet. Use the form above to send a booking link to someone.
invites-js-generating = Generating...
invites-js-copied = Copied!
invites-js-error = Error

invites-sent-count =
    { $count ->
        [one] Sent { $count } invite.
       *[other] Sent { $count } invites.
    }

invites-skipped-invalid =
    { $count ->
        [one] Skipped { $count } invalid row:
       *[other] Skipped { $count } invalid rows:
    }

invites-skipped-duplicate =
    { $count ->
        [one] Skipped { $count } duplicate row:
       *[other] Skipped { $count } duplicate rows:
    }

invites-failed =
    { $count ->
        [one] { $count } invite failed (DB or SMTP):
       *[other] { $count } invites failed (DB or SMTP):
    }

# Calendar source form (templates/source_form.html)

source-form-title-edit = Edit calendar source
source-form-title-add = Add calendar
source-form-heading-edit = Edit calendar source
source-form-heading-add = Connect a calendar
source-form-subtitle-edit = Update the connection. Leave the password blank to keep the existing one. After changing the URL or username, run a sync to refresh the discovered calendar list.
source-form-subtitle-add = Connect a CalDAV server or Microsoft Exchange (EWS) so calrs can check availability when guests book meetings.
source-form-backend = Backend

## ilia-ov3-dash
# The "Other / generic CalDAV" preset on the new-source form; it also becomes the default source name.
source-form-preset-other = Other CalDAV server

source-form-preset = Preset
source-form-connect-google = Connect with Google
source-form-google-unavailable = Google Calendar is not available. Contact your administrator.
source-form-name = Display name
source-form-name-placeholder = My Calendar
source-form-url-caldav = CalDAV URL
source-form-url-ews = EWS endpoint URL
source-form-username = Username
source-form-password = Password
source-form-password-keep = Leave blank to keep existing
source-form-password-placeholder = App password or account password
source-form-skip-test = Skip connection test
source-form-skip-test-help = Use this if the test hangs (common with some BlueMind/Zimbra setups). You can test the connection later.
source-form-save = Save changes
source-form-add = Add calendar source
source-form-help-google-configured = Click the button below to authorize calrs to access your Google Calendar.
source-form-help-google-unconfigured = Google Calendar integration is not configured yet. Ask your administrator to set up Google OAuth2 credentials in the admin panel.

# Calendar source form: provider help (templates/source_form.html)

source-form-help-bluemind = <strong>BlueMind</strong> — Use the DAV endpoint of your BlueMind server.<br> Typically: <code>https://mail.yourcompany.com/dav/</code><br> Username is your <strong>email address</strong> (e.g. <code>alice@yourcompany.com</code>), not just the login name.<br> If the connection test hangs, check "Skip connection test" and try syncing directly.
source-form-help-nextcloud = <strong>Nextcloud</strong> — Use the WebDAV root, not a specific calendar URL.<br> Typically: <code>https://cloud.example.com/remote.php/dav</code>
source-form-help-fastmail = <strong>Fastmail</strong> — Use your full email in the URL path.<br> Example: <code>https://caldav.fastmail.com/dav/calendars/user/you@fastmail.com/</code><br> Use an app-specific password (Settings &rarr; Privacy &amp; Security &rarr; Integrations).
source-form-help-icloud = <strong>iCloud</strong> — Use <code>https://caldav.icloud.com/</code><br> You need an app-specific password from <a href="https://appleid.apple.com" target="_blank" style="color: var(--accent);">appleid.apple.com</a> (Security &rarr; App-Specific Passwords).
source-form-help-zimbra = <strong>Zimbra</strong> — Use the DAV endpoint of your Zimbra server.<br> Typically: <code>https://mail.example.com/dav/</code>
source-form-help-sogo = <strong>SOGo</strong> — Use the SOGo DAV endpoint.<br> Typically: <code>https://mail.example.com/SOGo/dav/</code>
source-form-help-radicale = <strong>Radicale</strong> — Use the server root URL.<br> Typically: <code>https://cal.example.com/</code>
source-form-help-baikal = <strong>Baikal</strong>: Use the DAV endpoint of your Baikal server.<br> Typically: <code>https://cal.example.com/dav.php/</code>
source-form-help-exchange = <strong>Microsoft Exchange (EWS)</strong>. Use the SOAP endpoint:<br> <code>https://mail.example.com/EWS/Exchange.asmx</code><br> Username is the mailbox email; password must accept HTTP Basic over TLS (enable on a service mailbox if your tenant disabled Basic).<br> Make sure to also pick <strong>Microsoft Exchange (EWS)</strong> in the Backend dropdown above.
source-form-help-google = <strong>Google Calendar</strong>: Connect via OAuth2. No password needed.<br>
source-form-help-other = Enter your CalDAV server's <strong>DAV root URL</strong> — not a specific calendar or public link.<br> calrs will auto-discover your calendars via PROPFIND (RFC 4791).

# Markdown editor toolbar, short labels (templates/team_form.html, templates/team_settings.html)

editor-bold-short = Bold
editor-italic-short = Italic
editor-link-short = Insert link

# Team creation (templates/team_form.html)

team-form-heading = New team
team-form-name = Team name
team-form-name-placeholder = Engineering
team-form-slug = Slug
team-form-slug-hint = (URL-friendly identifier)
team-form-slug-pattern-title = Lowercase letters, numbers, and dashes only
team-form-description = Description
team-form-optional = (optional)
team-form-description-placeholder = What this team is about...
team-form-description-help = Shown on the team page. Supports **bold**, *italic*, and [links](url).
team-form-visibility = Visibility
team-form-public = Public
team-form-private = Private
team-form-visibility-help = Private teams get an invite token for sharing. Public teams are visible on the team profile page.
team-form-members = Members
team-form-members-help = You will be added as team admin automatically. Add individual users or link OIDC groups.
team-form-search-placeholder = Search users or groups...
team-form-search-users = Users
team-form-search-groups = OIDC Groups
team-form-you = (you)
team-form-submit = Create team

# Team settings (templates/team_settings.html)

team-settings-page-title = Settings
team-settings-subtitle = Team settings — team admins can edit these.
team-settings-public-url = Public URL
team-settings-public-url-help = Anyone can book via this link.
team-settings-invite-link = Invite link
team-settings-invite-link-help = Share this link to give people access to this private team's booking page.
team-settings-avatar = Team avatar
team-settings-profile = Profile
team-settings-description-placeholder = Tell people about this team...
team-settings-description-help = Shown on the team's public booking page. Supports **bold**, *italic*, and [links](url).
team-settings-visibility-help = Public teams are listed on the team profile page. Private teams require an invite link to access.
team-settings-members-help = Manage who belongs to this team. Add individual users or link OIDC groups for automatic sync.
team-settings-role-member = Member
team-settings-role-admin = Admin
team-settings-oidc-group = OIDC group
team-settings-remove = Remove
team-settings-save = Save changes
team-settings-danger-zone = Danger zone
team-settings-danger-help = Permanently delete this team. Event types will be unlinked (not deleted). This cannot be undone.
team-settings-delete = Delete this team
team-settings-delete-confirm = Delete team '{ $name }'? This cannot be undone.

# Event type form (templates/event_type_form.html)

etf-heading-edit = Edit event type
etf-heading-new = New event type
etf-disabled-banner = This event type is disabled, so nobody can book it yet.
etf-team = Team
etf-team-hint = (optional — leave empty for personal event type)
etf-team-personal = Personal
etf-scheduling-mode = Scheduling mode
etf-mode-round-robin = Round Robin — assign to one available member
etf-mode-collective = Collective — all members must be available
etf-scheduling-mode-help = Round Robin assigns to one available member (least busy first). Collective requires all members to be free at the same time.
etf-title = Title
etf-title-placeholder = 30min intro call
etf-slug = Slug
etf-slug-placeholder = auto-generated from title
etf-description-placeholder = A quick introductory call to discuss...
etf-description-help = Shown on the booking page. Supports **bold**, *italic*, and [links](url).
etf-location = Location
etf-location-link = Video call (static URL)
etf-location-jitsi = Jitsi (auto-generated room)
etf-location-webhook = Webhook (custom provider)
etf-location-phone = Phone
etf-location-in-person = In person
etf-location-custom = Custom
etf-location-details = Details
etf-location-details-placeholder = https://meet.example.com/my-room
etf-pattern-placeholder = Leave empty to use the org default pattern
etf-duration = Duration (minutes)
etf-slot-interval = Slot interval (minutes)
etf-slot-interval-placeholder = Same as duration
etf-slot-interval-help = How often slots start. Leave blank to match duration.
etf-required-members = Required members
etf-required-members-help = All checked members must be free for a slot to be offered. Uncheck members you want to exclude (their availability will be ignored).
etf-member-priority = Member Priority
etf-member-priority-help = Higher priority members are assigned bookings first when available. Same priority = balanced by recent booking count.
etf-member-timezone-title = Member's timezone. Their personal working hours are interpreted in this TZ.
etf-priority-high = High
etf-priority-medium = Medium
etf-priority-low = Low
etf-section-availability = Availability
etf-timezone-help = The hours below are interpreted in this timezone. For team event types, pick the team's working timezone (not necessarily the creator's).
etf-reset-default = Reset to my default
etf-reset-default-title = Replace these hours with your profile-default availability
etf-availability-prefilled = Pre-filled from your { $link }. You can override it here for this event type.
etf-availability-prefilled-link = default availability
etf-section-buffers = Buffers & notice
etf-buffer-before = Buffer before (min)
etf-buffer-after = Buffer after (min)
etf-min-notice = Minimum notice
etf-min-notice-help = How far in advance someone must book.
etf-section-limits = Booking limits
etf-first-slot-only = One slot per day
etf-first-slot-only-help = Only show the earliest available time each day.

## ilia-ov4-dash
etf-freq-per-member = Per team member

etf-freq-limit = Limit booking frequency
etf-freq-limit-help = Limit how many times this event can be booked per period.
etf-add-limit = Add limit
etf-section-options = Booking options
etf-requires-confirmation = Requires confirmation
etf-requires-confirmation-help = Bookings will be pending until you approve them from the dashboard.
etf-sms = SMS notifications
etf-sms-off = Off, no phone number asked
etf-sms-optional = Optional, guests may leave a number
etf-sms-required = Required, guests must leave a number
etf-sms-help = Texts the guest when their booking is confirmed, moved, cancelled, or about to start, in addition to email. Guests who leave the field empty simply get no SMS. Requires an SMS gateway in the { $link }.
etf-admin-panel-link = admin panel
etf-additional-guests = Additional guests
etf-guests-none = Guests cannot add others
etf-additional-guests-help = Allow the person booking to invite additional attendees who will receive the calendar invite.
etf-default-view = Default calendar view
etf-view-month = Month — calendar grid with slot list
etf-view-week = Week — 7-day columns with time slots
etf-view-column = Column — days listed with inline slots
etf-view-week-short = week
etf-view-column-short = column
etf-default-view-help = The view guests see by default. They can switch views anytime.
etf-conflict-calendars = Conflict calendars
etf-conflict-calendars-help = Select which calendars to check for conflicts. If none selected, all calendars are used.
etf-no-resources = No shared resources configured yet. Add one (demo lab, meeting room) in the { $link } to require it here.
etf-section-access = Access & notifications
etf-visibility-public = Public — visible on your profile
etf-visibility-internal = Internal — any colleague can generate invite links
etf-visibility-private = Private — invite link only
etf-visibility-help = Controls who can see and book this event type.
etf-vis-internal = Internal
etf-reminder = Booking reminder
etf-reminder-none = No reminder
etf-reminder-help = Send a reminder email to both you and your guest before the meeting.
etf-dynamic-group = Dynamic Group Link
etf-dynamic-group-help = Create an ad-hoc meeting link that checks availability for you and other users.
etf-dynamic-group-search = Search for a user to add...
etf-dynamic-group-note = Only users who allow dynamic group links are shown.
etf-dynamic-group-url = Group link URL
etf-watcher-teams = Watcher teams
etf-watcher-teams-help = Selected teams will be notified when bookings are made. Members can claim bookings to join as an attendee.
etf-save = Save changes
etf-create = Create event type
etf-js-loading = Loading...
etf-js-no-default = No default set
etf-js-reset-done = Reset!
etf-js-error = Error
etf-js-remove-limit = Remove limit
etf-period-day = Per day
etf-period-week = Per week
etf-period-month = Per month
etf-period-year = Per year

# Event type form: runtime summary hints (templates/event_type_form.html)

# %1 and %2 are substituted client-side; the values are only known once a field is edited.

etf-hint-no-days = No days set
etf-hint-every-day = Every day
etf-fmt-day-one = %1 day
etf-fmt-day-other = %1 days
etf-fmt-hours = %1h
etf-fmt-minutes = %1min
etf-hint-buffer-both = %1min before, %2min after
etf-hint-buffer-before = %1min buffer before
etf-hint-buffer-after = %1min buffer after
etf-hint-notice = %1 notice
etf-hint-no-buffers = No buffers, book anytime
etf-hint-max = Max %1
etf-hint-period-day = /day
etf-hint-period-week = /week
etf-hint-period-month = /month
etf-hint-period-year = /year
etf-hint-no-limits = No limits
etf-hint-confirmation-required = Confirmation required
etf-hint-auto-confirmed = Auto-confirmed
etf-hint-extra-guests-one = up to %1 extra guest
etf-hint-extra-guests-other = up to %1 extra guests
etf-hint-view = %1 view
etf-hint-reminder = reminder %1 before
etf-hint-no-reminder = no reminder

etf-guests-up-to =
    { $count ->
        [one] Up to { $count } additional guest
       *[other] Up to { $count } additional guests
    }

etf-reminder-hours =
    { $count ->
        [one] { $count } hour before
       *[other] { $count } hours before
    }

etf-reminder-days =
    { $count ->
        [one] { $count } day before
       *[other] { $count } days before
    }

# Event type form: preset banners and meeting-pattern help (templates/event_type_form.html)
# Literal braces are escaped as {"{"} because Fluent reads a bare { as a placeable.

etf-preset-public = Creating a <strong>public</strong> event type &mdash; anyone with the link can book.
etf-preset-private = Creating a <strong>private</strong> event type &mdash; only people you invite can book.
etf-preset-internal = Creating an <strong>internal</strong> event type &mdash; any colleague can share the booking link.
etf-preset-team = Creating a <strong>team</strong> event type &mdash; bookings are distributed across team members.
etf-pattern-hint = Optional pattern override. Tokens: <code>{"{"}username{"}"}</code>, <code>{"{"}event{"}"}</code>, <code>{"{"}date{"}"}</code>, <code>{"{"}random{"}"}</code>. Leave empty to use the organisation default configured by an admin.
etf-pattern-random-warning = This pattern has no <code>{"{"}random{"}"}</code> token. Two bookings of this event type on the same day will share the same room and the second guest can walk into the first guest's meeting. Use stable rooms only if that's what you want.
etf-webhook-hint = Per-booking meeting URL is fetched from the webhook configured by an admin under Admin &rarr; Meeting webhook. No URL needed here.

# Admin panel (templates/admin.html)

admin-page-title = Admin
admin-heading = Admin dashboard
admin-action-refused = Action refused:
admin-error-last-admin = This is the last enabled admin. Promote or enable another admin first.
admin-logo = Company logo
admin-logo-help = Displayed on public booking pages. PNG, JPEG, GIF or WebP, max 2 MB.
admin-logo-refused = The logo was not saved. Upload a PNG, JPEG, GIF or WebP image of at most 2 MB.
admin-favicon = Favicon
admin-favicon-help = Shown in browser tabs and bookmarks. Recommended: a square PNG of at least 32×32 pixels, max 512 KB. Without one, the calrs crab is shown.
admin-favicon-refused = The favicon was not saved. Upload a PNG, JPEG, GIF or WebP image of at most 512 KB.
admin-upload-save-failed = The file could not be written to the server's data directory. The server log has the details.

# User and team avatar uploads (#307)
avatar-upload-refused = The picture was not saved. Upload a PNG, JPEG, GIF or WebP image of at most 2 MB.
avatar-upload-save-failed = The picture could not be saved on the server. Try again later, or ask your administrator to check the server log.
admin-company-link = Company link
admin-company-link-help = Logo links to this URL on public booking pages. Leave empty for no link.
admin-theme = Theme
admin-theme-help = Choose a color theme for all pages. The dark/light toggle is separate — themes adapt to both modes.
admin-theme-default = Default
admin-theme-default-desc = Clean blue
admin-theme-nord-desc = Arctic frost
admin-theme-dracula-desc = Dark purple
admin-theme-gruvbox-desc = Retro warm
admin-theme-solarized-desc = Ethan's classic
admin-theme-tokyo-desc = Neon cityscape
admin-theme-custom = Custom
admin-theme-custom-desc = Your colors
admin-custom-colors = Custom colors
admin-color-accent = Accent
admin-color-accent-hover = Accent hover
admin-color-bg = Background
admin-color-surface = Surface
admin-color-text = Text
admin-save-theme = Save theme
admin-users = Users ({ $count })
admin-user-filter = Filter by name or email…
admin-badge-admin = admin
admin-badge-disabled = disabled
admin-impersonate = Impersonate
admin-demote = Demote
admin-promote = Promote
admin-disable = Disable
admin-enable = Enable
admin-delete = Delete
admin-no-users-match = No users match your filter.
admin-no-users = No users yet.
admin-groups = Groups ({ $count })
admin-group-filter = Filter by group name…
admin-group-name = Group name
admin-weight = weight:
admin-no-groups-match = No groups match your filter.
admin-no-groups = No groups synced yet. Groups are automatically synced from your OIDC provider.
admin-auth-settings = Auth settings
admin-registration-enabled = Registration enabled
admin-allowed-domains = Allowed email domains
admin-allowed-domains-hint = (comma-separated, leave empty for any)
admin-save-auth = Save auth settings
admin-system-settings = System settings
admin-base-url = Base URL
admin-base-url-help = Public URL of this instance. Used for OIDC redirects and links in emails (approve/decline, cancel, reminders).
admin-private-hosts = Private-host allowlist
admin-private-hosts-help = Comma-separated hostnames allowed to resolve to private/reserved IPs for CalDAV/EWS sources (SSRF guard opt-out). Only add hosts you control (e.g. a calendar server on the same Docker network). Leave empty to keep the guard active for all hosts.
admin-german-formal = German form of address
admin-german-formal-du = Informal (du)
admin-german-formal-sie = Formal (Sie)
admin-german-formal-help = How pages and emails address visitors whose browser asks for German. Users who picked a language in Profile & Settings keep theirs.
admin-unset-env = Unset the environment variable to edit this from here.
admin-save-system = Save system settings
admin-status = Status:
admin-status-enabled = enabled
admin-status-disabled = disabled
admin-status-disabled-paren = (disabled)
admin-status-configured = configured
admin-status-not-configured = not configured
admin-via-environment = (via environment)
admin-issuer = Issuer:
admin-client-id = Client ID:
admin-instance = Instance:
admin-oidc-settings = OIDC settings
admin-oidc-enabled = OIDC enabled
admin-issuer-url = Issuer URL
admin-client-id-label = Client ID
admin-client-secret = Client secret
admin-keep-current-hint = (leave empty to keep current)
admin-keep-current-set-hint = (leave empty to keep current — currently set)
admin-keep-unchanged = Leave empty to keep unchanged
admin-oidc-auto-register = Auto-register new users from OIDC
admin-save-oidc = Save OIDC settings
admin-google = Google Calendar (OAuth2)
admin-save-google = Save Google OAuth2 settings
admin-captcha = Captcha
admin-instance-url = Instance URL
admin-site-key = Site key
admin-secret = Secret
admin-widget-url = Widget script URL
admin-widget-url-help = Override if the CDN is blocked. Changes take effect immediately after saving.
admin-wasm-url = Widget WASM URL
admin-wasm-url-help = Where the widget loads its solver from. Set it to the WASM file your Cap server serves to keep booking pages from contacting cdn.jsdelivr.net. Leave empty for the default.
admin-captcha-wasm-url-invalid = The widget WASM URL must be an absolute http(s) URL.
admin-captcha-disable-help = Leave instance URL, site key and secret empty to disable captcha on booking pages.
admin-save-captcha = Save captcha settings
admin-resources = Resources
admin-resources-help = Shared bookable resources (demo lab, meeting rooms) backed by a calendar feed. Attached to event types, a busy resource blocks bookings.
admin-resource-stats = Events cached: { $events } &middot; Attached to { $attached } event type(s)
admin-never = never
admin-resource-sync-failed = (last attempt failed: { $error })
admin-writeback-enabled = Write-back: enabled ({ $via })
admin-writeback-readonly = Write-back: read-only
admin-teams-allowed = Teams allowed:
admin-teams-allowed-none = none (global admins only)
admin-sync-now = Sync now
admin-test-write = Test write
admin-delete-resource-confirm = Delete this resource? Event types using it will stop checking it.
admin-name = Name
admin-name-help = Leave empty to fetch the name from the feed.
admin-feed-url = ICS feed URL (publish address)
admin-feed-url-help = BlueMind: the resource calendar's public or private calendar address.
admin-caldav-url = CalDAV collection URL (for write-back)
admin-caldav-url-help = Optional. For BlueMind it is derived automatically from the feed URL.
admin-caldav-username = CalDAV username
admin-caldav-password = CalDAV password
admin-resource-teams = Teams allowed to use this resource
admin-resource-teams-help = Team admins of these teams can attach this resource to their team event types. Empty: global admins only.
admin-no-teams = No teams yet.
admin-save-resource = Save resource
admin-add-resource = Add resource
admin-jitsi = Jitsi (auto-generated meeting links)
admin-jitsi-help = When an event type's location is set to "Jitsi (auto-generated room)", calrs builds a fresh room URL per booking by appending the pattern below to your Jitsi base URL. No external API call is needed.
admin-display-name = Display name
admin-jitsi-display-name-placeholder = e.g. Meet DYB
admin-jitsi-display-name-help = Shown to guests on the slot picker and booking form. Defaults to "Video call" if empty.
admin-room-pattern = Room name pattern
admin-jitsi-disable-help = Leave the base URL empty to disable Jitsi auto-generation.
admin-save-jitsi = Save Jitsi settings
admin-meeting-webhook = Meeting webhook (bring-your-own provider)
admin-webhook-url = Webhook URL
admin-webhook-display-name-placeholder = e.g. Zoom, Whereby, Custom Meet
admin-webhook-display-name-help = Shown to guests instead of the generic "Video call" badge.
admin-authentication = Authentication
admin-auth-none = None
admin-auth-hmac = HMAC-SHA256 (X-Calrs-Signature header)
admin-shared-secret = Shared secret
admin-webhook-disable-help = Leave the URL empty to disable the meeting webhook.
admin-save-webhook = Save webhook settings
admin-smtp = SMTP settings
admin-smtp-test-sent = Test email sent.
admin-smtp-test-failed = Test email could not be sent. Check the server logs and your SMTP settings.
admin-smtp-env-error = SMTP environment configuration error:
admin-smtp-host = Host:
admin-smtp-from = From:
admin-smtp-enabled = SMTP enabled
admin-host = Host
admin-port = Port
admin-tls-mode = TLS mode
admin-tls-starttls = STARTTLS (port 587)
admin-tls-implicit = Implicit TLS (port 465)
admin-tls-none = None, unencrypted (local MTA only)
admin-smtp-username-hint = (leave empty for an unauthenticated relay)
admin-from-email = From email
admin-from-name = From name
admin-save-smtp = Save SMTP settings
admin-send-test-email = Send a test email to
admin-send-test-email-hint = (defaults to your account email)
admin-send-test-email-btn = Send test email
admin-smtp-clear-confirm = Delete the database SMTP configuration?
admin-clear-db-config = Clear database config
admin-sms = SMS settings
admin-sms-help = Optional. SMS is only sent for bookings on event types where "SMS notifications" is turned on, and only when the guest left a phone number.
admin-sms-test-sent = Test message sent.
admin-sms-test-checked = Credentials accepted.
admin-sms-test-error = The SMS gateway refused the request.
admin-sms-captcha-warning = The booking form is public and the recipient number comes from the guest, so SMS without a captcha is an open relay someone else can bill you for. Configure the captcha above, and restrict destination countries in your gateway's own settings.
admin-sms-sent-today = Sent today:
admin-sms-of-cap = of { $cap }
admin-sms-config-error = SMS configuration error:
admin-sms-gateway = Gateway:
admin-sms-account = Account:
admin-sms-sender = Sender:
admin-sms-enabled = SMS enabled
## ilia-ov3-admin
admin-sms-provider-webhook = Generic webhook
admin-sms-webhook-secret = HMAC secret
admin-sms-gateway-label = Gateway
admin-required-on-switch = Required when switching gateway
## ilia-ov2-settings
admin-sms-sender-label = Sender
admin-sms-api-base-url = API base URL
admin-sms-region-base-url = Region base URL
admin-sms-webhook-url = Webhook URL
admin-sms-twilio-sender-hint = A Twilio number in E.164 format (+15551234567), or an alphanumeric sender ID where the destination country allows one.
admin-sms-twilio-base-url-hint = Optional. Defaults to https://api.twilio.com. Use https, the credential travels on this connection.
admin-sms-gatewayapi-sender-hint = Up to 11 alphanumeric characters, or 15 digits.
admin-sms-gatewayapi-base-url-hint = Optional. Defaults to https://gatewayapi.com; use https://gatewayapi.eu to keep traffic in the EU.
admin-sms-sevenio-sender-hint = Up to 11 alphanumeric characters, or 16 digits.
admin-sms-sevenio-base-url-hint = Optional. Defaults to https://gateway.seven.io.
admin-sms-webhook-sender-hint = Optional. Passed through to your endpoint as "sender".
admin-sms-webhook-base-url-hint = calrs POSTs {"{"}"to", "text", "sender"{"}"} here and expects 2xx back.

admin-sms-docs = { $provider } API documentation
admin-sms-country = Default country code
admin-sms-country-hint = (used when guests enter a local phone number)
admin-sms-daily-cap = Daily limit
admin-sms-daily-cap-hint = (messages per day for the whole instance, 0 for no limit)
admin-sms-daily-cap-help = Past the limit calrs stops texting and keeps sending email, so bookings never fail because the SMS budget ran out.
admin-save-sms = Save SMS settings
admin-send-test-sms = Send a test message to
admin-send-test-sms-hint-check = (leave empty to only check the credentials)
admin-send-test-sms-hint-e164 = (E.164 format)
admin-test-gateway = Test gateway
admin-sms-clear-confirm = Delete the database SMS configuration?
admin-sms-allow-all = Let any user enable SMS on their event types
admin-sms-allow-all-help = Off by default: SMS spends credit on the account configured here, so only admins may switch an event type into an SMS mode.
admin-save-policy = Save policy
admin-page-of = Page %1 of %2
admin-show-more-js = Show %1 more
admin-show-fewer = Show fewer

# Admin panel: strings carrying markup or literal braces (templates/admin.html)

admin-delete-user-confirm = Permanently delete user { $email }?{"\u000A"}{"\u000A"}This removes their user record, scheduling account, calendar sources, event types, and all data uniquely owned by them. Past bookings will be deleted with their event types.{"\u000A"}{"\u000A"}For OIDC/SSO users: if auto-register is enabled, this person will be re-created on their next login.{"\u000A"}{"\u000A"}This cannot be undone.
admin-system-settings-help = Public URL and network-security settings. These can also be set with the environment variables <code>CALRS_BASE_URL</code> and <code>CALRS_ALLOW_PRIVATE_HOSTS</code>. When an environment variable is set it <strong>takes precedence</strong> over the value below.
admin-set-by-env = — set by environment ({ $var }), overriding the stored value
admin-google-help = To enable Google Calendar integration, create OAuth2 credentials at <a href="https://console.cloud.google.com/apis/credentials" target="_blank" style="color: var(--accent);">Google Cloud Console</a>. Enable the <strong>CalDAV API</strong> (reading busy times and write-back) and the <strong>Google Calendar API</strong> (Google Meet links), then add { $redirect_uri } as an authorized redirect URI.
admin-room-pattern-help = Available tokens: <code>{"{"}username{"}"}</code> (host), <code>{"{"}event{"}"}</code> (event type slug), <code>{"{"}date{"}"}</code> (YYYYMMDD), <code>{"{"}random{"}"}</code> (8 chars). Default: { $default }.
admin-room-pattern-warning = Without <code>{"{"}random{"}"}</code> the room name is predictable: two guests booking the same event type on the same day end up sharing a room and can see each other's meeting. Stable rooms are allowed (e.g. one personal room per host), but only enable this if you understand the trade-off.
admin-meeting-webhook-help = When an event type's location is set to "Webhook (custom provider)", calrs POSTs the booking payload to this URL on confirmation and expects a JSON body <code>{"{"}"url": "https://..."{"}"}</code> back.
admin-auth-hmac-help = With HMAC, calrs sends <code>X-Calrs-Signature: sha256=&lt;hex&gt;</code> over the raw request body.
admin-tls-none-warning = Pick <strong>None</strong> only for a relay on this machine that offers no STARTTLS, or whose certificate is self-signed. Mail, and any credentials, cross the wire in the clear.
admin-smtp-env-error-help = Fix the <code>CALRS_SMTP_*</code> environment variables, or unset them to manage SMTP from the database here.
admin-smtp-env-managed = Managed via <strong>environment variables</strong> (overrides the database). Edit the <code>CALRS_SMTP_*</code> variables to change it, or unset them to manage SMTP from here.
admin-smtp-env-help = Alternatively, configure via environment variables (which override this): <code>CALRS_SMTP_HOST</code>, <code>CALRS_SMTP_PORT</code>, <code>CALRS_SMTP_TLS_MODE</code> (<code>starttls</code>, <code>tls</code> or <code>none</code>), <code>CALRS_SMTP_USERNAME</code>, <code>CALRS_SMTP_PASSWORD</code>, <code>CALRS_SMTP_FROM_EMAIL</code>, <code>CALRS_SMTP_FROM_NAME</code>. Only <code>CALRS_SMTP_HOST</code> and <code>CALRS_SMTP_FROM_EMAIL</code> are required; omit the username and password to relay through a local MTA without authentication.
admin-sms-env-error-help = Fix the <code>CALRS_SMS_*</code> environment variables, or unset them to manage SMS from the database here.
admin-sms-env-managed = Managed via <strong>environment variables</strong> (overrides the database). Edit the <code>CALRS_SMS_*</code> variables to change it, or unset them to manage SMS from here.
admin-sms-env-help = Alternatively, configure via environment variables (which override this): <code>CALRS_SMS_PROVIDER</code>, <code>CALRS_SMS_API_KEY</code>, <code>CALRS_SMS_API_SECRET</code>, <code>CALRS_SMS_SENDER</code>, <code>CALRS_SMS_BASE_URL</code>, <code>CALRS_SMS_DAILY_CAP</code>, <code>CALRS_SMS_DEFAULT_COUNTRY_CODE</code>.
admin-sms-trial-warning = <strong>Twilio trial mode is on</strong> (<code>CALRS_SMS_TWILIO_TRIAL</code>). Guests receive Twilio's predefined <code>sms_appointment_reminders</code> template, not the real message, and only numbers verified in your Twilio console will be reached. This is a testing aid for trial accounts. Unset the variable before taking bookings.

admin-show-more =
    { $count ->
        [one] Show { $count } more
       *[other] Show { $count } more
    }

# Calendar source form: backend picker (templates/source_form.html)

source-form-backend-help = Pick the protocol your server speaks. EWS targets on-prem Exchange 2019/2016/2013.

admin-sms-going-live = <strong>Before going live:</strong> restrict destination countries in your gateway (Twilio calls this Geo Permissions), keep the account prepaid without auto-recharge, and leave the captcha on. Those three between them bound what an SMS pumping attempt can cost you.

troubleshoot-heading = Troubleshoot availability

# Host-side form validation errors (src/web/mod.rs)

form-error-team-name-slug-required = Name and slug are required.
form-error-team-name-length = Name must be at most 255 characters.
form-error-team-description-length = Description must be at most 5000 characters.
form-error-slug-charset = Slug must contain only lowercase letters, numbers, and dashes.
form-error-slug-reserved = This slug is reserved. Please choose a different one.
form-error-team-slug-taken = A team with this slug already exists.
form-error-title-required = Title is required to generate a slug.
form-error-event-type-slug-taken = An event type with this slug already exists.
form-error-event-type-slug-taken-team = An event type with this slug already exists in this team.
form-error-event-type-title-required = A title is required.
form-error-event-type-title-length = The title must be at most 255 characters.
form-error-event-type-duration = The duration must be a whole number of minutes, from 1 to 1440 (24 hours).
form-error-event-type-number = Buffers, minimum notice, reminder and additional guests must be whole numbers, 0 or more.
form-error-event-type-horizon = The booking horizon must be a whole number of days, 0 or more. Leave it empty for no limit.
form-error-location-required = Location details are required (e.g. a video call link, phone number, or address).
form-error-bookable-range-order = The end date must be on or after the start date.
form-error-bookable-range-invalid = Enter dates as YYYY-MM-DD.
form-error-not-team-admin = You are not a team admin of this team.
form-error-no-account = No scheduling account found. Please contact an administrator.
form-error-all-fields-required = All fields are required.
form-error-encryption = Encryption error.
form-error-connection-failed = Connection failed: { $error }. Check the URL and credentials, or check "Skip connection test" to save anyway.

# Settings page flash (src/web/mod.rs)

settings-saved = Settings saved.

# Profile settings validation and flash messages (src/web/mod.rs)

settings-error-name-length = Name must be between 1 and 255 characters.
settings-error-username-length = Username must be at least 2 characters.
settings-error-username-taken = This username is already taken.
settings-error-booking-email = Please enter a valid booking email address.
settings-error-save-failed = Failed to save settings.

# Host-facing error responses (src/web/mod.rs)

error-team-not-found-or-not-admin = Team not found or you are not a team admin.
error-team-not-found = Team not found.
error-event-type-not-found = Event type not found.
error-decrypt-failed = Failed to decrypt stored credentials.
error-source-not-found = Source not found.
error-source-no-password = Source has no stored password.
error-oauth-invalid-state = Invalid state parameter. Please try again.
error-oauth-no-code = No authorization code received.
error-oauth-not-configured = Google OAuth2 not configured.
error-no-scheduling-account = No scheduling account found.
error-private-event-type-not-found = Private event type not found.
error-access-denied = Access denied.

# Guest booking-flow errors (src/web/mod.rs)

error-slot-unavailable = This slot is no longer available.
error-slot-too-soon = This slot is no longer available (too soon).
error-slot-beyond-horizon = This slot is beyond the booking window.
error-slot-not-yet-bookable = This slot is not open for booking yet.
error-invite-required = This event type requires an invite link.
error-invite-invalid = Invalid invite link.
error-invite-expired = This invite link has expired.
error-invite-used = This invite link has already been used.
error-invalid-date = Invalid date.
error-invalid-time = Invalid time.
error-invalid-date-format = Invalid date format.
error-invalid-time-format = Invalid time format.
error-too-many-bookings = Too many booking attempts. Please try again in a few minutes.
error-too-many-requests = Too many requests. Please try again later.
error-no-members-available = No team members are available for this slot.
error-dynamic-group-public-only = Dynamic group links are only available for public event types.
error-user-not-found = User not found.

# Booking action error page: titles (templates/booking_action_error.html)

bae-title-captcha = Captcha verification failed
bae-title-invalid-booking = Invalid booking details
bae-title-unavailable = Not available right now
bae-title-cannot-approve = Cannot approve this booking
bae-title-invalid-link = Invalid link
bae-title-invalid-or-expired = Invalid or expired link
bae-title-booking-not-found = Booking not found
bae-title-already-approved = Already approved
bae-title-already-declined = Already declined
bae-title-already-cancelled = Already cancelled
bae-title-booking-cancelled = Booking cancelled
bae-title-booking-declined = Booking declined

# Booking action error page: bodies

bae-body-go-back = Please go back and try again.
bae-body-unavailable = The host isn't accepting more bookings for this date. Please pick a different date, or check back later.
bae-body-resource-gone = A required resource is no longer available for this time. Ask the guest to pick another slot.
bae-body-no-claim-token = No claim token provided.
bae-body-claim-invalid = This claim link is no longer valid.
bae-body-booking-gone = This booking no longer exists.
bae-body-decline-link-invalid = This decline link is invalid, has expired, or the booking has already been processed.
bae-body-cancel-link-invalid = This cancellation link is invalid, has expired, or the booking has already been cancelled.
bae-body-cancel-link-invalid-short = This cancellation link is invalid or has expired.
bae-body-reschedule-link-invalid = This reschedule link is invalid, has expired, or the booking has already been processed.
bae-body-approval-link-invalid = This approval link is invalid or has expired.
bae-body-already-approved = This booking has already been approved.
bae-body-already-declined = This booking has already been declined.
bae-body-already-cancelled = This booking has already been cancelled.
bae-body-was-cancelled = This booking was cancelled.
bae-body-declined-by-host = This booking has been declined by the host.

# Booking form validation (src/web/mod.rs)

validate-name-length = Name must be between 1 and 255 characters.
validate-email-length = Email must be between 1 and 255 characters.
validate-email-invalid = Please enter a valid email address.
validate-notes-length = Notes must be 5000 characters or less.
validate-date-too-far = Cannot book more than one year in advance.

# Additional guests and dynamic group links (src/web/mod.rs)

guests-not-allowed = Additional guests are not allowed for this event type.
guests-too-many =
    { $max ->
        [one] You can add at most one additional guest.
       *[other] You can add at most { $max } additional guests.
    }
guests-invalid-email = Invalid additional guest email: { $email }
dynamic-group-min-usernames = Dynamic group links require at least two usernames.
dynamic-group-user-not-found = User "{ $username }" not found.
dynamic-group-user-opted-out = User "{ $username }" has not enabled dynamic group links.

error-slot-unavailable-member = This slot is no longer available ({ $username } has a conflict).

# Pending requests: slot hold, expiry and approval re-check (#234)

sms-expired = Request expired: { $event }, { $date } at { $time } ({ $tz }). The host did not reply, please book another time.
email-pending-expired-status = Request expired
email-pending-expired-headline = Your booking request has expired: the host did not respond in time. Please pick another time.
email-pending-expired-action = Pick another time
bae-body-host-busy = The host (or a required team member) is no longer free at this time. Ask the guest to pick another slot.
bookings-error-generic = Something went wrong.
bookings-error-no-bookable-dates = This event type has no bookable dates left, so the guest could not pick a new time. Extend its date range first.
troubleshoot-detail-pending-request = { $title } (pending request)
etf-pending-blocks-slot = Pending requests block the time slot
etf-pending-blocks-slot-help = While a request waits for your answer, nobody else can book that time, on this event type or any of your others.
etf-pending-expiry = Request expiry
etf-pending-expiry-help = A request you have not answered by then is declined automatically and the guest is asked to pick another time. It comes sooner when the meeting gets too close for anyone else to book the slot. You get one reminder halfway.
etf-pending-expiry-never = Never

etf-pending-expiry-hours =
    { $hours ->
        [one] After { $hours } hour
       *[other] After { $hours } hours
    }

# Host, watcher and remaining guest emails (src/email.rs, #209)

email-footer-sent-by = Sent by
# Sender display name of guest emails: the host, then the instance name (#221).
email-sender-via = { $host } via { $instance }
email-date-at-time = { $date } at { $time }
email-detail-phone = Phone:
email-detail-expires = Expires:
email-detail-new-date = New date:
email-detail-new-time = New time:
email-detail-previous = Previous:
email-detail-requested = Requested:
email-detail-originally = Originally:
email-detail-organizer = Organizer:
email-detail-booked-by = Booked by:
email-detail-invited-by = Invited by:
email-detail-message = Message:
email-action-approve = Approve
email-action-decline = Decline
email-action-pick-new-time = Pick a new time
email-host-approve-line = Approve: { $url }
email-host-decline-line = Decline: { $url }
email-host-approval-dashboard = Log in to your dashboard to confirm or decline this booking.
email-host-new-subject = New booking: { $event }, { $guest } ({ $date })
email-host-new-heading = New booking!
email-host-new-message = { $guest } booked a slot with you.
email-host-confirmed-subject = Confirmed: { $event }, { $guest } ({ $date })
email-host-confirmed-heading = Booking confirmed
email-host-confirmed-message = You approved the booking with { $guest }.
email-host-confirmed-footer = The event has been added to your calendar.
email-host-sync-failure-subject = Action needed: calendar not updated for { $event } ({ $date })
email-host-sync-failure-heading = Your calendar was not updated
email-host-sync-failure-intro = This booking moved, but calrs could not update the event on your Google Calendar. The guest has the new time; your calendar still shows the old one.
email-host-sync-failure-action = Open the event in Google Calendar and move it to the time above. Move the existing event rather than recreating it, or the Google Meet link the guest already has stops working.
email-writeback-failure-subject = Action needed: { $event } ({ $date }) is not on your calendar
email-writeback-failure-heading = A booking is missing from your calendar
email-writeback-failure-intro = calrs could not add this booking to your calendar, even after several attempts. The booking is confirmed and the guest has the invitation.
email-writeback-failure-action = Add the event to your calendar yourself so you do not miss it, and check the calendar source in calrs. calrs keeps retrying for a few more hours; you will not get this email again for this booking.
email-detail-error = Error:
email-source-failure-subject = Action needed: calendar source "{ $source }" is not syncing
email-source-failure-heading = A calendar source stopped syncing
email-source-failure-intro = calrs has not been able to sync this calendar source for over an hour. Your booking pages still offer slots, but they are based on the last successful sync, so events added or moved since then are not taken into account.
email-source-failure-action = Check the source on the Calendar sources page: an expired password or revoked access is the usual cause. You will only hear about this source again if it recovers and then fails again.
email-detail-source = Calendar source:
email-detail-failing-since = Failing since:
email-host-reminder-subject = Reminder: { $event }, { $guest } ({ $date })
email-host-reminder-heading = Upcoming booking
email-host-reminder-message = Reminder: you have a booking with { $guest } coming up.
email-host-cancel-subject = Cancelled: { $event }, { $guest } ({ $date })
email-host-cancel-message-by-host = You cancelled this booking.
email-host-cancel-message-by-guest = { $guest } cancelled their booking.
email-host-approval-subject = Action required: { $event }, { $guest } ({ $date })
email-host-approval-heading = Action required
email-host-approval-message = { $guest } wants to book a slot with you.
email-host-approval-reminder-subject = Reminder: { $guest } is waiting for your answer ({ $event }, { $date })
email-host-approval-reminder-heading = Reminder: action required
email-host-approval-reminder-message = { $guest } is still waiting for your answer. If you do not respond, the request will be declined automatically on { $deadline }.
email-host-approval-footer = You can also manage this from your dashboard.
email-host-expired-subject = Expired: { $event } with { $guest } ({ $date })
email-host-expired-heading = Booking request expired
email-host-expired-message = Nobody answered the request from { $guest } before its deadline, so it was declined automatically and the guest was told to pick another time.
email-host-expired-footer = You can change how long requests wait in the event type settings.
email-host-reschedule-request-subject = Reschedule request: { $event }, { $guest } ({ $email })
email-host-reschedule-request-message = { $guest } wants to reschedule their booking.
email-watcher-claim-subject = Claim available: { $event }, { $guest } ({ $date })
email-watcher-claim-message = A new booking is available to claim. Click below to join as an attendee.
email-watcher-claim-link = Claim this booking: { $url }
email-watcher-claim-footer = You can also claim from your dashboard.
email-claim-confirmed-subject = Booking claimed: { $event }, { $guest } ({ $date })
email-claim-confirmed-footer = You will be added as an attendee on this meeting.
email-attendee-greeting = Hi,
email-attendee-message = You've been added as an attendee to a booking.
email-attendee-rescheduled-message = A booking you attend has moved to a new time.
email-attendee-cancelled-message = A booking you attend has been cancelled.
email-attendee-cancelled-subject = Cancelled: { $event }
email-original-booking-cancelled = Your original booking is cancelled as well.
email-pick-new-time-subject = Reschedule: { $event }, please pick a new time
email-pick-new-time-message = { $host } needs to reschedule your booking. Please pick a new time.
email-pick-new-time-link = Pick a new time: { $url }
email-rescheduled-message = Your booking has been rescheduled by { $host }.
email-reschedule-approved-message = { $host } has confirmed the new time of your booking.
email-rescheduled-ics-attached-plain = An updated calendar invite is attached.
email-rescheduled-ics-attached-html = An updated calendar invite is attached to this email.
email-need-to-reschedule = Need to reschedule? { $url }
email-invite-subject = { $host } invited you to book: { $event }
email-invite-message = { $host } has invited you to book: { $event }
email-invite-link = Click the link below to choose a time:
email-invite-action = Choose a time
email-cancel-reason-calendar-deleted = The calendar event was deleted by the host.

# Embed snippet generator (templates/embed.html)
embed-page-title = Embed — { $title }
embed-heading = Embed: { $title }
embed-subtitle = Generate a snippet to embed this event type on your own website.
embed-warn-internal = Heads up: this event type is <strong>internal</strong>. Embedded visitors will need a valid invite link to book — the generated snippet won't carry one. Consider switching visibility to public for direct embedding.
embed-warn-private = Heads up: this event type is <strong>private</strong>. Embedded visitors will need a valid invite link to book — the generated snippet won't carry one. Consider switching visibility to public for direct embedding.
embed-warn-private-team = Heads up: this team is <strong>private</strong>. Embedded visitors can't book without the team invite link — the generated snippet won't carry one. Consider switching the team to public for direct embedding.
embed-need-username = You need to set a <strong>username</strong> before you can embed this event type — the embed link is { $path }. { $link } and come back.
embed-need-username-link = Set a username in settings
embed-username-placeholder = username
embed-tab-inline = Inline
embed-tab-floating = Floating button
embed-tab-element = Element click
embed-layout = Layout
embed-theme = Theme
embed-theme-auto = Auto (follow visitor)
embed-brand-color = Brand color
embed-use-org-color = Use organization color
embed-button-text = Button text
embed-button-default = Book a meeting
embed-button-position = Button position
embed-pos-bottom-right = Bottom right
embed-pos-bottom-left = Bottom left
embed-pos-top-right = Top right
embed-pos-top-left = Top left
embed-button-colors = Button colors
embed-button-fg-placeholder = text
embed-button-colors-hint = Background / text
embed-show-icon = Show calendar icon
embed-element-help = Drop the snippet into your site once, then add the data attributes shown to any clickable element (button, link, div) you want to trigger the modal.
embed-snippet = Snippet
embed-loading = Loading…
embed-preview = Live preview
embed-preview-back = back to preview
embed-preview-floating-hint = Click the button to simulate the modal
embed-preview-element-hint = Demo triggers — on your site, add the data attributes from the snippet to any element. Click either one to try the modal:
embed-preview-text-link = or click this text link
embed-snippet-attr-comment = Add these attributes to any clickable element on your page:

# Leftovers in partly translated pages (host reschedule, impersonation banner, slots, team settings)
host-resched-when = { $date } at { $start } – { $end }
base-impersonating = You are impersonating { $name }
slots-rescheduling-banner = Rescheduling: { $title } (currently { $date } at { $time })
slots-invite-required-title = Booking requires an invite link
team-settings-slug-url = Used in the team URL: { $url }
team-event-type-no-teams = No teams exist yet.
team-event-type-not-team-admin = You are not a team admin of any teams.

# Google Calendar connection errors (src/web/sources.rs)
google-error-not-configured = Google Calendar integration is not configured. Ask your administrator to set up Google OAuth2 credentials in the admin panel.
google-error-no-base-url = The public base URL is not configured. Set it via the CALRS_BASE_URL environment variable or in the admin panel under System settings before using OAuth2 flows.
google-error-authorization = Google authorization failed: { $error }
google-error-token-exchange = Token exchange failed: { $error }
google-error-account-email = Failed to fetch Google account email: { $error }

# Admin panel notices (src/web/admin.rs)
admin-resource-feed-invalid = Invalid feed URL: { $error }
admin-resource-feed-check-failed = Feed check failed: { $error }
admin-resource-caldav-invalid = Invalid CalDAV URL: { $error }
admin-resource-credentials-failed = Could not store credentials: { $error }
admin-resource-added = Resource "{ $name }" added. Cached events: { $count }.
admin-resource-updated = Resource updated. Cached events: { $count }.
admin-resource-deleted = Resource deleted.
admin-resource-not-found = Resource not found.
admin-resource-synced = Feed synced. Cached events: { $count }.
admin-resource-sync-error = Sync failed: { $error }
admin-resource-no-caldav-url = No CalDAV URL configured and none could be derived from the feed URL.
admin-resource-no-write-credentials = No write credentials available (no service account, and no opted-in member has a matching calendar source).
admin-resource-write-passed = Write test passed with { $user } (created: { $created }, cleaned up: { $cleaned }).
admin-resource-write-verified = verified
admin-resource-write-unverified = unverified
admin-resource-write-check-calendar = check the resource calendar
admin-resource-write-failed = Write test failed. Last error: { $error }
admin-base-url-invalid = Base URL must be an absolute http(s) URL, e.g. https://cal.example.com
admin-company-link-invalid = Company link must start with http:// or https://
admin-smtp-host-required = SMTP host is required.
admin-smtp-from-invalid = Please enter a valid "from" email address.
admin-smtp-port-invalid = SMTP port must be a number between 1 and 65535.
admin-smtp-tls-invalid = TLS mode must be "starttls", "tls" or "none".
admin-smtp-password-required = A password is required when configuring SMTP for the first time.
admin-sms-switch-needs-secret = Switching SMS gateway requires entering that gateway's credential.
admin-sms-settings-invalid = SMS settings: { $error }.
admin-sms-sender-format = A numeric SMS sender must be in international format, e.g. +15551234567.
admin-sms-not-configured = SMS is not configured.
admin-sms-load-failed = Could not load the SMS configuration.
admin-sms-check-accepted = Credentials accepted by the gateway.
admin-sms-bad-number = That is not a usable phone number.
admin-sms-test-body = calrs test message. Your SMS gateway works.
admin-sms-test-accepted = Test message accepted by the gateway.

## Local multi-factor authentication

mfa-title = Two-factor authentication
mfa-invalid = Verification failed. Check your password or code. Codes can only be used once; wait for a new code if needed. After too many attempts, wait 15 minutes. If setup expired, start again.
mfa-restart = Back to login
mfa-manage = Manage two-factor authentication
mfa-recovery-help = Save these recovery codes somewhere safe. They are shown only once. Each code replaces an authenticator code for one use. New recovery codes replace all previous codes.
mfa-continue = Continue to dashboard
mfa-enabled = Two-factor authentication is enabled.
mfa-disabled = Two-factor authentication is disabled.
mfa-required-help = Your administrator requires two-factor authentication for local accounts. You cannot disable it.
mfa-code = Authenticator code or recovery code
mfa-regenerate = Generate new recovery codes
mfa-disable = Disable two-factor authentication
mfa-enable = Set up two-factor authentication
mfa-changes-help = Changes require your password. Disabling two-factor authentication or replacing recovery codes also requires a code and signs out your other sessions.
mfa-back = Back to settings
mfa-setup-help = Scan this QR code with your authenticator app, or enter the key below manually. Enter a six-digit code to finish setup. Setup expires after 10 minutes.
mfa-qr-alt = Authenticator setup QR code
mfa-login-help = Enter a code from your authenticator app, or use one of your saved recovery codes. This login expires after 10 minutes.
mfa-totp-code = Six-digit authenticator code
mfa-verify = Verify
mfa-policy-help = This policy applies to local accounts. SSO uses the identity provider’s MFA. Local admins must set up MFA before changing this policy. Requiring MFA signs out local sessions that have not passed MFA and requires enrollment at the next login.
mfa-policy = Local account MFA policy
mfa-optional = Optional
mfa-required = Required
mfa-save-policy = Save MFA policy

# First day of the week (#330)
week-start-label = First day of the week
week-start-inherit-site = Use site default
week-start-inherit-owner = Use owner’s default

## Booking directory and personal favorites

directory-title = Booking directory
directory-help = Find a colleague’s or team’s meeting link. Star the meetings you share often to keep them on your dashboard.
directory-search = Search by person, team or meeting title
directory-find = Search
directory-favorites-only = Favorites only
directory-favorites = Favorite meetings
directory-browse = Browse directory
directory-generate = Generate invite link
directory-invite-help = Single-use invite, valid for 7 days.
directory-add = Add { $title } to favorites
directory-remove = Remove { $title } from favorites
directory-added = Added to favorites.
directory-removed = Removed from favorites.
directory-error = This action could not be completed. Refresh the page and try again.
directory-manual-copy = Copy this link manually:
directory-empty = No meetings found. Try another search.
directory-favorites-empty = No favorites found. Star meetings in the directory to see them here.
directory-with-hosts = With { $names }

## ilia fork: TON payments (docs/ton.md)

# Price on the slot and booking pages
ton-slots-price = Price: ${ $usd }
ton-slots-price-approx = (≈ { $ton } TON)
ton-slots-price-note = paid in TON after you pick a time

# Event type form
ton-etf-section = Payment in TON
ton-etf-off-note = TON payments are not set up on this server. A paid event type cannot be booked until they are.
ton-etf-enabled = Paid booking
ton-etf-enabled-help = Guests pay in TON right after picking a time. The time is held while they pay, and an unpaid booking is released automatically.
ton-etf-price = Price
ton-etf-price-help = In USD: from $1 to $10000; the amount in TON is computed from the exchange rate when the guest books.
ton-etf-window = Time to pay (minutes)
ton-etf-window-help = From 10 to 240. Leave blank for the server default ({ $minutes } min).
ton-form-error-price = Enter a price in US dollars from 1 to 10000, with at most two decimals.
ton-form-error-window = The time to pay must be from 10 to 240 minutes.
ton-form-error-team = Team event types cannot be paid.

# Booking error pages
ton-bae-title-unavailable = This time cannot be booked
ton-bae-body-unavailable = Payment is temporarily unavailable, so this time cannot be booked right now. Please try again in a few minutes.
ton-bae-body-too-late = There is too little time left before this meeting to pay for it. Please pick a later time.
ton-bae-body-already-open = You already have an unpaid booking with this email address. We can email you its links: pay for it there, or cancel it to free the time, then book again.
ton-bae-body-personal-only = This meeting can only be booked on its host's own page.
ton-bae-title-unpaid = Booking not paid
ton-bae-body-unpaid = This booking has not been paid, so it cannot be approved. To confirm it free of charge, use "Confirm without payment" on the bookings page of your dashboard.
ton-bae-body-reschedule-unpaid = This booking has not been paid, so it cannot be moved. Pay for it, or cancel it and book another time.
ton-bae-body-pay-link-invalid = This payment link is not valid.

# Payment page
ton-pay-page-title = Payment: { $title }
ton-pay-held-until = The time is held for you until { $time } ({ $zone }).
ton-pay-left = %1 left
ton-pay-amount-label = Amount to pay
ton-pay-rate = ≈ ${ $usd } · 1 TON = ${ $rate } ({ $sources }, { $time } UTC)
ton-pay-open-wallet = Another wallet
ton-pay-open-tonkeeper = Open in Tonkeeper
ton-pay-qr-alt = QR code with the payment details
ton-pay-scan = Or scan this code with your wallet app.
ton-pay-qr-alt-tonkeeper = QR code for Tonkeeper with the payment details
ton-pay-scan-tonkeeper = Or scan this code with Tonkeeper or your phone's camera.
ton-pay-qr-other = QR code for another wallet (MyTonWallet etc.)
ton-pay-qr-tonkeeper = QR code for Tonkeeper
ton-pay-address = Address
ton-pay-comment = Comment
ton-pay-copy = Copy
ton-pay-copied = Copied
ton-pay-warn-comment = The comment is required: it is how we recognise your payment.
ton-pay-warn-encrypt = Do not encrypt the comment.
ton-pay-warn-token = Pay in TON, not in tokens.
ton-pay-warn-exchange = Paying from an exchange? Put the code above in the memo (comment) field.
ton-pay-warn-deadline = Send the payment before { $time } ({ $zone }). A payment sent later may not be counted.
ton-pay-check = I have paid, check now
ton-pay-email = Email me this link
ton-pay-emailed = We have sent this link to the email address you booked with.
ton-pay-email-unavailable = The link could not be emailed. Keep this page open or save its address.
ton-pay-checking-now = Checking for your payment. This page updates by itself.
ton-pay-state-waiting = Waiting for your payment.
ton-pay-state-seen = Your payment is visible on the network. Waiting for it to become final, which usually takes a few seconds.
ton-pay-state-partial = We received { $paid } TON. Please pay the remaining { $rest } TON with the same comment.
ton-pay-state-checking = The time to pay has run out. If you have already sent the payment, we are checking it. This page updates by itself.
ton-pay-state-paid = Payment received. Confirming your booking…
ton-pay-state-confirmed = Paid. Your booking is confirmed, and the confirmation email is on its way.
ton-pay-state-waived = No payment is needed for this booking. The host is confirming it.
ton-pay-state-busy = Payment received, but this time is no longer free. We will contact you.
ton-pay-state-expired = The time to pay has run out and the time was released. If you sent a payment anyway, write to us from the email address you booked with and we will refund it.
ton-pay-state-late = Your payment arrived after the deadline, so the booking was not confirmed. We will contact you about a refund. You can also write to us from the email address you booked with.
ton-pay-state-closed = This booking is no longer active.
ton-pay-state-revoked = This payment address is no longer in use. Do not send anything to it, and please book again.
ton-pay-rebook = Book another time

# Leaving the embedded widget
ton-exit-title = Time held
ton-exit-body = The time is held for you until { $time } ({ $zone }). Complete the payment to confirm your booking.
ton-exit-go = Go to payment

# Dashboard
ton-dash-badge-awaiting = awaiting payment · { $amount } TON · until { $time }
ton-dash-badge-partial = partly paid · { $paid } of { $amount } TON · until { $time }
ton-dash-badge-paid = paid · { $amount } TON
ton-dash-badge-overpaid = paid · overpaid by { $excess } TON
ton-dash-badge-busy = paid, time no longer free
ton-dash-badge-confirm-failed = paid, not confirmed · confirm it by hand
ton-dash-badge-refund = paid { $amount } TON · refund due
ton-dash-badge-waived = confirmed without payment
ton-dash-badge-unpaid = not paid
ton-dash-waive = Confirm without payment
ton-dash-error-unpaid = This booking has not been paid, so it cannot be confirmed or moved. Use "Confirm without payment" to confirm it free of charge, or decline it.
ton-dash-error-waive-failed = The booking could not be confirmed without payment: it is no longer awaiting payment.
ton-waive-page-title = Confirm without payment
ton-waive-heading = Confirm this booking without payment?
ton-waive-body = This booking is NOT paid ({ $amount } TON, due by { $time }). If you confirm it, the guest gets the meeting free of charge, and a payment that arrives later has to be refunded by hand.
ton-waive-submit = Confirm free of charge
ton-waive-back = Back to bookings

# Guest emails
ton-email-link-subject = Complete your booking: { $event }
ton-email-link-message = You started booking { $event } with { $host }. Complete your booking on the page below before the time runs out.
ton-email-link-action = Complete booking
ton-email-link-line = Complete your booking: { $url }
ton-email-expired-subject = Not booked: { $event }, { $date }
ton-email-expired-message = The payment for { $event } did not arrive in time, so the time was released. You are welcome to pick another time.
ton-email-expired-partial = We received part of the payment. We will contact you about a refund.
ton-email-rebook-line = Pick another time: { $url }

# Owner emails
ton-email-row-amount = Amount:
ton-email-row-paid = Received:
ton-email-row-excess = Overpaid:
ton-email-row-transfer = Transfer:
ton-email-row-sender = Sender:
ton-email-row-tx = Transaction:
ton-email-row-detail = Details:
ton-email-amount-value = { $amount } TON ≈ ${ $usd } (1 TON = ${ $rate })
ton-email-sender-wallet = { $address } (a wallet)
ton-email-sender-other = { $address } (not a wallet: an exchange or a contract)
ton-email-dashboard-line = Bookings: { $url }
ton-email-dashboard-action = Open bookings
ton-email-refund-rule = Refunds are made by hand from your wallet: to the sender's address when it is a wallet; otherwise only to an address the guest asks for from the email address they booked with, in a message that includes their payment or cancel link. The booking code, amount and transaction are public and prove nothing.
ton-email-received-subject = Payment received: { $event }, { $guest } ({ $date })
ton-email-received-heading = Payment received
ton-email-received-message = { $guest } paid { $amount } TON. The booking is confirmed.
ton-email-alert-subject = TON payment needs attention: { $what }
ton-email-alert-heading = A payment needs your attention
ton-alert-partial = partial payment
ton-alert-partial-body = Part of the amount due arrived. The booking stays unconfirmed until the rest arrives; if it does not, the booking is released at the deadline and the amount received has to be refunded.
ton-alert-overpaid = overpayment
ton-alert-overpaid-body = A payment arrived for a booking that is already paid. Refund it if it is worth it.
ton-alert-late = late payment
ton-alert-late-body = A payment arrived after its deadline. The booking was already released and can no longer be confirmed. Refund the payment, or ask the guest to book again and confirm the new booking without payment.
ton-alert-unmatched = unrecognised payment
ton-alert-unmatched-body = A payment arrived without a known booking code. Find the booking it belongs to, or refund it.
ton-alert-failed = failed transfer
ton-alert-failed-body = A transfer to the payment address failed on the network. The funds may still be on your wallet: check the transaction in the explorer.
ton-alert-not-text = transfer without a readable comment
ton-alert-not-text-body = A transfer with an encrypted comment, a token or another payload arrived. It is never counted automatically: check it by hand.
ton-alert-other = unusual transfer
ton-alert-other-body = A transfer could not be processed automatically. Check it in the payment records.
ton-email-problem-subject = TON payment problem: { $event } ({ $date })
ton-email-problem-urgent-subject = Urgent: a paid booking needs a decision: { $event } ({ $date })
ton-email-problem-heading = A paid booking needs your attention
ton-problem-host-busy = The guest paid, but the time is no longer free (a calendar event or another booking). The time stays held until the meeting starts. Ask the guest for a new time, confirm the booking by hand, or decline it and refund the payment.
ton-problem-partial = Only part of the amount was paid, and the booking then expired, was cancelled or declined, or was confirmed without payment. Refund the amount received unless you agreed otherwise with the guest.
ton-problem-overpaid = The booking is paid and confirmed with more than the amount due. Refund the excess if it is worth it.
ton-problem-refund-due = The booking was paid, then cancelled or declined. Refund the payment.
ton-problem-confirmed-unpaid = An unpaid booking was found confirmed. This should never happen: check how it was confirmed.
ton-problem-address-revoked = The payment address of this booking was revoked, and the booking will be released.
ton-email-health-subject = TON payments need attention ({ $kind })
ton-email-health-heading = TON payments need attention
ton-health-indexer-down = The payment watcher cannot reach the indexer. Payments are not being checked, and new paid bookings are refused until it recovers.
ton-health-auth-failed = The indexer refused the API key (it may be wrong, or for the other network). Check CALRS_TON_API_KEY.
ton-health-wrong-network = The indexer answers for another network than the one configured. Nothing is credited or expired until this is fixed.
ton-health-wallet-inactive = The receiving wallet is not active. Paid bookings are refused until it has made at least one outgoing transfer.
ton-health-lagging = The payment watcher is falling behind the transfers arriving at the address (possibly spam). Unpaid bookings do not expire until it catches up.
ton-health-stuck-finality = A transfer has not become final for a long time. The watcher waits for it before reading further.
ton-health-watcher-crashed = The payment watcher crashed several times and keeps restarting. Check the server log.
ton-health-disabled-with-open = TON payments are switched off while unpaid bookings are open. Their times are released at their deadlines.
ton-health-other-network-open = There are open bookings on another TON network than the one configured. They are released at their deadlines.
ton-health-rate-unavailable = No acceptable TON exchange rate is available (sources missing, disagreeing or outside your corridor). Paid bookings are refused until there is one.
ton-health-capacity = The limit of open or daily unpaid bookings was reached. New paid bookings are refused for now.
ton-health-other = The payment watcher reported a problem. Check the server log.

# TON follow-ups: releases, refunds, contact, resend
ton-resend-submit = Email me the links
ton-resend-title = Check your email
ton-resend-done = If there is an unpaid booking for this email address, we have sent its payment and cancel links to it.
ton-pay-state-taken = This time was taken before the booking was paid, so the booking was released. Nothing is due. Please pick another time.
ton-pay-state-refund = This booking ended without being confirmed. The { $paid } TON you paid will be refunded; the host will contact you.
ton-pay-contact = Questions? Write to { $host } at { $email }.
ton-pay-copy-address = Copy the address
ton-pay-copy-comment = Copy the comment
ton-pay-copy-manual = Copying did not work. The text is selected: copy it by hand.
ton-email-row-pay-by = Pay by:
ton-email-taken-message = The time you picked for { $event } was taken before the booking was paid, so the booking was released. Nothing is due. You are welcome to pick another time.
ton-email-refund-message = We received your payment for { $event }, but the time could not be confirmed, so the booking was released. { $host } will refund the payment and contact you.
ton-problem-host-busy-unpaid = The time of this unpaid booking was taken (a request was approved over it, or a calendar event appeared), so the booking was released before the guest paid. If anything was received, refund it.
ton-problem-confirm-failed = The guest paid, but the booking could not be confirmed automatically after several attempts (an error, not a busy host: see the server log). The time stays held until the meeting starts. Confirm the booking by hand, or decline it and refund the payment.
ton-health-capacity-near = A new paid booking took the last open place or the last of today's budget. The next ones are refused until bookings are paid or released. If this is not real demand, someone may be holding times without paying.
ton-health-payment-alerts-held = More transfers arrived for bookings you were already emailed about within the hour. They are recorded without an email each: check the payment records.
ton-waive-body-partial = This booking is only partly paid ({ $paid } of { $amount } TON received, due by { $time }). If you confirm it, the guest gets the meeting without paying the remaining { $rest } TON. Refund the amount received unless you agreed otherwise.

## Telegram (ilia fork)

tg-when = { $date }, { $time } ({ $tz })
tg-in-minutes =
    { $count ->
        [one] in { $count } minute
       *[other] in { $count } minutes
    }
tg-in-hours =
    { $count ->
        [one] in { $count } hour
       *[other] in { $count } hours
    }
tg-in-days =
    { $count ->
        [one] in { $count } day
       *[other] in { $count } days
    }
tg-reason = Reason: { $reason }
tg-link-line = { $label }: { $url }
tg-guest-confirmed = Booking confirmed: { $event }, { $when }. Host: { $host }.
tg-guest-pending = Request sent: { $event }, { $when }. You will get the host's answer here.
tg-guest-approved = Your request is confirmed: { $event }, { $when }.
tg-guest-reschedule-approved = Your new time is confirmed: { $event }, { $when }.
tg-guest-declined = Your request was declined: { $event }, { $when }.
tg-guest-cancelled-by-host = The host cancelled: { $event }, { $when }.
tg-guest-cancelled-by-guest = You cancelled your booking: { $event }, { $when }.
tg-guest-rescheduled = Booking moved: { $event } is now { $when }.
tg-guest-reschedule-pending = New time requested: { $event }, { $when }. You will get the host's answer here.
tg-guest-reschedule-requested = The host asks you to pick a new time for { $event }. It was { $when }.
tg-guest-reminder = Reminder: { $event } starts { $in }, { $when }.
tg-guest-expired = Request expired: { $event }, { $when }. The host did not reply in time; please book another time.
tg-button-cancel = Cancel booking
tg-button-reschedule = Reschedule
tg-button-calendar = Add to calendar
tg-button-pick-time = Pick a new time
tg-button-rebook = Book another time
tg-button-bookings = Open bookings
tg-host-new-booking = New booking: { $event }, { $when }. Guest: { $guest }, { $email }.
tg-host-approval-request = Request to approve: { $event }, { $when }. Guest: { $guest }, { $email }.
tg-host-reschedule-approval = Approve a new time: { $event }, { $when }. Guest: { $guest }, { $email }.
tg-host-rescheduled = { $guest } moved { $event } to { $when }.
tg-host-cancelled = { $guest } cancelled: { $event }, { $when }.
tg-host-guest-linked = { $guest } will get Telegram notifications for { $event }, { $when }: { $username }.
tg-host-guest-linked-differs = Entered on the form: { $entered }.
tg-host-no-username = no username
tg-bot-help = This bot sends notifications about bookings on { $site }. To turn them on, open the link on your confirmation page or in your email; for a paid booking, the link is on its payment page. /stop turns them off. Once a booking is linked to this chat, you can also write to its host here.
tg-bot-linked-guest = Done: notifications about this booking will come here.
tg-bot-linked-host = Done: host notifications for { $name } will come here.
tg-bot-link-invalid = This link no longer works: it belongs to one Telegram chat and works only until the meeting ends.
tg-bot-link-replaced = This link has been replaced. Open your booking's payment page again and use the Telegram link there.
tg-bot-rate-limited = Too many attempts. Please try again later.
tg-bot-stopped = Notifications are off. To turn them on again for a booking, open its link from the confirmation email again; hosts create a new link in Profile & Settings → Telegram.
tg-bot-not-linked = This chat gets no notifications.
tg-cmd-help = How this bot works
tg-cmd-stop = Stop notifications
tg-book-label = Telegram
tg-book-placeholder = @username
tg-book-help-optional = After booking, you can turn on notifications in Telegram.
tg-book-help-required = After booking, open the Telegram link to get notifications.
tg-book-invalid-title = Check your Telegram username
tg-book-invalid = Enter a Telegram username like @name (letters, digits and _, 4 to 32 characters).
tg-book-required = Enter your Telegram username.
tg-confirmed-title = Get notifications in Telegram
tg-confirmed-body = Open the bot and press the button at the bottom of the chat: notifications about this booking will come there.
tg-confirmed-button = Open Telegram
tg-email-button = Notifications in Telegram
tg-email-plain = Get notifications in Telegram: { $url }
tg-settings-nav = Telegram notifications
tg-settings-title = Telegram
tg-settings-intro = Get host notifications in your own Telegram chat: new bookings, requests to approve, cancellations and moves by guests.
tg-settings-off = Telegram is not configured on this server.
tg-settings-no-bot-name = The bot is not ready yet. Please try again in a minute.
tg-settings-linked = Host notifications go to { $username }. Linked: { $date }.
tg-settings-not-linked = Your Telegram is not linked.
tg-settings-link = Link my Telegram
tg-settings-link-open = Open Telegram
tg-settings-link-expires = In the chat, press the button at the bottom. The link works once within 15 minutes.
tg-settings-link-limited = Too many links this hour. Please try again later.
tg-settings-unlink = Unlink
tg-settings-modes = Guest Telegram per meeting type
tg-settings-modes-help = Optional or required: the booking form asks for the guest's Telegram, and the confirmation offers the link to the bot.
tg-settings-modes-none = No meeting types to set.
tg-settings-mode-off = Off
tg-settings-mode-optional = Optional
tg-settings-mode-required = Required
tg-settings-save = Save
tg-settings-saved = Saved.
tg-settings-team = Team { $team }
tg-dashboard-label = Telegram { $username }
tg-dashboard-entered = (entered { $entered })
tg-dashboard-not-linked = (not linked yet)
tg-bot-already-linked = Already on: notifications about this booking come to this chat.
tg-bot-link-invalid-host = This link has expired: links from Settings work once within 15 minutes. Create a new one in Profile & Settings → Telegram.
tg-settings-linked-no-username = Host notifications go to your Telegram chat. Linked: { $date }.
tg-bot-description = This bot sends notifications about your bookings on { $site }: confirmations, changes and reminders. Press the button below to turn them on for your booking. /stop turns them off.
tg-bot-short-description = Booking notifications from { $site }.
tg-button-join = Join

## Telegram: TON payments (ilia fork; docs/ton.md)

tg-guest-payment-due = Your booking awaits payment: { $event }, { $when }. Amount: { $amount } TON (≈ ${ $usd }). Pay by { $deadline }, otherwise the time is released.
tg-guest-payment-partial = Part of the payment for { $event }, { $when } has arrived: { $paid } of { $amount } TON. Pay the remaining { $rest } TON by { $deadline }, otherwise the time is released.
tg-guest-payment-received = Payment received, your booking is confirmed: { $event }, { $when }. Host: { $host }.
tg-guest-payment-late = A payment of { $value } TON for { $event }, { $when } arrived after the deadline, so it did not confirm the booking. { $host } will contact you about a refund.
tg-guest-payment-expired = The time to pay for { $event }, { $when } has run out, so the time is released. You can book another time.
tg-guest-payment-taken = The time you chose for { $event }, { $when } was taken before the booking was paid, so the booking was released. You do not need to pay. You can book another time.
tg-guest-payment-refund = We received your payment for { $event }, { $when }, but the time could not be confirmed, so the booking was released. { $host } will refund the payment and contact you.
tg-guest-payment-refund-part = Part of the payment had already arrived: { $host } will contact you about a refund.
tg-guest-payment-held = We received your payment for { $event }, { $when }, but the booking could not be confirmed automatically. { $host } will contact you.
tg-button-pay = Pay
tg-host-payment-received = Paid: { $event }, { $when }. Guest: { $guest }, { $email }. Received { $paid } TON (≈ ${ $usd }); the booking is confirmed.
tg-host-payment-problem = Payment needs attention: { $event }, { $when }. Guest: { $guest }, { $email }.
tg-host-payment-transfer = Transfer: { $value } TON.
tg-host-payment-amounts = Received { $paid } of { $amount } TON.
tg-host-payment-guest-telegram = Guest's Telegram: { $username }.

## ilia fork: booking flow v2 (docs/ton.md, docs/operations.md)
troubleshoot-label-busy = Busy
ton-etf-currency = Currency
ton-etf-currency-ton = TON
ton-etf-currency-usd = USD (converted to TON)
ton-etf-price-help-ton = In TON: from 1 to 100000, the guest pays exactly this amount (100 TON when left blank).
ton-etf-usd-no-rates = Exchange rates are not set up on this server (CALRS_TON_RATE_MIN_USD and CALRS_TON_RATE_MAX_USD), so a price in USD cannot be booked. Set the price in TON.
ton-form-error-price-ton = Enter a price in TON from 1 to 100000, with at most two decimals.
ton-form-error-currency = Choose TON or USD as the currency of the price.
ton-form-error-telegram = Choose off, optional or required for the guest’s Telegram.
ton-etf-telegram = Guest’s Telegram
ton-etf-telegram-help = Required by default for a paid booking: the guest then gets payment reminders and can be reached in Telegram. You can make it optional or turn it off.
ton-slots-price-ton = Price: { $ton } TON
ton-email-amount-value-ton = { $amount } TON
tg-guest-payment-due-ton = Your booking awaits payment: { $event }, { $when }. Amount: { $amount } TON. Pay by { $deadline }, otherwise the time is released.
tg-host-payment-received-ton = Paid: { $event }, { $when }. Guest: { $guest }, { $email }. Received { $paid } TON; the booking is confirmed.
ton-pay-renewed = The deadline was extended ({ $n } of { $max }).
ton-dash-badge-renewed = deadline extended { $n } of { $max }
ton-email-renewed-subject = Your time is still held: { $event }
ton-email-renewed-message = We have not received the payment for { $event } with { $host } yet, so your time is held until { $deadline }. If you have already paid, no action is needed: the payment will be counted. Otherwise, pay on the page below by then, or cancel the booking if your plans have changed.
tg-guest-payment-renewed = The payment for { $event }, { $when } has not arrived yet, so the time is held for you until { $deadline }. Pay the { $rest } TON still due by then, otherwise the time is released. If you have already paid, no action is needed: the payment will be counted.

## ilia fork: messages with guests (docs/operations.md)
msg-page-title = Messages
msg-back = Bookings
msg-guest = Guest: { $guest }
msg-sent = Message sent.
msg-empty = No messages yet.
msg-label = Message to the guest
msg-help-email = The guest receives it by email from the system address; their replies appear here and in your Telegram.
msg-help-no-email = Email is not set up on this server.
msg-help-telegram = The guest has linked Telegram and receives it there too.
msg-help-no-telegram = The guest has not linked Telegram.
msg-help-limits = At most 20 messages per booking a day, 5 of them by email.
msg-send = Send
msg-link = Messages
msg-link-count = Messages ({ $count })
msg-from-host = Host
msg-from-guest = Guest
msg-from-attendee = Attendee: { $address }
msg-from-other = Another address, not the guest's: { $address }
msg-via-dashboard = dashboard
msg-via-mcp = agent
msg-via-telegram = Telegram
msg-status-email-queued = email queued
msg-status-email-sent = email sent
msg-status-email-failed = email not delivered
msg-status-telegram-queued = Telegram queued
msg-status-telegram-sent = sent to Telegram
msg-status-telegram-failed = not delivered to Telegram
msg-error-empty = Write a message first.
msg-error-too-long = A message can have at most 2000 characters.
msg-error-limit = The daily message limit for this booking is reached. Try again tomorrow.
msg-error-no-channel = The guest cannot be reached: email is not set up on this server and the guest has not linked Telegram.
msg-error-closed = Messages can be sent only for pending or confirmed bookings, up to a week after the meeting.
msg-email-subject = Message from { $host }: { $event }, { $date }
msg-email-intro = { $host } sent you a message about { $event }:
msg-email-footer = To answer { $host }, reply to this email.
tg-guest-host-message = Message from { $host } about { $event }, { $when }:
tg-guest-reply-hint = To answer, write your reply here.
tg-host-guest-message = Message from { $guest } ({ $email }) about { $event }, { $when }:
tg-host-reply-hint = Reply to this message to answer the guest.
tg-bot-message-sent = Your message has been passed on to the host.
tg-bot-message-pick-booking = Several bookings are linked to this chat. Reply to a message about the booking you mean.
tg-bot-message-closed = This booking no longer takes messages.
tg-bot-message-after-payment = Messages to the host open once the booking is paid.
tg-bot-message-limit = Too many messages. Please try again later.
tg-bot-reply-sent = Sent to the guest.
tg-bot-reply-not-linked = This guest has not linked Telegram, so the reply was not sent. Write to them from the booking in the dashboard.
tg-bot-reply-not-allowed = This reply was not sent: the booking is not yours or no longer takes messages.
msg-error-internal = The message could not be saved, so nothing was sent. Please try again.
msg-link-new = Messages ({ $count } new)
msg-recent-title = Recent meetings
msg-recent-help = Meetings that have started or ended in the last week. You can still exchange messages with their guests.
tg-bot-message-sent-shortened = Your message has been passed on to the host, shortened to { $max } characters.
tg-bot-reply-too-long = The reply was not sent: a message can have at most { $max } characters.
tg-bot-reply-empty = The reply was not sent: it has no visible text.
tg-bot-reply-failed = The reply could not be saved, so it was not sent. Please try again.
troubleshoot-not-synced = Calendars are read only from { $from } to { $until }; events outside these days are not loaded, so none are shown here.
event-type-form-horizon-capped =
    { $days ->
        [one] This server offers slots at most { $days } day ahead, whatever the booking horizon says.
       *[other] This server offers slots at most { $days } days ahead, whatever the booking horizon says.
    }

## ilia fork: the system mailbox (src/inbox)
msg-help-email-no-inbox = The guest receives it by email from the system address. Email replies are not read on this server.
msg-via-email = email
msg-email-footer-no-reply = The booking system sent this message on behalf of { $host }; replies to it are not read.
tg-host-guest-email = Email from { $guest } ({ $email }) about { $event }, { $when }:
tg-host-attendee-email = Email from { $sender }, an additional attendee of the booking with { $guest } ({ $email }), about { $event }, { $when }:
tg-host-other-email = Email from another address, not the guest's: { $sender }, in the thread of the booking with { $guest } ({ $email }) about { $event }, { $when }:
booking-blocked-body-system = The host requires at least { $minutes } minutes of notice. If you cannot attend, please write to <a href="{ $href }">{ $address }</a>.
booking-blocked-body-no-reply = The host requires at least { $minutes } minutes of notice. If you cannot attend, please let the host know another way.

## ilia fork: passkeys and security keys (docs/passkeys-plan.md)

ilia-passkey-sign-in = Sign in with passkey
ilia-mfa-login-help-passkey = Confirm this sign-in with your passkey or security key. The sign-in expires after 10 minutes.
ilia-mfa-use-totp = Use authenticator code
ilia-mfa-use-recovery = Use recovery code
ilia-mfa-use-passkey = Use passkey
ilia-mfa-use-passkey-instead = Use a passkey or security key instead
ilia-mfa-totp-login-help = Enter the six-digit code from your authenticator app. The sign-in expires after 10 minutes.
ilia-mfa-recovery-code = Recovery code
ilia-mfa-recovery-login-help = Enter one of the recovery codes you saved. Each code works only once. The sign-in expires after 10 minutes.
ilia-passkey-unsupported = This browser cannot use passkeys or security keys. Choose another method below, or open this page in an up-to-date browser.
ilia-passkey-noscript = Passkeys need JavaScript. Turn it on for this site, or choose another method below.
ilia-passkey-cancelled = The request was cancelled or timed out. Try again, or choose another method.
ilia-passkey-invalid = The passkey or security key could not be verified, or this request expired. Try again, or choose another method.
ilia-passkey-key-type = This browser cannot read this type of key, so it cannot be added here. Try another browser, such as an up-to-date Firefox or Chrome, or choose another method.
ilia-passkey-wrong-site = Passkeys work only at { $origin }. Open this page at that address and try again, or choose another method.
ilia-mfa-enroll-help = Choose how you will confirm each sign-in. A passkey or security key is the safest choice: it works only on this site, so a fake copy of the page cannot use it.
ilia-passkey-add-security-key = Add a security key
ilia-passkey-add-device = Add a passkey on this device or phone
ilia-passkey-kinds-help = A security key is a USB or NFC key such as a YubiKey. A passkey on this device or phone uses Touch ID, Face ID, Windows Hello or the phone’s screen lock, and may be synced to your other devices.
ilia-mfa-use-app-instead = Use an authenticator app instead
ilia-passkey-name = Key name
ilia-passkey-name-help = Optional, for example “YubiKey on keyring” or “MacBook”. Up to 64 characters.
ilia-passkey-default-security-key = Security key { $n }
ilia-passkey-default-device = Passkey { $n }
ilia-passkey-register-help = Press a button and follow your browser: insert and touch the security key, or confirm with Touch ID, Face ID, Windows Hello or your phone.
ilia-passkey-exists = This key is already registered.
ilia-mfa-recovery-help = Save these recovery codes somewhere safe; they are shown only once. Each code replaces a passkey or an authenticator code for one sign-in. New codes replace all previous ones.
ilia-mfa-status-on = Two-factor authentication is on.
ilia-mfa-status-off = Two-factor authentication is off.
ilia-mfa-setup-intro = Add a second step to every sign-in: a passkey or security key (YubiKey, Touch ID, Face ID, Windows Hello, your phone), or an authenticator app.
ilia-passkey-list = Passkeys and security keys
ilia-passkey-none = No passkeys or security keys yet.
ilia-passkey-kind-hardware = Hardware key
ilia-passkey-kind-synced = Synced passkey
ilia-passkey-added = Added { $date }
ilia-passkey-last-used = Last used { $date }
ilia-passkey-never-used = Not used yet
ilia-passkey-other-site = Registered for { $rp_id }; it does not work at this site address.
ilia-passkey-rename = Rename
ilia-passkey-delete = Remove
ilia-passkey-add = Add a passkey or security key
ilia-passkey-new-name = New name
ilia-passkey-limit = You have reached the maximum number of keys ({ $max }). Remove one to add another.
ilia-passkey-backup-advice = Add a second key and keep it somewhere safe, so that losing one key does not lock you out.
ilia-passkey-unavailable = Passkeys are off: the server’s public address (CALRS_BASE_URL) is not an https:// address. Ask the administrator. Keys added before are listed and can be removed.
ilia-mfa-totp-title = Authenticator app
ilia-mfa-totp-on = An authenticator app is set up.
ilia-mfa-totp-off = No authenticator app is set up.
ilia-mfa-totp-add = Set up authenticator app
ilia-mfa-totp-remove = Remove authenticator app
ilia-mfa-recovery-title = Recovery codes
ilia-mfa-recovery-left =
    { $count ->
        [one] { $count } recovery code left
       *[other] { $count } recovery codes left
    }
ilia-mfa-last-factor = Two-factor authentication is required, so your last sign-in method cannot be removed. Add another one first.
ilia-mfa-last-factor-off = This is your last method: removing it turns two-factor authentication off and deletes your recovery codes.
ilia-mfa-changes-help = Every change here needs your password and a passkey or code, and signs out your other sessions.
ilia-mfa-confirm-title = Confirm it’s you
ilia-mfa-confirm-help = Enter your password, then confirm with a passkey or a code.
ilia-passkey-confirm = Confirm with passkey
ilia-mfa-confirm-code = Confirm with code
ilia-mfa-action-passkey-add = You are adding a passkey or security key.
ilia-mfa-action-passkey-rename = You are renaming the key “{ $name }”.
ilia-mfa-action-passkey-delete = You are removing the key “{ $name }”.
ilia-mfa-action-totp-add = You are setting up an authenticator app.
ilia-mfa-action-totp-remove = You are removing the authenticator app.
ilia-mfa-action-regenerate = You are replacing all recovery codes.
ilia-mfa-action-policy = You are setting the two-factor policy for local accounts to:
ilia-mfa-policy-continue = Continue

## ilia fork: the instance name (CALRS_BRAND_NAME) and the error page (src/web/error_page.rs)
ilia-login-subtitle = Sign in to your { $brand } account
ilia-register-subtitle = Register for a new { $brand } account
ilia-error-not-found = Not found
ilia-error-not-found-body = There is nothing at this address. Check the link.
ilia-error-gone = Link no longer valid
ilia-error-bad-request = Invalid request
ilia-error-forbidden = Access restricted

# ilia fork: TON price by demand (the ladder)
ton-slots-ladder-now = Now: { $ton } TON
ton-slots-ladder-now-free = Now: free, the host confirms the request
ton-slots-ladder-locked = The price is fixed when you book.
ton-slots-ladder-summary-free = Free while the calendar is empty, then { $from } to { $to } TON as it fills up.
ton-slots-ladder-summary = { $from } to { $to } TON, depending on how full the calendar is.
ton-slots-ladder-all-free = Free; the host confirms each request.
ton-slots-ladder-fee = The network fee is paid by your wallet.
ton-quote-title = The price has changed
ton-quote-was = You were shown { $ton } TON.
ton-quote-was-free = You were shown a free booking.
ton-quote-now = The price now: { $ton } TON.
ton-quote-now-free = Now it is free: the host confirms the request.
ton-quote-why = The calendar or the prices changed since the page was loaded. Nothing has been booked yet.
ton-quote-why-free-used = You already had a free booking recently, so this booking is paid. Nothing has been booked yet.
ton-quote-confirm-help = Please confirm the price. Nothing has been booked yet.
ton-quote-confirm = Book for { $ton } TON
ton-quote-confirm-free = Request it for free
ton-bae-body-free-used = You already had a free booking recently, so this one cannot be free. Please try again later.
ton-form-error-mode = Choose a fixed price or the price by demand.
ton-etf-mode = Pricing
ton-etf-mode-fixed = Fixed price
ton-etf-mode-ladder = By demand (the site’s price ladder, in TON)
ton-etf-ladder-now = Current ladder:
ton-etf-ladder-help = The price is the step for the number of your upcoming bookings in the coming days (confirmed or paid ones, free requests awaiting your approval, and team bookings you host; unpaid ones do not count): free at first (you confirm each free booking), then higher as your calendar fills up. Guests see the price now and pay the price they were shown. The ladder is set by the administrator.

# ilia fork: the TON settings page (/dashboard/admin/ton)
nav-admin-ton = TON payments
ton-etf-ladder-edit = Edit the ladder
## ilia-ov-ton
ton-admin-debug-tg-saved-locked = Nothing changed: the guest Telegram username is locked.
dbg-err-requote = The price changed twice while booking. Nothing was booked: try again.
ton-admin-page-title = TON payments
ton-admin-heading = TON payments
ton-admin-intro = These settings apply to TON payments on this server. A value set in the server's environment wins, then one saved here or by config apply, then the default.
ton-admin-saved-live = Saved. The new settings apply now; open invoices keep their amounts and deadlines.
ton-admin-saved-off = Saved. Payments are off on this server (CALRS_TON_NETWORK is not set); the settings apply once they are on.
ton-admin-saved-restart = Saved. The settings take effect after calrs restarts.
ton-admin-saved-invalid = Saved, but the server's settings are invalid as a whole, so payments stay off. See the status below.
ton-admin-saved-none = Nothing changed.
ton-admin-errors = Nothing was saved: correct the fields marked below.
ton-admin-status = Status
ton-admin-status-on = Payments are on
ton-admin-status-off = Payments are off
ton-admin-network = Network
ton-admin-address = Wallet address
ton-admin-address-raw = Raw form
ton-admin-indexer = Indexer
ton-admin-watcher = Watcher
ton-admin-watcher-ready = ready

## ilia-ov3-ton
ton-admin-notready-wallet = the receiving wallet is not active (or not checked yet)
ton-admin-notready-network = the indexer is on another network (or not checked yet)
ton-admin-notready-stale = the watcher has not completed a step in the last 2 minutes
ton-admin-off-unset = CALRS_TON_NETWORK is not set

ton-admin-watcher-not-ready = not ready: { $reason }
ton-admin-last-step = last step
ton-admin-open-invoices = Open invoices
ton-admin-api-key = Indexer API key
ton-admin-coingecko-key = CoinGecko key
ton-admin-secret-credentials = set (systemd credentials)
ton-admin-secret-env = set (environment)
ton-admin-not-set = not set
ton-admin-env-only = The network, the wallet address, the indexer and the API keys are set only in the server's environment and systemd credentials, never on this page.
ton-admin-src-env = set by the environment ({ $var })
ton-admin-src-apply = set by config apply
ton-admin-src-panel = set here
ton-admin-src-default = default
ton-admin-locked-env = To change it here, remove { $var } from the server's environment and restart calrs.
ton-admin-locked-apply = Change it in the config apply document, or leave it out of its ton section to hand it to this page.
ton-admin-default-hint = Leave empty for the default: { $value }.
ton-admin-blank-default = Leave empty for none.
ton-admin-group-pricing = Price by demand
ton-admin-group-timing = Invoice timing
ton-admin-group-rate = Exchange rate (USD prices)
ton-admin-group-alerts = Alerts
ton-admin-f-ladder = Ladder (TON by the number of bookings)
ton-admin-f-free_per_guest_days = One free booking per guest within (days, 0 = no limit)
ton-admin-f-window_min = Payment window (minutes)
ton-admin-f-grace_min = Grace after the window (minutes)
ton-admin-f-fuse_hours = Safety fuse after the grace (hours)
ton-admin-f-renew_min_lead_hours = Renew an unpaid window when the meeting is at least this far away (hours)
ton-admin-f-renew_window_min = Renewed window (minutes)
ton-admin-f-max_renewals = Renewals per invoice
ton-admin-f-underpay_tolerance_ton = Accepted underpayment (TON)
ton-admin-f-rate_min_usd = Lowest accepted rate (USD per TON)
ton-admin-f-rate_max_usd = Highest accepted rate (USD per TON)
ton-admin-f-rate_max_spread_pct = Largest spread between the sources (%)
ton-admin-f-rate_max_age_min = Oldest accepted rate (minutes)
ton-admin-f-rate_max_jump_pct = Largest jump from the last rate (%)
ton-admin-f-alert_email = Alert email (empty: every admin)
ton-admin-ladder-help = Steps separated by commas: the price at 0, 1, 2… confirmed bookings in the coming days; the last step is the ceiling. "free" only at the start: a free booking waits for the owner's approval.
ton-admin-ladder-row = Bookings: %1
ton-admin-ladder-add = Add a step
ton-admin-ladder-remove = Remove
ton-admin-free = free
ton-admin-preview = Preview
ton-admin-preview-button = Preview the ladder
ton-admin-previewing = A preview of the ladder entered above. Nothing is saved yet.
ton-admin-ladder-invalid = The ladder in effect is invalid: { $error }
ton-admin-preview-n = Bookings ahead
ton-admin-preview-price = Price
ton-admin-preview-or-more = { $n } or more
ton-admin-my-demand = Your bookings ahead now: { $n }. Your ladder price now: { $price }.
ton-admin-types = Priced event types
ton-admin-types-none = No event type has a price.
ton-admin-type = Event type
ton-admin-type-owner = Owner
ton-admin-type-price = Price
ton-admin-type-now = Now
ton-admin-mode-ladder = by demand
ton-admin-type-off = price switched off
ton-admin-type-below-ceiling = At or below the ladder's ceiling: an "any time" price is normally above it.
ton-admin-save = Save
ton-admin-audit = Recent changes
ton-admin-audit-none = No changes yet.
ton-admin-audit-at = When
ton-admin-audit-who = Who
ton-admin-audit-key = Setting
ton-admin-audit-change = Change
ton-admin-audit-default = (default)
ton-admin-via-apply = config apply

## ilia fork: the debug booking page (CALRS_DEBUG_BOOKING, src/web/debug_booking.rs)

nav-admin-debug-booking = Debug booking
dbg-title = Debug booking
dbg-intro = Book a test meeting as a guest would, without filling in the guest form each time. The booking goes through the same path as the public booking page: the checks, the price and the TON invoice, the hold on the time and the emails. Payment is never skipped or marked as received: pay the invoice from your wallet, or cancel the booking.
dbg-switch-note = Switch this page off once debugging is done, with the switch at the top of the page:
dbg-cancelled = Test bookings cancelled: { $count }.
dbg-no-types = You have no enabled personal event type.
dbg-event-type = Event type
dbg-priced = priced
dbg-private-short = private
dbg-choose = Show
dbg-price = Price
dbg-price-now-free = free now (you approve the request)
dbg-price-now-ton = { $ton } TON now
dbg-price-ladder = by demand, fixed at booking
dbg-price-free-used = this guest already had a free booking recently, so the first price applies
dbg-free = free
dbg-private = This event type is not public: a one-time invite for your own address is made for each test booking.
dbg-ton-off = TON payments are off on this server, so a priced event type cannot be booked now.
dbg-captcha-on = Captcha is on for booking forms. This page cannot solve it, so the booking path refuses test bookings while captcha is on.
dbg-err-username = Your account has no username, so it has no booking pages to book on.
dbg-slot = Time
dbg-tz = Times in { $tz }, your profile's time zone.
dbg-no-slots = No free time in the next 31 days.
dbg-name = Name
dbg-email = Email
dbg-email-help = Only your own addresses: your sign-in email and your booking email.
dbg-telegram-username = Telegram username
dbg-telegram-required = required by this event type
dbg-notes = Notes
dbg-notes-default = Test booking from the debug page.
dbg-own-chat = Telegram: deliver guest messages to my own linked Telegram chat
dbg-own-chat-off = Telegram is off on this server (no bot token).
dbg-own-chat-linked = Your linked chat: { $who }.
dbg-own-chat-help = The guest side of the booking (what to pay, the confirmation, the reminder, a cancellation) goes to the chat the bot already writes to you in, as if you had opened the guest's link there. Unticked, the payment page shows the guest's link as usual.
dbg-own-chat-not-linked = Your Telegram chat is not linked. Link it first, then come back:
dbg-own-chat-no-username = no username
dbg-test-note = This is a real booking, marked as a test. While it stands it holds the time, counts toward the demand price and sends the usual emails and messages to your own address. Cancel it when you are done.
dbg-submit = Book as a guest
dbg-list = My test bookings
dbg-list-none = No test bookings yet.
dbg-col-when = When
dbg-col-type = Event type
dbg-col-status = Status
dbg-col-payment = Payment
dbg-col-telegram = Telegram
dbg-open-pay = payment page
dbg-own-chat-yes = my chat
dbg-cancel-all = Cancel all my test bookings ({ $count })
dbg-cancel-all-help = Cancels every test booking of yours that still stands (a pending one is declined) the way the bookings dashboard does: open invoices are closed and the usual notices are sent. Other bookings are not touched.
dbg-cancel-reason = Test booking cancelled from the debug page.
dbg-impersonating = Stop impersonating to use the debug booking page.
dbg-err-type = Choose one of your own event types.
dbg-err-email = Use one of your own addresses.
dbg-err-slot = Choose a time from the list.
dbg-err-own-chat = Link your Telegram chat first, or untick the Telegram box.
dbg-own-chat-failed = The test booking was made, but its guest messages could not be routed to your Telegram chat: the chat was unlinked, the bot was stopped, or the link expired meanwhile. The booking stands and holds the time: open its payment page from the list below, or cancel it.
dbg-st-pending = pending
dbg-st-confirmed = confirmed
dbg-st-cancelled = cancelled
dbg-st-declined = declined
dbg-inv-open = awaiting payment
dbg-inv-paid = paid
dbg-inv-confirmed = paid and confirmed
dbg-inv-expired = expired
dbg-inv-closed = closed
dbg-inv-waived = confirmed without payment
dbg-banner = DEBUG · test booking
dbg-badge-test = TEST
ton-admin-debug-title = Debug
ton-admin-debug-intro = Test booking: book your own event types the way a guest does (the real price, invoice, payment page and Telegram) without typing the guest form. Admins only. While it is off, its page answers 404 and nothing links to it.
ton-admin-debug-state-on = Test booking: on
ton-admin-debug-state-off = Test booking: off
ton-admin-debug-turn-on = Turn on
ton-admin-debug-turn-off = Turn off
ton-admin-debug-open = Open test booking
ton-admin-debug-locked-env = Locked by the server's environment ({ $var }={ $value }). To switch it here, remove { $var } from the environment and restart calrs.
ton-admin-debug-locked-apply = Locked by config apply (site.ton.debug_booking). Change it in the config apply document, or leave the key out to hand it to this page.
ton-admin-debug-saved-on = Test booking is on.
ton-admin-debug-saved-off = Test booking is off.
ton-admin-debug-saved-locked = Nothing changed: the switch is locked.
ton-admin-f-debug_booking = Test booking
dbg-telegram-from-chat = Prefilled from your linked Telegram chat.
dbg-telegram-from-setting = Prefilled from the Debug block of
dbg-telegram-none = No username to prefill: set one in the Debug block of
dbg-timing = Timing for this test booking
dbg-timing-normal = Normal: the site's settings (the TON payments page)
dbg-timing-fast = Fast test: { $window } min to pay; renewals: { $renewals }, { $renew } min each (the guest is asked to pay again); then the invoice expires and the booking is cancelled automatically. Grace { $grace } min, margin { $margin } min.
dbg-timing-custom = Custom, in minutes (an empty field takes the fast value):
dbg-timing-window = Window to pay, min
dbg-timing-renew-window = Renewed window, min
dbg-timing-renewals = Renewals
dbg-timing-help = For this test booking's invoice only, and it may be shorter than the site allows (at least { $min } min). The window still ends before the meeting's minimum notice. Unpaid, the booking is cancelled automatically: the watcher checks about every 10 seconds while it runs (less often only if the TON indexer rate-limits or rejects it), and calrs frees the time within a minute after the invoice expires. Payment is never marked as received.
dbg-err-timing = Choose a timing. Custom minutes: window 2–240, renewed window 2–1440, renewals 0–5.
dbg-col-timing = Timing
dbg-timing-site = site settings
dbg-timing-summary = test: { $window } min, renewals { $renewals } × { $renew } min
dbg-renewals = renewed { $n } of { $max }
dbg-cd-until = pay by { $expires } (a transfer counts until { $settle })
dbg-cd-pay = { $time } left to pay
dbg-cd-grace = window ended; a transfer sent in the next { $time } still counts
dbg-cd-wait = window ended: renewed or expired at the watcher's next step
dbg-list-live = While a test booking is pending, this list refreshes itself every 10 seconds.
dbg-guest-lang = Guest language
dbg-guest-lang-help = The language of the guest's pages, emails and Telegram messages for this test booking. By default, your own interface language. Emails to hosts stay in each host's own language.
dbg-err-lang = Choose a guest language from the list.
dbg-col-lang = Language
dbg-no-chat-title = Your Telegram chat is not linked
dbg-no-chat-body = “Deliver guest messages to my own linked Telegram chat” is not available until you link your chat in Profile & Settings → Telegram. Link it there, then come back to this page.
dbg-no-chat-link = Link my Telegram chat
ton-admin-debug-tg-label = Guest Telegram username for test bookings
ton-admin-debug-tg-save = Save
ton-admin-debug-tg-help = Prefilled on the debug booking page when your linked Telegram chat has no username (for event types that ask for one). Without @; empty: none.
ton-admin-debug-tg-locked-apply = Locked by config apply (site.ton.debug_booking_telegram). Change it in the config apply document, or leave the key out to hand it to this page.
ton-admin-debug-tg-saved = Guest Telegram username saved.
ton-admin-debug-tg-invalid = Nothing saved: not a Telegram username (a Latin letter, then letters, digits and _, 4 to 32 characters).
ton-admin-f-debug_booking_telegram = Test booking: guest Telegram username

## ilia fork: event type translations (src/web/event_texts.rs)
etf-translations = Translations
etf-translations-help = Guests reading the booking pages in one of these languages see this title and description instead of the ones above, in their emails and messages too. Every other language shows the ones above.
etf-translation-language = Language
etf-translation-description-help = Supports **bold**, *italic*, and [links](url). Left empty, this language shows no description.
etf-translation-add = Add a language
etf-translation-remove = Remove
form-error-translation-language = Choose a language for each translation.
form-error-translation-duplicate = Each language can have only one translation.
form-error-translation-title-required = Each translation needs a title.
form-error-translation-title-length = A translated title must be at most 255 characters.

## ilia fork: 3CX video meetings (src/web/threecx.rs)
admin-threecx = 3CX video meetings
admin-threecx-help = Each confirmed booking of an event type whose location is “3CX meeting” gets its own scheduled meeting on this 3CX phone system. Guests get the meeting link; hosts get the organizer link in their calendar and emails. Moving or cancelling the booking moves or deletes the meeting. The guest’s email address is never sent to 3CX, so 3CX sends the guest nothing.
admin-threecx-base-url = Phone system address
admin-threecx-extension = Organizer extension
admin-threecx-extension-help = The extension the meetings belong to, for example 100.
admin-threecx-api-key = Video conference API key
admin-threecx-api-key-help = The key of the 3CX video conference API, from the 3CX admin console. It is stored encrypted and never shown again.
admin-threecx-display-name-help = What guests see as the place, for example “3CX”. Empty: “Video call”.
admin-threecx-save = Save 3CX settings
admin-threecx-test = Test connection
admin-threecx-clear = Remove 3CX settings
admin-threecx-notice-saved = 3CX settings saved.
admin-threecx-notice-cleared = 3CX settings removed, the key with them.
admin-threecx-notice-ok = 3CX answered: the address and the key work.
admin-threecx-notice-auth = 3CX refused the key. Calls to 3CX pause for an hour so that the phone system does not block this server; saving a new key or testing again ends the pause.
admin-threecx-notice-transport = 3CX did not answer. Check the address and that this server can reach it.
admin-threecx-notice-service = 3CX answered with an error. Check that video conferencing is enabled for the extension.
admin-threecx-notice-bad-link = 3CX answered with a meeting link on another address. Check the phone system address.
admin-threecx-notice-not-set = Enter the address, the extension and the key first.
admin-threecx-notice-bad-base-url = Not saved: the address must be https:// and the host only, without a path.
admin-threecx-notice-bad-extension = Not saved: the extension must be 1 to 10 digits.
admin-threecx-notice-bad-api-key = Not saved: the key must be 16 to 256 characters without spaces.
admin-threecx-notice-bad-display-name = Not saved: the label must be at most 100 characters.
admin-threecx-notice-bad-pair = Not saved: the address and the extension go together, and a key needs both.
admin-threecx-notice-key-env = Not saved: the key is set by the deployment (CALRS_THREECX_API_KEY). Change it there.
admin-threecx-api-key-env = Set by the deployment (CALRS_THREECX_API_KEY). Change it in the deployment’s secrets and deploy.
admin-threecx-api-key-env-bad = CALRS_THREECX_API_KEY is set but is not a key (16 to 256 characters without spaces): 3CX meetings are off until the deployment fixes it.
admin-threecx-api-key-env-placeholder = Set by the deployment
etf-location-threecx = 3CX meeting (made for each booking)
etf-threecx-hint = A 3CX meeting is made for each confirmed booking and follows its moves and cancellation. Set up 3CX on the admin page.

## ilia fork: profile translations (src/web/user_texts.rs)
settings-translations-help = Guests reading your pages in one of these languages see this name, title and bio instead of the ones above, in their emails and messages too. A field left empty shows the one above; every other language shows the ones above.
settings-error-translation-name-length = A translated name must be at most 255 characters.
