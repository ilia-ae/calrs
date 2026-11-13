# Booking confirmation page (templates/confirmed.html)

confirmed-page-title-pending = Boeking in afwachting
confirmed-page-title-booked = Boeking bevestigd

confirmed-heading-reschedule-requested = Verzetten aangevraagd
confirmed-heading-rescheduled = Verzet!
confirmed-heading-pending = In afwachting van bevestiging
confirmed-heading-booked = Je afspraak staat!

confirmed-subtitle-reschedule-requested = Je verzoek om te verzetten is naar { $host } gestuurd. Je krijgt een e-mail op { $email } zodra het is goedgekeurd.
confirmed-subtitle-rescheduled = Je boeking is verzet. Er is een bevestiging gestuurd naar { $email }.
confirmed-subtitle-pending = Je boekingsaanvraag is naar { $host } gestuurd. Je krijgt een e-mail op { $email } zodra die is bevestigd.
confirmed-subtitle-booked = Er is een bevestiging gestuurd naar { $email }.

confirmed-detail-event = Afspraak:
confirmed-detail-date = Datum:
confirmed-detail-time = Tijd:
confirmed-detail-with = Met:
confirmed-detail-location = Locatie:
confirmed-detail-notes = Opmerkingen:
confirmed-detail-additional-guests = Extra gasten:

confirmed-book-another = Nog een tijd boeken

confirmed-add-to-calendar = Toevoegen aan agenda

# Slot picker (templates/slots.html)

## ilia-ov-slots
slots-time-format-24 = 24 u
slots-time-format-12 = 12 u
slots-prev-month = Vorige maand
slots-next-month = Volgende maand
slots-prev-week = Vorige week
slots-next-week = Volgende week
book-remove-guest = Gast verwijderen
ilia-error-back-to-site = Terug naar de website

slots-location-video = Videogesprek
slots-location-phone = Telefoongesprek
slots-location-google-meet = Google Meet

slots-tz-label = Jouw tijdzone
slots-time-format-label = Tijdnotatie

## ilia-ov5-guest
tz-city-midway = Midway
tz-city-hawaii = Hawaï
tz-city-alaska = Alaska
tz-city-los-angeles = Los Angeles
tz-city-denver = Denver
tz-city-chicago = Chicago
tz-city-new-york = New York
tz-city-sao-paulo = São Paulo
tz-city-cape-verde = Kaapverdië
tz-city-london = Londen
tz-city-paris = Parijs, Brussel
tz-city-helsinki = Helsinki, Kyiv
tz-city-moscow = Moskou
tz-city-dubai = Dubai
tz-city-india = India
tz-city-bangkok = Bangkok
tz-city-shanghai = Shanghai
tz-city-tokyo = Tokio
tz-city-sydney = Sydney
tz-city-auckland = Auckland

tz-group-common = Veelgebruikte tijdzones
tz-region-africa = Afrika
tz-region-america = Amerika
tz-region-antarctica = Antarctica
tz-region-arctic = Arctis
tz-region-asia = Azië
tz-region-atlantic = Atlantische Oceaan
tz-region-australia = Australië
tz-region-europe = Europa
tz-region-indian = Indische Oceaan
tz-region-pacific = Stille Oceaan
slots-offset-note-day = Tijden op %1 zijn in %2 (de klok wordt verzet op %3).
slots-offset-note-range = De klok wordt verzet op %1: vanaf dan zijn de tijden in %2.

slots-view-month = Maandweergave
slots-view-week = Weekweergave
slots-view-column = Kolomweergave

slots-weekday-mon = ma
slots-weekday-tue = di
slots-weekday-wed = wo
slots-weekday-thu = do
slots-weekday-fri = vr
slots-weekday-sat = za
slots-weekday-sun = zo

slots-weekday-mon-short = M
slots-weekday-tue-short = D
slots-weekday-wed-short = W
slots-weekday-thu-short = D
slots-weekday-fri-short = V
slots-weekday-sat-short = Z
slots-weekday-sun-short = Z

slots-select-date = Kies een datum

## ilia-ov3-guest
# The deferred slot load failed (slots.html); the booking form's TON price note (partials/ton_price.html).
slots-load-failed = Beschikbaarheid kon niet worden geladen. Vernieuw de pagina.
ton-book-price-note = betalen in TON na het boeken

slots-loading-availability = Beschikbaarheid laden...
slots-click-highlighted = Klik op een gemarkeerde datum om de beschikbare tijden te zien
slots-bookable-range-both = Boekbaar van { $from } tot en met { $until }
slots-bookable-range-from = Boekbaar vanaf { $from }
slots-bookable-range-until = Boekbaar tot en met { $until }
slots-no-longer-bookable = Deze afspraak is niet meer boekbaar.
slots-no-times-month = Geen beschikbare tijden deze maand
slots-no-times-day = Geen beschikbare tijden op deze dag
slots-no-availability-participants = Deze maand is er geen tijd waarop alle deelnemers beschikbaar zijn
slots-week-more = meer

# Booking form (templates/book.html)

book-page-title = { $title } boeken
book-back-to-times = Terug naar tijden
book-name-label = Je naam
book-name-placeholder = Sanne de Vries
book-email-label = E-mail
book-email-placeholder = sanne@example.com
book-email-invalid = Vul een volledig e-mailadres in, inclusief het domein (bijv. sanne@example.com).
book-notes-label = Opmerkingen
book-notes-optional = (optioneel)
book-notes-placeholder = Is er iets wat je wilt bespreken?
book-additional-guests-label = Extra gasten
book-additional-guests-hint = (optioneel, maximaal { $max })
book-add-guest-btn = + E-mailadres van gast toevoegen
book-guest-email-placeholder = collega@example.com
book-phone-label = Telefoonnummer
book-phone-placeholder = 06 12345678
book-phone-help = Een lokaal nummer is prima; we gaan uit van { $country }, tenzij je begint met +.
book-phone-optional-consequence = Laat dit leeg als je liever geen sms'jes over deze boeking ontvangt.
book-phone-required = Voor deze boeking is een telefoonnummer verplicht.
book-phone-invalid-title = Ongeldig telefoonnummer
book-phone-invalid = Vul een telefoonnummer in waar we een sms naartoe kunnen sturen, of laat het veld leeg.
book-phone-country-search = Zoeken
book-phone-country-label = Kies een land
book-phone-country-none = Geen land gekozen
book-phone-country-no-results = Geen landen gevonden
captcha-label = Beveiligingscontrole
captcha-initial-state = Bevestig dat je een mens bent
captcha-verifying = Controleren...
captcha-solved = Je bent een mens
captcha-error = Fout
captcha-troubleshooting = Probleemoplossing
captcha-wasm-disabled = Schakel WASM in om dit veel sneller op te lossen
captcha-verify-aria = Klik om te bevestigen dat je een mens bent
captcha-verifying-aria = Bezig met controleren, even geduld
captcha-verified-aria = Gecontroleerd
captcha-required = Bevestig dat je een mens bent
captcha-error-aria = Er is een fout opgetreden, probeer het opnieuw
book-confirm-button = Boeking bevestigen

# SMS notifications (src/sms/message.rs).
#
# These are text messages, billed per 160-character segment (70 if the text
# contains any character outside the GSM-7 alphabet, which includes most
# accented letters). Keep them short and plain.

sms-confirmed = Boeking bevestigd: { $event }, { $date } om { $time } ({ $tz }).
sms-cancelled = Boeking geannuleerd: { $event }, { $date } om { $time } ({ $tz }).
sms-rescheduled = Boeking verzet: { $event } is nu { $date } om { $time } ({ $tz }).
sms-reminder = Herinnering: { $event } begint { $date } om { $time } ({ $tz }).

# Shared labels used across the cancel / decline / approve / reschedule / claim flows

common-detail-guest = Gast:
common-detail-reason = Reden:
common-reason-optional = (optioneel)
common-close-page = Je kunt deze pagina sluiten.

# Cancel flow (booking_cancel_form.html, booking_cancelled_guest.html)

cancel-page-title = Boeking annuleren
cancel-heading = Boeking annuleren
cancel-subtitle = Je staat op het punt je boeking te annuleren.
cancel-reason-label = Reden
cancel-reason-placeholder-host = Laat de organisator weten waarom...
cancel-button = Boeking annuleren
cancelled-heading = Boeking geannuleerd
cancelled-subtitle = Je boeking is geannuleerd en de organisator is op de hoogte gebracht.

# Decline flow (booking_decline_form.html, booking_declined.html)

decline-page-title = Boeking afwijzen
decline-heading = Boeking afwijzen
decline-subtitle = Je staat op het punt deze boekingsaanvraag af te wijzen.
decline-reason-placeholder-guest = Laat de gast weten waarom...
decline-button = Boeking afwijzen
declined-heading = Boeking afgewezen
declined-subtitle = De boeking is afgewezen en de gast is op de hoogte gebracht.

# Approve flow (booking_approve_form.html, booking_approved.html)

approve-page-title = Boeking goedkeuren
approve-heading = Boeking goedkeuren
approve-subtitle = Je staat op het punt deze boekingsaanvraag goed te keuren.
approve-button = Boeking goedkeuren
approved-heading = Boeking goedgekeurd
approved-subtitle = De boeking is bevestigd en er is een bevestiging gestuurd naar { $email }.

# Claim flow (booking_claim_form.html, booking_claimed.html, booking_already_claimed.html)

claim-page-title = Boeking overnemen
claim-heading = Boeking overnemen
claim-subtitle = Je staat op het punt deze boeking over te nemen. Je wordt toegevoegd als deelnemer.
claim-assigned-to = Toegewezen aan:
claim-button = Deze boeking overnemen
claimed-page-title = Boeking overgenomen
claimed-heading = Boeking overgenomen
claimed-subtitle = Je hebt deze boeking overgenomen. Er is een agenda-uitnodiging naar je e-mailadres gestuurd.
already-claimed-page-title = Al overgenomen
already-claimed-heading = Al overgenomen
already-claimed-subtitle = Deze boeking is al overgenomen door { $name }.

# Generic error page (booking_action_error.html)

action-error-page-title = Fout bij boekingsactie

# Host-initiated reschedule (booking_host_reschedule.html)

host-resched-page-title = Boeking verzetten — calrs
host-resched-heading = Boeking verzetten
host-resched-subtitle = { $guest } krijgt dan een e-mail met het verzoek een nieuw tijdstip te kiezen.
host-resched-currently = Huidig tijdstip:
host-resched-button = Verzoek om te verzetten sturen
host-resched-cancel-link = Annuleren

# Guest reschedule confirmation (booking_reschedule_confirm.html)

resched-confirm-page-title = Verzetten bevestigen
resched-confirm-heading = Verzetten bevestigen
resched-confirm-subtitle = Je staat op het punt je boeking naar een nieuw tijdstip te verzetten.
resched-was = Was:
resched-new = Nieuw:
resched-button = Verzetten bevestigen
resched-back-to-picker = Terug naar tijdkeuze

# Base layout chrome (templates/base.html)

base-loader-checking = Beschikbaarheid controleren
base-loader-please-wait = Even geduld, de nieuwste agendagegevens worden geladen...
base-stop-impersonating = Stoppen met meekijken als gebruiker
base-theme-toggle = Thema wisselen
base-language-choose = Kies je taal
base-powered-by = Mogelijk gemaakt door

# Profile (templates/profile.html)

profile-pick-event-type-invite = Kies een afspraaktype om een tijd te boeken.
profile-no-event-type = Nog geen afspraaktypes beschikbaar.

# Month and weekday names + per-locale date format patterns.
# Used by server-side date formatters in src/i18n.rs.

common-month-1 = januari
common-month-2 = februari
common-month-3 = maart
common-month-4 = april
common-month-5 = mei
common-month-6 = juni
common-month-7 = juli
common-month-8 = augustus
common-month-9 = september
common-month-10 = oktober
common-month-11 = november
common-month-12 = december

common-weekday-long-mon = maandag
common-weekday-long-tue = dinsdag
common-weekday-long-wed = woensdag
common-weekday-long-thu = donderdag
common-weekday-long-fri = vrijdag
common-weekday-long-sat = zaterdag
common-weekday-long-sun = zondag

# Format patterns are parametric per locale to handle word order. Translators
# pick where each placeholder lands. Example outputs:
#   EN: April 2026  /  Tuesday, March 12, 2026
#   FR: avril 2026  /  mardi 12 mars 2026
#   ES: abril 2026  /  martes, 12 de marzo de 2026
common-format-month-year = { $month } { $year }
common-format-long-date = { $weekday } { $day } { $month } { $year }
common-format-date = { $day } { $month } { $year }

# Email signatures and shared bits (src/email.rs)

email-signature = — calrs
email-action-reschedule = Verzetten
email-action-cancel-booking = Boeking annuleren

# Email: guest booking confirmation

# Kept to "event — date": Exchange titles the guest appointment after the
# email Subject header, not the ICS SUMMARY (#157).
email-confirm-subject = { $event } — { $date }
email-confirm-greeting = Hoi { $name },
email-confirm-headline = Je boeking is bevestigd!
email-confirm-ics-attached-plain = Er is een agenda-uitnodiging bijgevoegd.
email-confirm-ics-attached-html = Er is een agenda-uitnodiging bij deze e-mail gevoegd.
email-confirm-need-to-cancel = Wil je annuleren? { $url }

# Email: guest reminder

email-reminder-subject = Herinnering: { $event } om { $time }
email-reminder-headline = Je afspraak komt eraan.

# Email: guest cancellation

email-cancel-subject = Geannuleerd: { $event } — { $date }
email-cancel-headline-by-host = Je boeking is geannuleerd door { $host }.
email-cancel-headline-by-guest = Je boeking is geannuleerd.
email-cancel-ics-attached-plain = Er is een agenda-annulering bijgevoegd.
email-cancel-ics-attached-html = Er is een agenda-annulering bij deze e-mail gevoegd.

# Confirmation email: notice-window policy lines (src/email.rs)

email-confirm-cancel-notice = Let op: annuleren kan tot uiterlijk { $minutes } minuten van tevoren.
email-confirm-reschedule-notice = Let op: verzetten kan tot uiterlijk { $minutes } minuten van tevoren.

# Event type form: Google Meet location + cancel/reschedule minimum notice
# (templates/event_type_form.html, src/google_meet.rs)

## ilia-ov-eventform
# The collapsed "Payment in TON" section's summary: no payment, or the price comes from the demand ladder.
ton-etf-hint-off = Gratis
ton-etf-hint-ladder = Naar vraag

event-type-form-location-google-meet = Google Meet (automatisch gegenereerde link)
event-type-form-location-google-meet-hint = Bij bevestiging wordt een unieke Google Meet-link aangemaakt, op naam van de toegewezen organisator. Elke organisator (jij, of elk geschikt teamlid) moet Google Calendar gekoppeld hebben met een agenda geselecteerd om naar terug te schrijven.

google-meet-prereq-no-host = Voor Google Meet is een organisator met een gekoppelde Google Calendar nodig.
google-meet-prereq-no-eligible = Voor Google Meet is minstens één geschikt teamlid met een gekoppelde Google Calendar nodig.
google-meet-prereq-missing = Voor Google Meet moet elke organisator Google Calendar gekoppeld hebben met een agenda geselecteerd om naar terug te schrijven. Ontbreekt nog bij: { $names }. Koppel ze via Dashboard → Agendabronnen.
google-meet-unavailable-title = Google Meet is niet beschikbaar
google-meet-dynamic-group-unavailable = De organisator moet Google Calendar gekoppeld hebben met een agenda geselecteerd om naar terug te schrijven.

event-type-form-cancel-notice-label = Minimale aankondigingstijd voor annuleren
event-type-form-reschedule-notice-label = Minimale aankondigingstijd voor verzetten
event-type-form-notice-help = Laat leeg voor geen beperking.
event-type-form-resources-label = Benodigde faciliteiten
event-type-form-resources-hint = Tijdsloten worden alleen aangeboden als de geselecteerde faciliteiten beschikbaar zijn, volgens de modus hieronder.
event-type-form-resources-mode-all = Alle geselecteerde faciliteiten moeten vrij zijn
event-type-form-resources-mode-round-robin = Eén vrije faciliteit is genoeg (die wordt aan de boeking toegewezen)
event-type-form-notice-unit-minutes = minuten
event-type-form-notice-unit-hours = uur
event-type-form-notice-unit-days = dagen
event-type-form-booking-horizon-label = Boekingshorizon
event-type-form-booking-horizon-help = Hoeveel dagen vooruit gasten kunnen boeken. Laat leeg voor geen limiet, 0 voor alleen vandaag.
event-type-form-bookable-range-label = Boekbare periode
event-type-form-bookable-from = van
event-type-form-bookable-until = tot en met
event-type-form-bookable-range-help = Alleen deze dagen zijn boekbaar. Laat leeg voor geen limiet. Gebruik uitzonderingen voor andere tijden op losse dagen.
event-type-form-bookable-range-past-warning = Deze periode is al voorbij. Het afspraaktype wordt niet meer openbaar getoond.
event-type-form-bookable-range-too-far-warning = Deze periode begint over meer dan een jaar. Gasten kunnen maximaal een jaar vooruit boeken, dus boeken kan vanaf { $date }.
event-type-form-bookable-range-horizon-warning = De boekingshorizon eindigt voordat deze periode begint. Gasten kunnen boeken vanaf { $date }.
event-type-form-bookable-range-bookings-outside =
    { $count ->
        [one] 1 bestaande boeking valt buiten de boekbare periode.
       *[other] { $count } bestaande boekingen vallen buiten de boekbare periode.
    }
event-type-form-bookable-range-bookings-link = Boekingen bekijken
event-type-form-saved = Opgeslagen.

# Booking confirmation: cancel/reschedule policy notices (templates/confirmed.html)

confirmed-cancel-notice-info = Annuleren kan tot uiterlijk { $minutes } minuten voor de afspraak.
confirmed-reschedule-notice-info = Verzetten kan tot uiterlijk { $minutes } minuten voor de afspraak.

# Booking action blocked page (templates/booking_action_blocked.html)

booking-blocked-title-cancel = Deze boeking kan niet meer online worden geannuleerd
booking-blocked-title-reschedule = Deze boeking kan niet meer online worden verzet
booking-blocked-body = De organisator vraagt minstens { $minutes } minuten van tevoren bericht. Kun je niet aanwezig zijn? Mail dan rechtstreeks naar <a href="mailto:{ $host_email }">{ $host_email }</a>.

# Dashboard event types listing (templates/dashboard_event_types.html)

## ilia-ov-dates
# Booking times on the dashboard lists: the day ("Today", "Tomorrow", a weekday
# or a date), then the start and end times and the host timezone.
dashboard-when-today = Vandaag
dashboard-when-tomorrow = Morgen
dashboard-when-range = { $date }, { $start } — { $end } { $tz }

dashboard-event-types-copy = Kopiëren
dashboard-event-types-copied = Gekopieerd!
dashboard-event-types-copy-title = Boekingslink kopiëren
dashboard-event-types-copy-failed = Kopiëren mislukt

# Dashboard sidebar and shared chrome (templates/dashboard_base.html)

## ilia-ov-shell
nav-menu = Menu
event-types-more-actions = Meer acties

nav-section-scheduling = Planning
nav-overview = Overzicht
nav-event-types = Afspraaktypes
nav-bookings = Boekingen
nav-teams = Teams
nav-section-shared-links = Gedeelde links
nav-invite-links = Uitnodigingslinks
nav-section-calendars = Agenda's
nav-sources = Bronnen
nav-section-personal = Persoonlijk
nav-settings = Profiel en instellingen
nav-troubleshoot = Probleemoplossing
nav-section-admin = Beheer
nav-admin-panel = Beheerpaneel
nav-sign-out = Uitloggen
nav-release-notes = Release notes bekijken

# Timezone mismatch banner (templates/dashboard_base.html)

tz-banner-text = De tijdzone van je browser is { $detected }, maar je boekingstijdzone staat ingesteld op { $current }.
tz-banner-update = Bijwerken
tz-banner-dismiss = Sluiten

# Markdown editor toolbar (templates/dashboard_base.html)

editor-link-prompt = Voer een URL in:
editor-link-default-label = linktekst
editor-placeholder-text = tekst
editor-nothing-to-preview = Niets om te bekijken

# Dashboard overview (templates/dashboard_overview.html)

overview-page-title = Dashboard
overview-welcome = Welkom, { $name }
overview-public-page = Openbare pagina:
overview-avail-banner-title = Standaardbeschikbaarheid
overview-avail-banner-body = Je standaardwerktijden zijn ingesteld op ma–vr, 9:00–17:00. Deze worden gebruikt wanneer anderen je toevoegen aan dynamische groepsafspraken.
overview-avail-banner-cta = Controleer je beschikbaarheid
overview-dismiss = Sluiten
overview-getting-started = Aan de slag
overview-getting-started-help = Volg deze stappen om boekingen te gaan ontvangen.
overview-step-connect-calendar = Koppel een agenda
overview-step-first-event-type = Maak je eerste afspraaktype aan
overview-step-share-link = Deel je boekingslink
overview-pending-approval = In afwachting van goedkeuring
overview-booking-with = { $title } met { $guest }
overview-badge-pending = in afwachting
overview-guest-booked = Geboekt door gast:
overview-confirm = Bevestigen
overview-decline = Afwijzen
overview-stat-event-types = Afspraaktypes
overview-stat-upcoming = Komende boekingen
overview-stat-pending = In afwachting van goedkeuring
overview-stat-sources = Agendabronnen
overview-quick-actions = Nieuw afspraaktype aanmaken
overview-action-public-title = Openbare boekingspagina
overview-action-public-desc = Deel een link — iedereen kan een tijdslot kiezen en een afspraak met je boeken.
overview-action-team-title = Teamplanning
overview-action-team-desc = Verdeel boekingen over teamleden of vind een tijd waarop iedereen kan.
overview-action-team-desc-empty = Maak eerst een team aan en stel daarna gedeelde afspraaktypes in.
overview-action-private-title = Privé, alleen op uitnodiging
overview-action-private-desc = Genereer eenmalige links voor specifieke contacten. Niemand anders kan boeken.
overview-action-shared-title = Gedeelde uitnodigingslinks
overview-action-shared-desc = Elke collega in het team kan boekingslinks genereren om extern te delen.
overview-action-reason-calendar = Koppel eerst een agenda
overview-action-reason-ask-admin = Vraag een beheerder om een team aan te maken
overview-action-reason-team-admin = Vereist een team — maak er eerst een aan
overview-action-reason-team-member = Vereist een team — vraag een beheerder

# Dashboard bookings (templates/dashboard_bookings.html)

## ilia-ov-lists
team-form-slug-placeholder = engineering
troubleshoot-label-past = Al voorbij
troubleshoot-legend-beyond-horizon = Buiten horizon of periode

bookings-page-title = Boekingen
bookings-pending-approval = In afwachting van goedkeuring
bookings-available-to-claim = Beschikbaar om over te nemen
bookings-upcoming = Komende boekingen
bookings-with = { $title } met { $guest }
bookings-guest-booked = Geboekt door gast:
bookings-resource = Faciliteit:
bookings-confirm = Bevestigen
bookings-reschedule = Verzetten
bookings-decline = Afwijzen
bookings-claim = Overnemen
bookings-badge-awaiting-reschedule = wacht op nieuw tijdstip
bookings-badge-writeback-failed = niet in je agenda
bookings-writeback-failed-detail = Schrijven naar de agenda mislukt: { $error }
bookings-cancel = Annuleren
bookings-reason-placeholder = Reden (optioneel)
bookings-confirm-cancel = Annulering bevestigen
bookings-back = Terug
bookings-empty = Nog geen komende boekingen.<br>Deel je { $link } zodat mensen een afspraak met je kunnen boeken.
bookings-empty-link-label = links naar afspraaktypes

# Dashboard teams listing (templates/dashboard_teams.html)

teams-page-title = Teams
teams-heading = Teams
teams-new = Nieuw
teams-badge-public = openbaar
teams-badge-private = privé
teams-settings = Instellingen
teams-view = Bekijken
teams-empty = Nog geen teams.
teams-empty-admin = { $link } om samen te werken met je team.
teams-empty-admin-link-label = Maak er een aan
teams-empty-member = Teams worden aangemaakt door beheerders. Vraag je beheerder om er een aan te maken en jou als lid toe te voegen.

# Dashboard invite links (templates/dashboard_internal.html)

invite-links-page-title = Uitnodigingslinks
invite-links-heading = Uitnodigingslinks
invite-links-new = Nieuwe interne afspraak
invite-links-help = Genereer eenmalige boekingslinks voor interne afspraaktypes. Elke ingelogde collega kan hier links aanmaken en delen.
invite-links-duration = { $minutes } min
invite-links-hosted-by = Georganiseerd door { $host }
invite-links-get-link = Link ophalen
invite-links-invites = Uitnodigingen
invite-links-empty = Nog geen interne afspraaktypes.<br>{ $link } met zichtbaarheid "Intern" zodat elke collega boekingslinks kan genereren.
invite-links-empty-link-label = Maak een afspraaktype aan
invite-links-js-generating = Bezig met genereren...
invite-links-js-copied = Gekopieerd!
invite-links-js-error = Fout

teams-member-count =
    { $count ->
        [one] { $count } lid
       *[other] { $count } leden
    }

# Dashboard calendar sources (templates/dashboard_sources.html)

sources-page-title = Agendabronnen
sources-heading = Agendabronnen
sources-add = Toevoegen
sources-last-sync = Laatst gesynchroniseerd:
sources-sync-failing = Synchronisatie mislukt sinds { $since } UTC: { $error }
sources-sync = Synchroniseren
sources-full-resync = Volledig opnieuw synchroniseren
sources-full-resync-title = Cache wissen en alle afspraken opnieuw ophalen van de server
sources-test = Testen
sources-reconnect = Opnieuw koppelen
sources-reconnect-title = De Google-toestemmingsprocedure opnieuw doorlopen
sources-edit = Bewerken
sources-remove = Verwijderen
sources-remove-confirm = Bron '{ $name }' verwijderen? Hiermee worden alle gesynchroniseerde afspraken van deze bron verwijderd.
sources-no-write-calendar = Geen agenda gekozen om naar te schrijven. Bevestigde boekingen blijven in calrs en worden niet naar deze agenda gezet. Kies hieronder een agenda om terugschrijven in te schakelen.
sources-write-bookings-to = Boekingen schrijven naar:
sources-write-none = Geen (niet schrijven)
sources-empty = Geen agendabronnen gekoppeld. { $link } om beschikbaarheid te controleren.
sources-empty-link-label = Voeg er een toe
sources-calendars = Agenda's
sources-calendars-synced = Gesynchroniseerde agenda's: { $synced } van { $total }
source-calendars-page-title = Te synchroniseren agenda's
source-calendars-heading = Te synchroniseren agenda's
source-calendars-back = Terug naar agendabronnen
source-calendars-help = Kies welke agenda's van { $name } calrs synchroniseert. Niet-aangevinkte agenda's worden nooit opgehaald en blokkeren je beschikbaarheid niet.
source-calendars-select-all = Alles selecteren
source-calendars-select-none = Niets selecteren
source-calendars-write-badge = Boekingen worden hier geschreven
source-calendars-write-hint = Bevestigde boekingen worden in deze agenda geschreven, of die nu gesynchroniseerd wordt of niet.
source-calendars-save = Opslaan
source-calendars-saved = Opgeslagen. Nieuw aangevinkte agenda's worden bij de volgende synchronisatie opgehaald; gebruik Synchroniseren op de bronnenpagina om ze nu op te halen.
source-calendars-empty = Nog geen agenda's gevonden voor deze bron. Start een synchronisatie om ze te vinden.

# Dashboard event types listing (templates/dashboard_event_types.html)

event-types-page-title = Afspraaktypes
event-types-heading = Afspraaktypes
event-types-new = Nieuw
event-types-badge-disabled = uitgeschakeld
event-types-badge-internal = intern
event-types-badge-private = privé
event-types-badge-resources = faciliteiten
event-types-badge-active = actief
event-types-badge-expired = verlopen
event-types-badge-range-from = vanaf { $date }
event-types-badge-range-until = t/m { $date }
event-types-send-invites = Uitnodigingen versturen
event-types-duration = { $minutes } min
event-types-mode-collective = collectief
event-types-mode-round-robin = round-robin
event-types-edit = Bewerken
event-types-disable = Uitschakelen
event-types-enable = Inschakelen
event-types-embed = Embed
event-types-overrides = Uitzonderingen
event-types-team-settings = Teaminstellingen
event-types-invites = Uitnodigingen
event-types-view-public = Openbare pagina bekijken
event-types-view-page = Pagina bekijken
event-types-delete = Verwijderen
event-types-delete-confirm = Afspraaktype '{ $title }' verwijderen? Dit kan niet ongedaan worden gemaakt.
event-types-duplicate = Dupliceren
event-types-duplicate-title = { $title } (kopie)
event-types-empty = Nog geen afspraaktypes. { $link } om boekingen te gaan ontvangen.
event-types-empty-link-label = Maak er een aan

# Markdown editor toolbar (templates/settings.html, templates/team_form.html)

editor-bold = Vet (Ctrl+B)
editor-italic = Cursief (Ctrl+I)
editor-strikethrough = Doorhalen
editor-code = Inline code
editor-link = Link invoegen (Ctrl+K)
editor-toggle-preview = Voorbeeld aan/uit
editor-preview = Voorbeeld

# Profile and settings (templates/settings.html)

## ilia-ov-settings
settings-choose-file = Bestand kiezen
settings-no-file = Geen bestand gekozen
admin-provider-local = Wachtwoord
admin-provider-oidc = SSO
admin-theme-vates-desc = Rood en ruimteblauw

settings-page-title = Instellingen
settings-heading = Profiel en instellingen
settings-public-page-label = Je openbare boekingspagina
settings-copy = Kopiëren
settings-copied = Gekopieerd!
settings-open = Openen
settings-avatar = Avatar
settings-upload = Uploaden
settings-remove = Verwijderen
settings-display-name = Weergavenaam
settings-display-name-placeholder = Je naam
settings-username = Gebruikersnaam
settings-username-hint = (gebruikt in je boekings-URL)
settings-username-pattern-title = Alleen kleine letters, cijfers en streepjes
settings-username-help = Je openbare boekingspagina:
settings-title = Functie
settings-title-placeholder = bijv. Software Engineer, Product Manager
settings-title-help = Getoond op je openbare profiel en in de zijbalk.
settings-bio = Bio
settings-bio-placeholder = Vertel iets over jezelf...
settings-bio-help = Getoond op je openbare boekingspagina. Ondersteunt **vet**, *cursief*, ~~doorhalen~~, `code` en [links](url).
settings-booking-email = E-mailadres voor boekingen
settings-booking-email-help = Dit e-mailadres verschijnt op je openbare boekingspagina's en in e-mailmeldingen. Laat leeg om je inlog-e-mailadres te gebruiken.
settings-booking-email-warning = Zorg dat dit e-mailadres bestaat bij je e-mailprovider. Anders worden meldingen niet afgeleverd.
settings-timezone = Tijdzone
settings-timezone-help = Je beschikbaarheidsregels en boekingstijden worden in deze tijdzone berekend.
settings-language = Taal
settings-language-auto = Automatisch (browserstandaard)
settings-language-help = Kies een taal voor de interface, of laat op Automatisch staan om de instelling van je browser te volgen.
settings-public-page-language = Taal van openbare boekingspagina's
settings-public-page-language-visitor = De taal van elke bezoeker
settings-public-page-language-host = Altijd mijn taal
settings-public-page-language-help = Taal van je openbare profiel, je boekingspagina's en de e-mails die je gasten ontvangen. "Altijd mijn taal" gebruikt de taal die je hierboven kiest.
settings-error-public-page-language = Kies een geldige taal voor de openbare boekingspagina's.
settings-dynamic-group = Anderen mogen mij toevoegen aan dynamische groepslinks
settings-dynamic-group-help = Als dit is ingeschakeld, kunnen andere gebruikers ad-hoc collectieve afspraak-URL's maken waar jij in zit (bijv. { $example }).
settings-lend-resource = Mijn agendatoegang uitlenen voor het reserveren van faciliteiten
settings-lend-resource-help = Als een boeking een gedeelde faciliteit moet reserveren (demolab, vergaderruimte) waar jouw agenda-account naar kan schrijven, mag calrs je opgeslagen agendagegevens daarvoor gebruiken.
settings-default-availability = Standaardbeschikbaarheid
settings-default-availability-help = Je standaardwerktijden. Worden gebruikt voor dynamische groepslinks wanneer anderen je aan een afspraak toevoegen.
settings-copy-to-all = Naar alle dagen kopiëren
settings-copy-to-all-title = Kopieer de tijdvakken van de eerste ingeschakelde dag naar alle andere ingeschakelde dagen
settings-add-window = Tijdvak toevoegen
settings-remove-window = Tijdvak verwijderen
settings-save = Instellingen opslaan
settings-appearance = Weergave
settings-theme-system = Systeem
settings-theme-light = Licht
settings-theme-dark = Donker

# Sign in (templates/auth/login.html)

## ilia-ov-auth
ilia-mfa-recovery-copy = Codes kopiëren
ilia-mfa-recovery-copied = Codes gekopieerd
ilia-mfa-recovery-copy-manual = Kopiëren lukte niet. De codes zijn geselecteerd: kopieer ze met de hand.

login-page-title = Inloggen
login-heading = Inloggen
login-subtitle = Log in op je calrs-account
login-sso = Inloggen met SSO
login-or = of
login-email = E-mail
login-password = Wachtwoord
login-submit = Inloggen met e-mail
login-no-account = Nog geen account? { $link }
login-register-link = Registreren

# Registration (templates/auth/register.html)

register-page-title = Registreren
register-heading = Account aanmaken
register-subtitle = Registreer een nieuw calrs-account
register-domains-limited = Registratie is beperkt tot: { $domains }
register-name = Naam
register-name-placeholder = Je naam
register-email = E-mail
register-password = Wachtwoord
register-password-hint = (min. 12 tekens)
register-submit = Account aanmaken
register-have-account = Heb je al een account? { $link }
register-signin-link = Inloggen

# Authentication errors (src/auth.rs)

auth-error-rate-limited = Te veel inlogpogingen. Probeer het later opnieuw.
auth-error-invalid-credentials = Ongeldig e-mailadres of wachtwoord
auth-error-internal = Interne fout
auth-error-registration-disabled = Registratie is uitgeschakeld.
auth-error-name-length = Naam moet tussen 1 en 255 tekens lang zijn
auth-error-email-length = E-mailadres moet tussen 1 en 255 tekens lang zijn
auth-error-email-invalid = Voer een geldig e-mailadres in
auth-error-email-domain = E-maildomein niet toegestaan
auth-error-password-length = Wachtwoord moet minimaal 12 tekens lang zijn
auth-error-email-taken = E-mailadres is al geregistreerd
auth-error-create-failed = Account aanmaken mislukt

# Calendar source test and write-back setup (templates/source_test.html, templates/source_write_setup.html)

source-test-page-title = Agendabron
source-test-sync-heading = Synchronisatie: { $name }
source-test-heading = Verbindingstest
source-write-page-title = Terugschrijven naar agenda instellen
source-write-back = Terug naar dashboard
source-write-heading = Waar moeten boekingen naartoe?
source-write-help = Als iemand een afspraak met je boekt, kan calrs die automatisch in je agenda zetten. Kies naar welke agenda boekingen voor { $name } worden geschreven.
source-write-save = Opslaan
source-write-skip = Nu overslaan
source-write-sync-results = Synchronisatieresultaten

source-write-event-count =
    { $count ->
        [one] { $count } afspraak
       *[other] { $count } afspraken
    }

# Date overrides (templates/overrides.html)

overrides-page-title = Datumuitzonderingen
overrides-heading = Datumuitzonderingen
overrides-back-teams = Terug naar teams
overrides-back-event-types = Terug naar afspraaktypes
overrides-intro = Voeg uitzonderingen voor specifieke datums toe voor { $title }
overrides-add-heading = Nieuwe uitzondering toevoegen
overrides-date = Datum
overrides-type = Soort uitzondering
overrides-type-blocked = Hele dag blokkeren
overrides-type-custom = Aangepaste tijden
overrides-start-time = Begintijd
overrides-end-time = Eindtijd
overrides-add-submit = Uitzondering toevoegen
overrides-existing = Bestaande uitzonderingen
overrides-badge-blocked = geblokkeerd
overrides-badge-custom = aangepaste tijden
overrides-delete = Verwijderen
overrides-delete-confirm = Deze uitzondering verwijderen?
overrides-empty = Nog geen datumuitzonderingen.<br>Gebruik het formulier hierboven om specifieke datums te blokkeren (feestdagen, vrije dagen) of aangepaste tijden in te stellen.

# Public team page (templates/team_profile.html)

team-profile-subtitle = Kies een afspraaktype om een tijd te boeken.
team-profile-empty = Nog geen afspraaktypes beschikbaar.

# Availability troubleshoot (templates/troubleshoot.html, src/web/mod.rs)

## ilia-ov2-lists
# The troubleshoot timeline's legend entry for time already gone today.
troubleshoot-legend-past = Verstreken tijd
# Accessible names of the troubleshoot page's day arrows.
troubleshoot-prev-day = Vorige dag
troubleshoot-next-day = Volgende dag
# Accessible name of a member's role select on the team settings page.
team-settings-role-of = Rol van { $name }

troubleshoot-page-title = Beschikbaarheid controleren
troubleshoot-empty = Geen afspraaktypes gevonden. { $link } om je beschikbaarheid te controleren.
troubleshoot-empty-link-label = Maak er een aan
troubleshoot-subtitle = Bekijk waarom tijdsloten beschikbaar of geblokkeerd zijn voor { $title }
troubleshoot-duration = { $minutes } min
troubleshoot-buffer-before = { $minutes } min buffertijd vooraf
troubleshoot-buffer-after = { $minutes } min buffertijd achteraf
troubleshoot-min-notice = { $minutes } min aankondigingstijd
troubleshoot-blocked-override = Geblokkeerd door datumuitzondering (vrije dag)
troubleshoot-custom-hours-active = Uitzondering met aangepaste tijden actief (vervangt wekelijkse regels)
troubleshoot-legend-available = Beschikbaar
troubleshoot-legend-calendar-event = Agenda-afspraak
troubleshoot-legend-booking = Boeking
troubleshoot-legend-resource = Faciliteit bezet
troubleshoot-legend-outside = Buiten werktijden
troubleshoot-legend-buffer = Buffertijd / min. aankondigingstijd
troubleshoot-blocked-slots = Geblokkeerde tijdsloten
troubleshoot-none-date-blocked = Deze datum is geblokkeerd door een beschikbaarheidsuitzondering (vrije dag). Geen tijdsloten beschikbaar.
troubleshoot-none-custom-hours = Uitzondering met aangepaste tijden actief, maar zonder passende tijdvakken. Controleer je uitzonderingsinstellingen.
troubleshoot-none-no-rules = Geen beschikbaarheidsregels voor deze dag van de week. Dit afspraaktype is niet te boeken op { $date }.
troubleshoot-none-all-bookable = Geen geblokkeerde tijdsloten binnen de beschikbare uren. Alle tijden zijn te boeken.
troubleshoot-label-outside = Buiten beschikbaarheid
troubleshoot-label-available = Beschikbaar
troubleshoot-label-min-notice = Min. aankondigingstijd ({ $minutes } min)
troubleshoot-label-beyond-horizon = Buiten boekingshorizon ({ $days } dagen)
troubleshoot-label-outside-date-range = Buiten de boekbare periode
troubleshoot-label-buffer = Buffertijd ({ $minutes } min)
troubleshoot-label-resource-busy = Faciliteit bezet: { $names }
troubleshoot-detail-around = Rond: { $label }
troubleshoot-detail-around-booking = Rond boeking van { $guest }
troubleshoot-reason-calendar-event = Agenda-afspraak: { $label }
troubleshoot-reason-booking = Boeking: { $label }

# Invite management (templates/invite_form.html)

invites-heading = Uitnodigingen
invites-back-teams = Terug naar teams
invites-back-event-types = Terug naar afspraaktypes
invites-intro = Verstuur uitnodigingslinks voor { $title }
invites-capped = <strong>Invoer is beperkt tot { $max } ontvangers per keer.</strong> Verstuur de rest in een volgende ronde.
invites-failed-hint = — bekijk de serverlogs voor details.
invites-quick-link = Snelle link
invites-quick-link-help = Genereer een eenmalige link en kopieer die naar je klembord.
invites-get-link = Link ophalen
invites-or-email = Of verstuur per e-mail
invites-recipients = Ontvangers
invites-recipients-hint = (één e-mailadres per regel, max. { $max })
invites-message = Persoonlijk bericht
invites-message-hint = (optioneel, wordt naar elke ontvanger gestuurd)
invites-message-placeholder = Ik laat je graag een demo zien...
invites-expires-in = Verloopt over
invites-expires-days = { $days } dagen
invites-expires-never = Nooit
invites-allow-multiple = Meerdere boekingen per ontvanger toestaan
invites-send = Uitnodigingen versturen
invites-sent-heading = Verstuurde uitnodigingen
invites-badge-expired = verlopen
invites-badge-used = gebruikt
invites-badge-active = actief
invites-sent-by = Verstuurd door { $name }
invites-uses = { $used }/{ $max } keer gebruikt
invites-expires-at = Verloopt { $date }
invites-copy-link = Link kopiëren
invites-delete = Verwijderen
invites-delete-confirm = Deze uitnodiging verwijderen?
invites-empty = Nog geen uitnodigingen verstuurd. Gebruik het formulier hierboven om iemand een boekingslink te sturen.
invites-js-generating = Genereren...
invites-js-copied = Gekopieerd!
invites-js-error = Fout

invites-sent-count =
    { $count ->
        [one] { $count } uitnodiging verstuurd.
       *[other] { $count } uitnodigingen verstuurd.
    }

invites-skipped-invalid =
    { $count ->
        [one] { $count } ongeldige regel overgeslagen:
       *[other] { $count } ongeldige regels overgeslagen:
    }

invites-skipped-duplicate =
    { $count ->
        [one] { $count } dubbele regel overgeslagen:
       *[other] { $count } dubbele regels overgeslagen:
    }

invites-failed =
    { $count ->
        [one] { $count } uitnodiging mislukt (DB of SMTP):
       *[other] { $count } uitnodigingen mislukt (DB of SMTP):
    }

# Calendar source form (templates/source_form.html)

source-form-title-edit = Agendabron bewerken
source-form-title-add = Agenda toevoegen
source-form-heading-edit = Agendabron bewerken
source-form-heading-add = Een agenda koppelen
source-form-subtitle-edit = Werk de verbinding bij. Laat het wachtwoord leeg om het huidige te behouden. Synchroniseer na het wijzigen van de URL of gebruikersnaam om de lijst met gevonden agenda's te vernieuwen.
source-form-subtitle-add = Koppel een CalDAV-server of Microsoft Exchange (EWS) zodat calrs je beschikbaarheid kan controleren wanneer gasten een afspraak boeken.
source-form-backend = Backend

## ilia-ov3-dash
# The "Other / generic CalDAV" preset on the new-source form; it also becomes the default source name.
source-form-preset-other = Andere CalDAV-server

source-form-preset = Voorinstelling
source-form-connect-google = Koppelen met Google
source-form-google-unavailable = Google Agenda is niet beschikbaar. Neem contact op met je beheerder.
source-form-name = Weergavenaam
source-form-name-placeholder = Mijn agenda
source-form-url-caldav = CalDAV-URL
source-form-url-ews = URL van EWS-endpoint
source-form-username = Gebruikersnaam
source-form-password = Wachtwoord
source-form-password-keep = Laat leeg om het huidige te behouden
source-form-password-placeholder = App-wachtwoord of accountwachtwoord
source-form-skip-test = Verbindingstest overslaan
source-form-skip-test-help = Gebruik dit als de test blijft hangen (komt vaak voor bij sommige BlueMind/Zimbra-installaties). Je kunt de verbinding later testen.
source-form-save = Wijzigingen opslaan
source-form-add = Agendabron toevoegen
source-form-help-google-configured = Klik op de knop hieronder om calrs toegang te geven tot je Google Agenda.
source-form-help-google-unconfigured = De koppeling met Google Agenda is nog niet ingesteld. Vraag je beheerder om Google OAuth2-gegevens in te stellen in het beheerpaneel.

# Calendar source form: provider help (templates/source_form.html)

source-form-help-bluemind = <strong>BlueMind</strong> — Gebruik het DAV-endpoint van je BlueMind-server.<br> Meestal: <code>https://mail.yourcompany.com/dav/</code><br> De gebruikersnaam is je <strong>e-mailadres</strong> (bijv. <code>alice@yourcompany.com</code>), niet alleen de inlognaam.<br> Blijft de verbindingstest hangen? Vink dan "Verbindingstest overslaan" aan en probeer direct te synchroniseren.
source-form-help-nextcloud = <strong>Nextcloud</strong> — Gebruik de WebDAV-root, niet de URL van een specifieke agenda.<br> Meestal: <code>https://cloud.example.com/remote.php/dav</code>
source-form-help-fastmail = <strong>Fastmail</strong> — Gebruik je volledige e-mailadres in het URL-pad.<br> Voorbeeld: <code>https://caldav.fastmail.com/dav/calendars/user/you@fastmail.com/</code><br> Gebruik een app-specifiek wachtwoord (Settings &rarr; Privacy &amp; Security &rarr; Integrations).
source-form-help-icloud = <strong>iCloud</strong> — Gebruik <code>https://caldav.icloud.com/</code><br> Je hebt een app-specifiek wachtwoord nodig van <a href="https://appleid.apple.com" target="_blank" style="color: var(--accent);">appleid.apple.com</a> (Inloggen en beveiliging &rarr; App-specifieke wachtwoorden).
source-form-help-zimbra = <strong>Zimbra</strong> — Gebruik het DAV-endpoint van je Zimbra-server.<br> Meestal: <code>https://mail.example.com/dav/</code>
source-form-help-sogo = <strong>SOGo</strong> — Gebruik het SOGo-DAV-endpoint.<br> Meestal: <code>https://mail.example.com/SOGo/dav/</code>
source-form-help-radicale = <strong>Radicale</strong> — Gebruik de root-URL van de server.<br> Meestal: <code>https://cal.example.com/</code>
source-form-help-baikal = <strong>Baikal</strong>: Gebruik het DAV-endpoint van je Baikal-server.<br> Meestal: <code>https://cal.example.com/dav.php/</code>
source-form-help-exchange = <strong>Microsoft Exchange (EWS)</strong>. Gebruik het SOAP-endpoint:<br> <code>https://mail.example.com/EWS/Exchange.asmx</code><br> De gebruikersnaam is het e-mailadres van de mailbox; voor dit account moet inloggen met HTTP Basic-authenticatie via TLS zijn toegestaan (schakel dit in op een servicemailbox als je tenant Basic heeft uitgeschakeld).<br> Kies ook <strong>Microsoft Exchange (EWS)</strong> in het Backend-menu hierboven.
source-form-help-google = <strong>Google Agenda</strong>: koppelen via OAuth2. Geen wachtwoord nodig.<br>
source-form-help-other = Voer de <strong>DAV-root-URL</strong> van je CalDAV-server in — niet een specifieke agenda of openbare link.<br> calrs vindt je agenda's automatisch via PROPFIND (RFC 4791).

# Markdown editor toolbar, short labels (templates/team_form.html, templates/team_settings.html)

editor-bold-short = Vet
editor-italic-short = Cursief
editor-link-short = Link invoegen

# Team creation (templates/team_form.html)

team-form-heading = Nieuw team
team-form-name = Teamnaam
team-form-name-placeholder = Engineering
team-form-slug = Slug
team-form-slug-hint = (URL-vriendelijke naam)
team-form-slug-pattern-title = Alleen kleine letters, cijfers en streepjes
team-form-description = Beschrijving
team-form-optional = (optioneel)
team-form-description-placeholder = Waar dit team zich mee bezighoudt...
team-form-description-help = Wordt getoond op de teampagina. Ondersteunt **vet**, *cursief* en [links](url).
team-form-visibility = Zichtbaarheid
team-form-public = Openbaar
team-form-private = Privé
team-form-visibility-help = Privéteams krijgen een uitnodigingstoken om te delen. Openbare teams zijn zichtbaar op de teamprofielpagina.
team-form-members = Leden
team-form-members-help = Je wordt automatisch toegevoegd als teambeheerder. Voeg losse gebruikers toe of koppel OIDC-groepen.
team-form-search-placeholder = Zoek gebruikers of groepen...
team-form-search-users = Gebruikers
team-form-search-groups = OIDC-groepen
team-form-you = (jij)
team-form-submit = Team aanmaken

# Team settings (templates/team_settings.html)

team-settings-page-title = Instellingen
team-settings-subtitle = Teaminstellingen — teambeheerders kunnen deze bewerken.
team-settings-public-url = Openbare URL
team-settings-public-url-help = Iedereen kan via deze link boeken.
team-settings-invite-link = Uitnodigingslink
team-settings-invite-link-help = Deel deze link om mensen toegang te geven tot de boekingspagina van dit privéteam.
team-settings-avatar = Teamavatar
team-settings-profile = Profiel
team-settings-description-placeholder = Vertel mensen over dit team...
team-settings-description-help = Wordt getoond op de openbare boekingspagina van het team. Ondersteunt **vet**, *cursief* en [links](url).
team-settings-visibility-help = Openbare teams worden vermeld op de teamprofielpagina. Voor privéteams is een uitnodigingslink nodig.
team-settings-members-help = Beheer wie bij dit team hoort. Voeg losse gebruikers toe of koppel OIDC-groepen voor automatische synchronisatie.
team-settings-role-member = Lid
team-settings-role-admin = Beheerder
team-settings-oidc-group = OIDC-groep
team-settings-remove = Verwijderen
team-settings-save = Wijzigingen opslaan
team-settings-danger-zone = Gevarenzone
team-settings-danger-help = Verwijder dit team permanent. Afspraaktypes worden losgekoppeld (niet verwijderd). Dit kan niet ongedaan worden gemaakt.
team-settings-delete = Dit team verwijderen
team-settings-delete-confirm = Team '{ $name }' verwijderen? Dit kan niet ongedaan worden gemaakt.

# Event type form (templates/event_type_form.html)

etf-heading-edit = Afspraaktype bewerken
etf-heading-new = Nieuw afspraaktype
etf-disabled-banner = Dit afspraaktype is uitgeschakeld, dus niemand kan het nog boeken.
etf-team = Team
etf-team-hint = (optioneel — laat leeg voor een persoonlijk afspraaktype)
etf-team-personal = Persoonlijk
etf-scheduling-mode = Planningsmodus
etf-mode-round-robin = Round-robin — toewijzen aan één beschikbaar lid
etf-mode-collective = Collectief — alle leden moeten beschikbaar zijn
etf-scheduling-mode-help = Round-robin wijst toe aan één beschikbaar lid (minst drukke eerst). Collectief vereist dat alle leden tegelijk vrij zijn.
etf-title = Titel
etf-title-placeholder = Kennismakingsgesprek van 30 min
etf-slug = Slug
etf-slug-placeholder = automatisch gegenereerd uit titel
etf-description-placeholder = Een kort kennismakingsgesprek om te bespreken...
etf-description-help = Wordt getoond op de boekingspagina. Ondersteunt **vet**, *cursief* en [links](url).
etf-location = Locatie
etf-location-link = Videogesprek (vaste URL)
etf-location-jitsi = Jitsi (automatisch aangemaakte ruimte)
etf-location-webhook = Webhook (eigen provider)
etf-location-phone = Telefoon
etf-location-in-person = Op locatie
etf-location-custom = Aangepast
etf-location-details = Details
etf-location-details-placeholder = https://meet.example.com/my-room
etf-pattern-placeholder = Laat leeg om het standaardpatroon van de organisatie te gebruiken
etf-duration = Duur (minuten)
etf-slot-interval = Tijdslotinterval (minuten)
etf-slot-interval-placeholder = Gelijk aan duur
etf-slot-interval-help = Hoe vaak een tijdslot begint. Laat leeg om de duur aan te houden.
etf-required-members = Verplichte leden
etf-required-members-help = Alle aangevinkte leden moeten vrij zijn voordat een tijdslot wordt aangeboden. Vink leden uit die je wilt uitsluiten (hun beschikbaarheid wordt dan genegeerd).
etf-member-priority = Prioriteit van leden
etf-member-priority-help = Leden met een hogere prioriteit krijgen boekingen als eerste toegewezen als ze beschikbaar zijn. Gelijke prioriteit = verdeeld op basis van het aantal recente boekingen.
etf-member-timezone-title = Tijdzone van het lid. Diens persoonlijke werktijden worden in deze tijdzone geïnterpreteerd.
etf-priority-high = Hoog
etf-priority-medium = Gemiddeld
etf-priority-low = Laag
etf-section-availability = Beschikbaarheid
etf-timezone-help = De tijden hieronder gelden in deze tijdzone. Kies voor team-afspraaktypes de werktijdzone van het team (niet per se die van de maker).
etf-reset-default = Terugzetten naar mijn standaard
etf-reset-default-title = Vervang deze tijden door de standaardbeschikbaarheid uit je profiel
etf-availability-prefilled = Vooraf ingevuld vanuit je { $link }. Je kunt deze hier aanpassen voor dit afspraaktype.
etf-availability-prefilled-link = standaardbeschikbaarheid
etf-section-buffers = Buffertijden en aankondiging
etf-buffer-before = Buffertijd vooraf (min)
etf-buffer-after = Buffertijd achteraf (min)
etf-min-notice = Minimale aankondigingstijd
etf-min-notice-help = Hoe ver van tevoren iemand moet boeken.
etf-section-limits = Boekingslimieten
etf-first-slot-only = Eén tijdslot per dag
etf-first-slot-only-help = Toon alleen de vroegst beschikbare tijd per dag.

## ilia-ov4-dash
etf-freq-per-member = Per teamlid

etf-freq-limit = Boekingsfrequentie beperken
etf-freq-limit-help = Beperk hoe vaak deze afspraak per periode geboekt kan worden.
etf-add-limit = Limiet toevoegen
etf-section-options = Boekingsopties
etf-requires-confirmation = Bevestiging vereist
etf-requires-confirmation-help = Boekingen blijven in afwachting totdat je ze goedkeurt via het dashboard.
etf-sms = Sms-meldingen
etf-sms-off = Uit, geen telefoonnummer gevraagd
etf-sms-optional = Optioneel, gasten mogen een nummer opgeven
etf-sms-required = Verplicht, gasten moeten een nummer opgeven
etf-sms-help = Stuurt de gast naast een e-mail ook een sms wanneer de boeking is bevestigd, verzet, geannuleerd of op het punt staat te beginnen. Gasten die het veld leeg laten, krijgen gewoon geen sms. Vereist een sms-gateway in het { $link }.
etf-admin-panel-link = beheerpaneel
etf-additional-guests = Extra gasten
etf-guests-none = Gasten kunnen niemand anders toevoegen
etf-additional-guests-help = Sta degene die boekt toe om extra deelnemers uit te nodigen die de agenda-uitnodiging ontvangen.
etf-default-view = Standaard agendaweergave
etf-view-month = Maand — kalenderraster met lijst van tijdsloten
etf-view-week = Week — 7 kolommen met tijdsloten
etf-view-column = Kolom — dagen onder elkaar met tijdsloten
etf-view-week-short = week
etf-view-column-short = kolom
etf-default-view-help = De weergave die gasten standaard zien. Ze kunnen altijd van weergave wisselen.
etf-conflict-calendars = Agenda's voor conflicten
etf-conflict-calendars-help = Kies welke agenda's worden gecontroleerd op conflicten. Als je niets selecteert, worden alle agenda's gebruikt.
etf-no-resources = Nog geen gedeelde faciliteiten ingesteld. Voeg er een toe (demolab, vergaderruimte) in het { $link } om die hier verplicht te maken.
etf-section-access = Toegang en meldingen
etf-visibility-public = Openbaar — zichtbaar op je profiel
etf-visibility-internal = Intern — elke collega kan uitnodigingslinks maken
etf-visibility-private = Privé — alleen via uitnodigingslink
etf-visibility-help = Bepaalt wie dit afspraaktype kan zien en boeken.
etf-vis-internal = Intern
etf-reminder = Boekingsherinnering
etf-reminder-none = Geen herinnering
etf-reminder-help = Stuur jou en je gast vóór de afspraak een herinnering per e-mail.
etf-dynamic-group = Dynamische groepslink
etf-dynamic-group-help = Maak een ad-hoc afspraaklink die de beschikbaarheid van jou en andere gebruikers controleert.
etf-dynamic-group-search = Zoek een gebruiker om toe te voegen...
etf-dynamic-group-note = Alleen gebruikers die dynamische groepslinks toestaan, worden getoond.
etf-dynamic-group-url = URL van groepslink
etf-watcher-teams = Meekijkende teams
etf-watcher-teams-help = Geselecteerde teams krijgen een melding als er geboekt wordt. Leden kunnen boekingen overnemen om als deelnemer aan te sluiten.
etf-save = Wijzigingen opslaan
etf-create = Afspraaktype aanmaken
etf-js-loading = Laden...
etf-js-no-default = Geen standaard ingesteld
etf-js-reset-done = Teruggezet!
etf-js-error = Fout
etf-js-remove-limit = Limiet verwijderen
etf-period-day = Per dag
etf-period-week = Per week
etf-period-month = Per maand
etf-period-year = Per jaar

# Event type form: runtime summary hints (templates/event_type_form.html)


# %1 and %2 are substituted client-side; the values are only known once a field is edited.

etf-hint-no-days = Geen dagen ingesteld
etf-hint-every-day = Elke dag
etf-fmt-day-one = %1 dag
etf-fmt-day-other = %1 dagen
etf-fmt-hours = %1 u
etf-fmt-minutes = %1 min
etf-hint-buffer-both = %1 min vooraf, %2 min achteraf
etf-hint-buffer-before = %1 min buffertijd vooraf
etf-hint-buffer-after = %1 min buffertijd achteraf
etf-hint-notice = minimaal %1 vooraf
etf-hint-no-buffers = Geen buffertijden, altijd te boeken
etf-hint-max = Max. %1
etf-hint-period-day = /dag
etf-hint-period-week = /week
etf-hint-period-month = /maand
etf-hint-period-year = /jaar
etf-hint-no-limits = Geen limieten
etf-hint-confirmation-required = Bevestiging vereist
etf-hint-auto-confirmed = Automatisch bevestigd
etf-hint-extra-guests-one = maximaal %1 extra gast
etf-hint-extra-guests-other = maximaal %1 extra gasten
etf-hint-view = %1weergave
etf-hint-reminder = herinnering %1 van tevoren
etf-hint-no-reminder = geen herinnering

etf-guests-up-to =
    { $count ->
        [one] Maximaal { $count } extra gast
       *[other] Maximaal { $count } extra gasten
    }

etf-reminder-hours =
    { $count ->
        [one] { $count } uur van tevoren
       *[other] { $count } uur van tevoren
    }

etf-reminder-days =
    { $count ->
        [one] { $count } dag van tevoren
       *[other] { $count } dagen van tevoren
    }

# Event type form: preset banners and meeting-pattern help (templates/event_type_form.html)
# Literal braces are escaped as {"{"} because Fluent reads a bare { as a placeable.

etf-preset-public = Je maakt een <strong>openbaar</strong> afspraaktype aan &mdash; iedereen met de link kan boeken.
etf-preset-private = Je maakt een <strong>privé</strong> afspraaktype aan &mdash; alleen mensen die je uitnodigt kunnen boeken.
etf-preset-internal = Je maakt een <strong>intern</strong> afspraaktype aan &mdash; elke collega kan de boekingslink delen.
etf-preset-team = Je maakt een <strong>team</strong>-afspraaktype aan &mdash; boekingen worden over de teamleden verdeeld.
etf-pattern-hint = Optioneel afwijkend patroon. Tokens: <code>{"{"}username{"}"}</code>, <code>{"{"}event{"}"}</code>, <code>{"{"}date{"}"}</code>, <code>{"{"}random{"}"}</code>. Laat leeg om de standaard van de organisatie te gebruiken die een beheerder heeft ingesteld.
etf-pattern-random-warning = Dit patroon bevat geen <code>{"{"}random{"}"}</code>-token. Twee boekingen van dit afspraaktype op dezelfde dag delen dan dezelfde ruimte, en de tweede gast kan zo de afspraak van de eerste gast binnenlopen. Gebruik vaste ruimtes alleen als je dat echt wilt.
etf-webhook-hint = De afspraak-URL per boeking wordt opgehaald via de webhook die een beheerder heeft ingesteld onder Beheerpaneel &rarr; Vergaderwebhook. Hier is geen URL nodig.

# Admin panel (templates/admin.html)

admin-page-title = Beheer
admin-heading = Beheerdashboard
admin-action-refused = Actie geweigerd:
admin-error-last-admin = Dit is de laatste actieve beheerder. Maak eerst iemand anders beheerder of schakel een andere beheerder in.
admin-logo = Bedrijfslogo
admin-logo-help = Wordt getoond op openbare boekingspagina's. PNG, JPEG, GIF of WebP, max. 2 MB.
admin-logo-refused = Het logo is niet opgeslagen. Upload een PNG-, JPEG-, GIF- of WebP-afbeelding van maximaal 2 MB.
admin-favicon = Favicon
admin-favicon-help = Verschijnt in browsertabbladen en bladwijzers. Aanbevolen: een vierkante PNG van minstens 32×32 pixels, max. 512 KB. Zonder eigen favicon wordt de calrs-krab getoond.
admin-favicon-refused = Het favicon is niet opgeslagen. Upload een PNG-, JPEG-, GIF- of WebP-afbeelding van maximaal 512 KB.
admin-upload-save-failed = Het bestand kon niet in de gegevensmap van de server worden geschreven. De serverlog bevat de details.

# User and team avatar uploads (#307)
avatar-upload-refused = De afbeelding is niet opgeslagen. Upload een PNG-, JPEG-, GIF- of WebP-afbeelding van maximaal 2 MB.
avatar-upload-save-failed = De afbeelding kon niet op de server worden opgeslagen. Probeer het later opnieuw, of vraag je beheerder het serverlogboek te bekijken.
admin-company-link = Bedrijfslink
admin-company-link-help = Het logo linkt naar deze URL op openbare boekingspagina's. Laat leeg voor geen link.
admin-theme = Thema
admin-theme-help = Kies een kleurthema voor alle pagina's. De schakelaar voor donker/licht staat hier los van: thema's passen zich aan beide modi aan.
admin-theme-default = Standaard
admin-theme-default-desc = Strak blauw
admin-theme-nord-desc = Arctische vorst
admin-theme-dracula-desc = Donkerpaars
admin-theme-gruvbox-desc = Warm retro
admin-theme-solarized-desc = De klassieker van Ethan
admin-theme-tokyo-desc = Neonstad
admin-theme-custom = Aangepast
admin-theme-custom-desc = Jouw kleuren
admin-custom-colors = Aangepaste kleuren
admin-color-accent = Accent
admin-color-accent-hover = Accent (hover)
admin-color-bg = Achtergrond
admin-color-surface = Oppervlak
admin-color-text = Tekst
admin-save-theme = Thema opslaan
admin-users = Gebruikers ({ $count })
admin-user-filter = Filter op naam of e-mail…
admin-badge-admin = beheerder
admin-badge-disabled = uitgeschakeld
admin-impersonate = Inloggen als
admin-demote = Degraderen
admin-promote = Promoveren
admin-disable = Uitschakelen
admin-enable = Inschakelen
admin-delete = Verwijderen
admin-no-users-match = Geen gebruikers gevonden met dit filter.
admin-no-users = Nog geen gebruikers.
admin-groups = Groepen ({ $count })
admin-group-filter = Filter op groepsnaam…
admin-group-name = Groepsnaam
admin-weight = gewicht:
admin-no-groups-match = Geen groepen gevonden met dit filter.
admin-no-groups = Nog geen groepen gesynchroniseerd. Groepen worden automatisch gesynchroniseerd vanuit je OIDC-provider.
admin-auth-settings = Authenticatie-instellingen
admin-registration-enabled = Registratie ingeschakeld
admin-allowed-domains = Toegestane e-maildomeinen
admin-allowed-domains-hint = (kommagescheiden, laat leeg om alles toe te staan)
admin-save-auth = Authenticatie-instellingen opslaan
admin-system-settings = Systeeminstellingen
admin-base-url = Basis-URL
admin-base-url-help = Openbare URL van deze installatie. Wordt gebruikt voor OIDC-redirects en links in e-mails (goedkeuren/afwijzen, annuleren, herinneringen).
admin-private-hosts = Toegestane privéhosts
admin-private-hosts-help = Kommagescheiden hostnamen die naar privé- of gereserveerde IP-adressen mogen resolven voor CalDAV/EWS-bronnen (uitzondering op de SSRF-beveiliging). Voeg alleen hosts toe die je zelf beheert (bijv. een agendaserver op hetzelfde Docker-netwerk). Laat leeg om de beveiliging voor alle hosts actief te houden.
admin-german-formal = Aanspreekvorm in het Duits
admin-german-formal-du = Informeel (du)
admin-german-formal-sie = Formeel (Sie)
admin-german-formal-help = Hoe pagina's en e-mails bezoekers aanspreken van wie de browser om Duits vraagt. Wie een taal koos in Profiel en instellingen, houdt die.
admin-unset-env = Verwijder de omgevingsvariabele om dit hier te kunnen bewerken.
admin-save-system = Systeeminstellingen opslaan
admin-status = Status:
admin-status-enabled = ingeschakeld
admin-status-disabled = uitgeschakeld
admin-status-disabled-paren = (uitgeschakeld)
admin-status-configured = geconfigureerd
admin-status-not-configured = niet geconfigureerd
admin-via-environment = (via omgevingsvariabelen)
admin-issuer = Issuer:
admin-client-id = Client-ID:
admin-instance = Instantie:
admin-oidc-settings = OIDC-instellingen
admin-oidc-enabled = OIDC ingeschakeld
admin-issuer-url = Issuer-URL
admin-client-id-label = Client-ID
admin-client-secret = Client-secret
admin-keep-current-hint = (laat leeg om de huidige waarde te behouden)
admin-keep-current-set-hint = (laat leeg om de huidige waarde te behouden — momenteel ingesteld)
admin-keep-unchanged = Laat leeg om ongewijzigd te laten
admin-oidc-auto-register = Nieuwe gebruikers automatisch registreren via OIDC
admin-save-oidc = OIDC-instellingen opslaan
admin-google = Google Agenda (OAuth2)
admin-save-google = Google OAuth2-instellingen opslaan
admin-captcha = Captcha
admin-instance-url = Instantie-URL
admin-site-key = Sitesleutel
admin-secret = Secret
admin-widget-url = URL van widgetscript
admin-widget-url-help = Overschrijf dit als het CDN geblokkeerd is. Wijzigingen gaan direct na opslaan in.
admin-wasm-url = URL van widget-WASM
admin-wasm-url-help = Waar de widget zijn oplosser vandaan laadt. Vul het WASM-bestand van je Cap-server in, zodat boekingspagina's geen contact meer maken met cdn.jsdelivr.net. Laat leeg voor de standaardwaarde.
admin-captcha-wasm-url-invalid = De URL van de widget-WASM moet een absolute http(s)-URL zijn.
admin-captcha-disable-help = Laat instantie-URL, sitesleutel en secret leeg om captcha op boekingspagina's uit te schakelen.
admin-save-captcha = Captcha-instellingen opslaan
admin-resources = Faciliteiten
admin-resources-help = Gedeelde boekbare faciliteiten (demolab, vergaderruimtes) op basis van een agendafeed. Gekoppeld aan afspraaktypes blokkeert een bezette faciliteit boekingen.
admin-resource-stats = Afspraken in cache: { $events } &middot; Gekoppeld aan { $attached } afspraaktype(s)
admin-never = nooit
admin-resource-sync-failed = (laatste poging mislukt: { $error })
admin-writeback-enabled = Terugschrijven: ingeschakeld ({ $via })
admin-writeback-readonly = Terugschrijven: alleen-lezen
admin-teams-allowed = Toegestane teams:
admin-teams-allowed-none = geen (alleen globale beheerders)
admin-sync-now = Nu synchroniseren
admin-test-write = Schrijven testen
admin-delete-resource-confirm = Deze faciliteit verwijderen? Afspraaktypes die haar gebruiken, controleren haar dan niet meer.
admin-name = Naam
admin-name-help = Laat leeg om de naam uit de feed op te halen.
admin-feed-url = ICS-feed-URL (publicatieadres)
admin-feed-url-help = BlueMind: het openbare of privé-agendaadres van de faciliteitenagenda.
admin-caldav-url = CalDAV-collectie-URL (voor terugschrijven)
admin-caldav-url-help = Optioneel. Voor BlueMind wordt deze automatisch afgeleid van de feed-URL.
admin-caldav-username = CalDAV-gebruikersnaam
admin-caldav-password = CalDAV-wachtwoord
admin-resource-teams = Teams die deze faciliteit mogen gebruiken
admin-resource-teams-help = Teambeheerders van deze teams kunnen deze faciliteit koppelen aan de afspraaktypes van hun team. Leeg: alleen globale beheerders.
admin-no-teams = Nog geen teams.
admin-save-resource = Faciliteit opslaan
admin-add-resource = Faciliteit toevoegen
admin-jitsi = Jitsi (automatisch gegenereerde vergaderlinks)
admin-jitsi-help = Als de locatie van een afspraaktype is ingesteld op "Jitsi (automatisch gegenereerde ruimte)", maakt calrs per boeking een nieuwe ruimte-URL door het onderstaande patroon aan je Jitsi-basis-URL toe te voegen. Er is geen externe API-aanroep nodig.
admin-display-name = Weergavenaam
admin-jitsi-display-name-placeholder = bijv. Meet DYB
admin-jitsi-display-name-help = Wordt aan gasten getoond bij de tijdslotkiezer en het boekingsformulier. Standaard "Videogesprek" als dit leeg is.
admin-room-pattern = Patroon voor ruimtenaam
admin-jitsi-disable-help = Laat de basis-URL leeg om het automatisch genereren van Jitsi-links uit te schakelen.
admin-save-jitsi = Jitsi-instellingen opslaan
admin-meeting-webhook = Vergaderwebhook (eigen provider)
admin-webhook-url = Webhook-URL
admin-webhook-display-name-placeholder = bijv. Zoom, Whereby, Custom Meet
admin-webhook-display-name-help = Wordt aan gasten getoond in plaats van het algemene label "Videogesprek".
admin-authentication = Authenticatie
admin-auth-none = Geen
admin-auth-hmac = HMAC-SHA256 (X-Calrs-Signature-header)
admin-shared-secret = Gedeeld secret
admin-webhook-disable-help = Laat de URL leeg om de vergaderwebhook uit te schakelen.
admin-save-webhook = Webhookinstellingen opslaan
admin-smtp = SMTP-instellingen
admin-smtp-test-sent = Testmail verzonden.
admin-smtp-test-failed = De testmail kon niet worden verzonden. Controleer de serverlogs en je SMTP-instellingen.
admin-smtp-env-error = Fout in SMTP-configuratie via omgevingsvariabelen:
admin-smtp-host = Host:
admin-smtp-from = Van:
admin-smtp-enabled = SMTP ingeschakeld
admin-host = Host
admin-port = Poort
admin-tls-mode = TLS-modus
admin-tls-starttls = STARTTLS (poort 587)
admin-tls-implicit = Impliciete TLS (poort 465)
admin-tls-none = Geen, onversleuteld (alleen lokale MTA)
admin-smtp-username-hint = (laat leeg voor een relay zonder authenticatie)
admin-from-email = Afzenderadres
admin-from-name = Afzendernaam
admin-save-smtp = SMTP-instellingen opslaan
admin-send-test-email = Stuur een testmail naar
admin-send-test-email-hint = (standaard het e-mailadres van je account)
admin-send-test-email-btn = Testmail versturen
admin-smtp-clear-confirm = De SMTP-configuratie in de database verwijderen?
admin-clear-db-config = Databaseconfiguratie wissen
admin-sms = SMS-instellingen
admin-sms-help = Optioneel. Er wordt alleen een SMS verstuurd voor boekingen van afspraaktypes waarbij "SMS-meldingen" is ingeschakeld, en alleen als de gast een telefoonnummer heeft opgegeven.
admin-sms-test-sent = Testbericht verzonden.
admin-sms-test-checked = Inloggegevens geaccepteerd.
admin-sms-test-error = De SMS-gateway heeft het verzoek geweigerd.
admin-sms-captcha-warning = Het boekingsformulier is openbaar en het ontvangende nummer komt van de gast, dus SMS zonder captcha is een open relay waarvoor iemand anders jou kosten kan laten maken. Stel hierboven de captcha in en beperk de bestemmingslanden in de instellingen van je gateway zelf.
admin-sms-sent-today = Vandaag verzonden:
admin-sms-of-cap = van { $cap }
admin-sms-config-error = Fout in SMS-configuratie:
admin-sms-gateway = Gateway:
admin-sms-account = Account:
admin-sms-sender = Afzender:
admin-sms-enabled = SMS ingeschakeld
## ilia-ov3-admin
admin-sms-provider-webhook = Eigen webhook
admin-sms-webhook-secret = HMAC-geheim
admin-sms-gateway-label = Gateway
admin-required-on-switch = Verplicht bij het wisselen van gateway
## ilia-ov2-settings
admin-sms-sender-label = Afzender
admin-sms-api-base-url = API-basis-URL
admin-sms-region-base-url = Regio-basis-URL
admin-sms-webhook-url = Webhook-URL
admin-sms-twilio-sender-hint = Een Twilio-nummer in E.164-formaat (+15551234567), of een alfanumerieke afzender-ID waar het bestemmingsland dat toestaat.
admin-sms-twilio-base-url-hint = Optioneel. Standaard https://api.twilio.com. Gebruik https, de inloggegevens gaan over deze verbinding.
admin-sms-gatewayapi-sender-hint = Tot 11 alfanumerieke tekens, of 15 cijfers.
admin-sms-gatewayapi-base-url-hint = Optioneel. Standaard https://gatewayapi.com; gebruik https://gatewayapi.eu om het verkeer in de EU te houden.
admin-sms-sevenio-sender-hint = Tot 11 alfanumerieke tekens, of 16 cijfers.
admin-sms-sevenio-base-url-hint = Optioneel. Standaard https://gateway.seven.io.
admin-sms-webhook-sender-hint = Optioneel. Wordt als "sender" aan je endpoint doorgegeven.
admin-sms-webhook-base-url-hint = calrs stuurt hier een POST {"{"}"to", "text", "sender"{"}"} naartoe en verwacht een 2xx-antwoord.

admin-sms-docs = API-documentatie van { $provider }
admin-sms-country = Standaard landcode
admin-sms-country-hint = (gebruikt als gasten een lokaal telefoonnummer invoeren)
admin-sms-daily-cap = Daglimiet
admin-sms-daily-cap-hint = (berichten per dag voor de hele installatie, 0 voor geen limiet)
admin-sms-daily-cap-help = Boven de limiet stopt calrs met SMS'en en blijft het e-mails versturen, zodat boekingen nooit mislukken omdat het SMS-budget op is.
admin-save-sms = SMS-instellingen opslaan
admin-send-test-sms = Stuur een testbericht naar
admin-send-test-sms-hint-check = (laat leeg om alleen de inloggegevens te controleren)
admin-send-test-sms-hint-e164 = (E.164-notatie)
admin-test-gateway = Gateway testen
admin-sms-clear-confirm = De SMS-configuratie in de database verwijderen?
admin-sms-allow-all = Alle gebruikers SMS laten inschakelen voor hun afspraaktypes
admin-sms-allow-all-help = Standaard uit: SMS kost tegoed op het hier ingestelde account, dus alleen beheerders mogen een afspraaktype in een SMS-modus zetten.
admin-save-policy = Beleid opslaan
admin-page-of = Pagina %1 van %2
admin-show-more-js = Nog %1 tonen
admin-show-fewer = Minder tonen

# Admin panel: strings carrying markup or literal braces (templates/admin.html)

admin-delete-user-confirm = Gebruiker { $email } definitief verwijderen?{"\u000A"}{"\u000A"}Hiermee worden het gebruikersrecord, het planningsaccount, de agendabronnen, de afspraaktypes en alle gegevens die alleen van deze gebruiker zijn verwijderd. Eerdere boekingen worden samen met hun afspraaktypes verwijderd.{"\u000A"}{"\u000A"}Voor OIDC/SSO-gebruikers: als automatisch registreren is ingeschakeld, wordt deze persoon bij de volgende keer inloggen opnieuw aangemaakt.{"\u000A"}{"\u000A"}Dit kan niet ongedaan worden gemaakt.
admin-system-settings-help = Openbare URL en netwerkbeveiligingsinstellingen. Deze kun je ook instellen met de omgevingsvariabelen <code>CALRS_BASE_URL</code> en <code>CALRS_ALLOW_PRIVATE_HOSTS</code>. Als een omgevingsvariabele is ingesteld, <strong>heeft die voorrang</strong> op de waarde hieronder.
admin-set-by-env = — ingesteld via omgevingsvariabele ({ $var }), overschrijft de opgeslagen waarde
admin-google-help = Om de koppeling met Google Agenda in te schakelen, maak je OAuth2-inloggegevens aan in de <a href="https://console.cloud.google.com/apis/credentials" target="_blank" style="color: var(--accent);">Google Cloud Console</a>. Schakel de <strong>CalDAV API</strong> (bezette tijden lezen en boekingen terugschrijven) en de <strong>Google Calendar API</strong> (Google Meet-links) in en voeg daarna { $redirect_uri } toe als geautoriseerde redirect-URI.
admin-room-pattern-help = Beschikbare tokens: <code>{"{"}username{"}"}</code> (organisator), <code>{"{"}event{"}"}</code> (slug van het afspraaktype), <code>{"{"}date{"}"}</code> (JJJJMMDD), <code>{"{"}random{"}"}</code> (8 tekens). Standaard: { $default }.
admin-room-pattern-warning = Zonder <code>{"{"}random{"}"}</code> is de ruimtenaam voorspelbaar: twee gasten die op dezelfde dag hetzelfde afspraaktype boeken, delen dan een ruimte en kunnen elkaars vergadering zien. Vaste ruimtes zijn toegestaan (bijv. één persoonlijke ruimte per organisator), maar schakel dit alleen in als je de afweging begrijpt.
admin-meeting-webhook-help = Als de locatie van een afspraaktype is ingesteld op "Webhook (eigen provider)", stuurt calrs bij bevestiging de boekingsgegevens via een POST naar deze URL en verwacht een JSON-body <code>{"{"}"url": "https://..."{"}"}</code> terug.
admin-auth-hmac-help = Met HMAC stuurt calrs <code>X-Calrs-Signature: sha256=&lt;hex&gt;</code> mee over de ruwe request-body.
admin-tls-none-warning = Kies <strong>Geen</strong> alleen voor een relay op deze machine die geen STARTTLS biedt of een zelfondertekend certificaat heeft. E-mail en eventuele inloggegevens gaan dan onversleuteld over het netwerk.
admin-smtp-env-error-help = Corrigeer de omgevingsvariabelen <code>CALRS_SMTP_*</code>, of verwijder ze om SMTP hier via de database te beheren.
admin-smtp-env-managed = Beheerd via <strong>omgevingsvariabelen</strong> (overschrijven de database). Pas de variabelen <code>CALRS_SMTP_*</code> aan om dit te wijzigen, of verwijder ze om SMTP hier te beheren.
admin-smtp-env-help = Je kunt dit ook instellen via omgevingsvariabelen (die dit overschrijven): <code>CALRS_SMTP_HOST</code>, <code>CALRS_SMTP_PORT</code>, <code>CALRS_SMTP_TLS_MODE</code> (<code>starttls</code>, <code>tls</code> of <code>none</code>), <code>CALRS_SMTP_USERNAME</code>, <code>CALRS_SMTP_PASSWORD</code>, <code>CALRS_SMTP_FROM_EMAIL</code>, <code>CALRS_SMTP_FROM_NAME</code>. Alleen <code>CALRS_SMTP_HOST</code> en <code>CALRS_SMTP_FROM_EMAIL</code> zijn verplicht; laat gebruikersnaam en wachtwoord weg om zonder authenticatie via een lokale MTA te versturen.
admin-sms-env-error-help = Corrigeer de omgevingsvariabelen <code>CALRS_SMS_*</code>, of verwijder ze om SMS hier via de database te beheren.
admin-sms-env-managed = Beheerd via <strong>omgevingsvariabelen</strong> (overschrijven de database). Pas de variabelen <code>CALRS_SMS_*</code> aan om dit te wijzigen, of verwijder ze om SMS hier te beheren.
admin-sms-env-help = Je kunt dit ook instellen via omgevingsvariabelen (die dit overschrijven): <code>CALRS_SMS_PROVIDER</code>, <code>CALRS_SMS_API_KEY</code>, <code>CALRS_SMS_API_SECRET</code>, <code>CALRS_SMS_SENDER</code>, <code>CALRS_SMS_BASE_URL</code>, <code>CALRS_SMS_DAILY_CAP</code>, <code>CALRS_SMS_DEFAULT_COUNTRY_CODE</code>.
admin-sms-trial-warning = <strong>De proefmodus van Twilio staat aan</strong> (<code>CALRS_SMS_TWILIO_TRIAL</code>). Gasten ontvangen het vooraf gedefinieerde <code>sms_appointment_reminders</code>-sjabloon van Twilio, niet het echte bericht, en alleen nummers die in je Twilio-console zijn geverifieerd worden bereikt. Dit is een testhulpmiddel voor proefaccounts. Verwijder de variabele voordat je boekingen aanneemt.

admin-show-more =
    { $count ->
        [one] Nog { $count } tonen
       *[other] Nog { $count } tonen
    }

# Calendar source form: backend picker (templates/source_form.html)

source-form-backend-help = Kies het protocol dat je server gebruikt. EWS is bedoeld voor on-premises Exchange 2019/2016/2013.

admin-sms-going-live = <strong>Voordat je live gaat:</strong> beperk de bestemmingslanden in je gateway (Twilio noemt dit Geo Permissions), houd het account prepaid zonder automatisch opwaarderen en laat de captcha aan. Samen beperken die drie wat een poging tot SMS pumping je kan kosten.

troubleshoot-heading = Problemen met beschikbaarheid oplossen

# Host-side form validation errors (src/web/mod.rs)

form-error-team-name-slug-required = Naam en slug zijn verplicht.
form-error-team-name-length = De naam mag maximaal 255 tekens lang zijn.
form-error-team-description-length = De beschrijving mag maximaal 5000 tekens lang zijn.
form-error-slug-charset = De slug mag alleen kleine letters, cijfers en streepjes bevatten.
form-error-slug-reserved = Deze slug is gereserveerd. Kies een andere.
form-error-team-slug-taken = Er bestaat al een team met deze slug.
form-error-title-required = Een titel is verplicht om een slug te genereren.
form-error-event-type-slug-taken = Er bestaat al een afspraaktype met deze slug.
form-error-event-type-slug-taken-team = Er bestaat in dit team al een afspraaktype met deze slug.
form-error-event-type-title-required = Een titel is verplicht.
form-error-event-type-title-length = De titel mag hoogstens 255 tekens lang zijn.
form-error-event-type-duration = De duur moet een geheel aantal minuten zijn, van 1 tot 1440 (24 uur).
form-error-event-type-number = Buffertijden, minimale aankondigingstijd, herinnering en extra gasten moeten gehele getallen zijn, 0 of meer.
form-error-event-type-horizon = De boekingshorizon moet een geheel aantal dagen zijn, 0 of meer. Laat het leeg voor geen limiet.
form-error-location-required = Locatiegegevens zijn verplicht (bijv. een link voor een videogesprek, een telefoonnummer of een adres).
form-error-bookable-range-order = De einddatum moet op of na de begindatum liggen.
form-error-bookable-range-invalid = Vul datums in als JJJJ-MM-DD.
form-error-not-team-admin = Je bent geen teambeheerder van dit team.
form-error-no-account = Geen planningsaccount gevonden. Neem contact op met een beheerder.
form-error-all-fields-required = Alle velden zijn verplicht.
form-error-encryption = Versleutelingsfout.
form-error-connection-failed = Verbinding mislukt: { $error }. Controleer de URL en inloggegevens, of vink "Verbindingstest overslaan" aan om toch op te slaan.

# Settings page flash (src/web/mod.rs)

settings-saved = Instellingen opgeslagen.

# Profile settings validation and flash messages (src/web/mod.rs)

settings-error-name-length = De naam moet tussen 1 en 255 tekens lang zijn.
settings-error-username-length = De gebruikersnaam moet minstens 2 tekens lang zijn.
settings-error-username-taken = Deze gebruikersnaam is al in gebruik.
settings-error-booking-email = Voer een geldig e-mailadres voor boekingen in.
settings-error-save-failed = Instellingen opslaan mislukt.

# Host-facing error responses (src/web/mod.rs)

error-team-not-found-or-not-admin = Team niet gevonden of je bent geen teambeheerder.
error-team-not-found = Team niet gevonden.
error-event-type-not-found = Afspraaktype niet gevonden.
error-decrypt-failed = Opgeslagen inloggegevens konden niet worden ontsleuteld.
error-source-not-found = Bron niet gevonden.
error-source-no-password = Voor deze bron is geen wachtwoord opgeslagen.
error-oauth-invalid-state = Ongeldige state-parameter. Probeer het opnieuw.
error-oauth-no-code = Geen autorisatiecode ontvangen.
error-oauth-not-configured = Google OAuth2 is niet geconfigureerd.
error-no-scheduling-account = Geen planningsaccount gevonden.
error-private-event-type-not-found = Privé-afspraaktype niet gevonden.
error-access-denied = Toegang geweigerd.

# Guest booking-flow errors (src/web/mod.rs)

error-slot-unavailable = Dit tijdslot is niet meer beschikbaar.
error-slot-too-soon = Dit tijdslot is niet meer beschikbaar (te kort van tevoren).
error-slot-beyond-horizon = Dit tijdslot valt buiten de boekingsperiode.
error-slot-not-yet-bookable = Dit tijdslot is nog niet te boeken.
error-invite-required = Voor dit afspraaktype is een uitnodigingslink nodig.
error-invite-invalid = Ongeldige uitnodigingslink.
error-invite-expired = Deze uitnodigingslink is verlopen.
error-invite-used = Deze uitnodigingslink is al gebruikt.
error-invalid-date = Ongeldige datum.
error-invalid-time = Ongeldige tijd.
error-invalid-date-format = Ongeldige datumnotatie.
error-invalid-time-format = Ongeldige tijdnotatie.
error-too-many-bookings = Te veel boekingspogingen. Probeer het over een paar minuten opnieuw.
error-too-many-requests = Te veel verzoeken. Probeer het later opnieuw.
error-no-members-available = Er zijn geen teamleden beschikbaar voor dit tijdslot.
error-dynamic-group-public-only = Dynamische groepslinks zijn alleen beschikbaar voor openbare afspraaktypes.
error-user-not-found = Gebruiker niet gevonden.

# Booking action error page: titles (templates/booking_action_error.html)

bae-title-captcha = Captcha-verificatie mislukt
bae-title-invalid-booking = Ongeldige boekingsgegevens
bae-title-unavailable = Nu niet beschikbaar
bae-title-cannot-approve = Deze boeking kan niet worden goedgekeurd
bae-title-invalid-link = Ongeldige link
bae-title-invalid-or-expired = Ongeldige of verlopen link
bae-title-booking-not-found = Boeking niet gevonden
bae-title-already-approved = Al goedgekeurd
bae-title-already-declined = Al afgewezen
bae-title-already-cancelled = Al geannuleerd
bae-title-booking-cancelled = Boeking geannuleerd
bae-title-booking-declined = Boeking afgewezen

# Booking action error page: bodies

bae-body-go-back = Ga terug en probeer het opnieuw.
bae-body-unavailable = De organisator neemt voor deze datum geen boekingen meer aan. Kies een andere datum of kijk later nog eens.
bae-body-resource-gone = Een benodigde faciliteit is op dit tijdstip niet meer beschikbaar. Vraag de gast een ander tijdslot te kiezen.
bae-body-no-claim-token = Geen token voor overnemen opgegeven.
bae-body-claim-invalid = Deze link om over te nemen is niet meer geldig.
bae-body-booking-gone = Deze boeking bestaat niet meer.
bae-body-decline-link-invalid = Deze afwijslink is ongeldig of verlopen, of de boeking is al verwerkt.
bae-body-cancel-link-invalid = Deze annuleringslink is ongeldig of verlopen, of de boeking is al geannuleerd.
bae-body-cancel-link-invalid-short = Deze annuleringslink is ongeldig of verlopen.
bae-body-reschedule-link-invalid = Deze link om te verzetten is ongeldig of verlopen, of de boeking is al verwerkt.
bae-body-approval-link-invalid = Deze goedkeuringslink is ongeldig of verlopen.
bae-body-already-approved = Deze boeking is al goedgekeurd.
bae-body-already-declined = Deze boeking is al afgewezen.
bae-body-already-cancelled = Deze boeking is al geannuleerd.
bae-body-was-cancelled = Deze boeking is geannuleerd.
bae-body-declined-by-host = Deze boeking is afgewezen door de organisator.

# Booking form validation (src/web/mod.rs)

validate-name-length = De naam moet tussen 1 en 255 tekens lang zijn.
validate-email-length = Het e-mailadres moet tussen 1 en 255 tekens lang zijn.
validate-email-invalid = Voer een geldig e-mailadres in.
validate-notes-length = Opmerkingen mogen maximaal 5000 tekens lang zijn.
validate-date-too-far = Je kunt niet meer dan een jaar vooruit boeken.

# Additional guests and dynamic group links (src/web/mod.rs)

guests-not-allowed = Extra gasten zijn niet toegestaan voor dit afspraaktype.
guests-too-many =
    { $max ->
        [one] Je kunt maximaal één extra gast toevoegen.
       *[other] Je kunt maximaal { $max } extra gasten toevoegen.
    }
guests-invalid-email = Ongeldig e-mailadres van extra gast: { $email }
dynamic-group-min-usernames = Voor dynamische groepslinks zijn minstens twee gebruikersnamen nodig.
dynamic-group-user-not-found = Gebruiker "{ $username }" niet gevonden.
dynamic-group-user-opted-out = Gebruiker "{ $username }" heeft dynamische groepslinks niet ingeschakeld.

error-slot-unavailable-member = Dit tijdslot is niet meer beschikbaar ({ $username } heeft een conflict).

# Pending requests: slot hold, expiry and approval re-check (#234)

sms-expired = Aanvraag verlopen: { $event }, { $date } om { $time } ({ $tz }). Geen reactie, kies een ander tijdstip.
email-pending-expired-status = Aanvraag verlopen
email-pending-expired-headline = Je boekingsaanvraag is verlopen: de organisator heeft niet op tijd gereageerd. Kies een ander tijdstip.
email-pending-expired-action = Kies een ander tijdstip
bae-body-host-busy = De organisator (of een vereist teamlid) is op dit tijdstip niet meer beschikbaar. Vraag de gast een ander tijdslot te kiezen.
bookings-error-generic = Er is iets misgegaan.
bookings-error-no-bookable-dates = Dit afspraaktype heeft geen boekbare dagen meer, dus de gast kan geen nieuwe tijd kiezen. Verleng eerst de boekingsperiode.
troubleshoot-detail-pending-request = { $title } (openstaande aanvraag)
etf-pending-blocks-slot = Openstaande aanvragen blokkeren het tijdslot
etf-pending-blocks-slot-help = Zolang een aanvraag op je antwoord wacht, kan niemand anders dat tijdstip boeken, bij dit en bij je andere afspraaktypes.
etf-pending-expiry = Verloop van aanvragen
etf-pending-expiry-help = Een aanvraag die je dan nog niet hebt beantwoord, wordt automatisch afgewezen en de gast wordt gevraagd een ander tijdstip te kiezen. De termijn eindigt eerder als de afspraak te dichtbij komt om het tijdslot nog door iemand anders te laten boeken. Halverwege krijg je één herinnering.
etf-pending-expiry-never = Nooit

etf-pending-expiry-hours =
    { $hours ->
        [one] Na { $hours } uur
       *[other] Na { $hours } uur
    }

# Host, watcher and remaining guest emails (src/email.rs, #209)

email-footer-sent-by = Verzonden door
email-sender-via = { $host } via { $instance }
email-date-at-time = { $date } om { $time }
email-detail-phone = Telefoon:
email-detail-expires = Verloopt:
email-detail-new-date = Nieuwe datum:
email-detail-new-time = Nieuwe tijd:
email-detail-previous = Eerder:
email-detail-requested = Gevraagd:
email-detail-originally = Oorspronkelijk:
email-detail-organizer = Organisator:
email-detail-booked-by = Geboekt door:
email-detail-invited-by = Uitgenodigd door:
email-detail-message = Bericht:
email-action-approve = Goedkeuren
email-action-decline = Afwijzen
email-action-pick-new-time = Kies een nieuw tijdstip
email-host-approve-line = Goedkeuren: { $url }
email-host-decline-line = Afwijzen: { $url }
email-host-approval-dashboard = Log in op je dashboard om deze boeking te bevestigen of af te wijzen.
email-host-new-subject = Nieuwe boeking: { $event }, { $guest } ({ $date })
email-host-new-heading = Nieuwe boeking!
email-host-new-message = { $guest } heeft een afspraak met je geboekt.
email-host-confirmed-subject = Bevestigd: { $event }, { $guest } ({ $date })
email-host-confirmed-heading = Boeking bevestigd
email-host-confirmed-message = Je hebt de boeking met { $guest } goedgekeurd.
email-host-confirmed-footer = Het evenement is aan je agenda toegevoegd.
email-host-sync-failure-subject = Actie nodig: agenda niet bijgewerkt voor { $event } ({ $date })
email-host-sync-failure-heading = Je agenda is niet bijgewerkt
email-host-sync-failure-intro = Deze boeking is verzet, maar calrs kon het evenement in je Google Agenda niet bijwerken. De gast heeft de nieuwe tijd; je agenda toont nog de oude.
email-host-sync-failure-action = Open het evenement in Google Agenda en verplaats het naar de tijd hierboven. Verplaats het bestaande evenement in plaats van het opnieuw aan te maken, anders werkt de Google Meet-link die de gast al heeft niet meer.
email-writeback-failure-subject = Actie vereist: { $event } ({ $date }) staat niet in je agenda
email-writeback-failure-heading = Er ontbreekt een boeking in je agenda
email-writeback-failure-intro = calrs kon deze boeking ook na meerdere pogingen niet aan je agenda toevoegen. De boeking is bevestigd en de gast heeft de uitnodiging.
email-writeback-failure-action = Zet het evenement zelf in je agenda zodat je het niet mist, en controleer de agendabron in calrs. calrs blijft het nog een paar uur proberen; je krijgt deze e-mail voor deze boeking niet nog een keer.
email-detail-error = Fout:
email-source-failure-subject = Actie vereist: agendabron "{ $source }" synchroniseert niet
email-source-failure-heading = Een agendabron synchroniseert niet meer
email-source-failure-intro = calrs kan deze agendabron al meer dan een uur niet synchroniseren. Je boekingspagina's bieden nog steeds tijden aan, maar die zijn gebaseerd op de laatste geslaagde synchronisatie, dus evenementen die sindsdien zijn toegevoegd of verplaatst worden niet meegenomen.
email-source-failure-action = Controleer de bron op de pagina Agendabronnen: meestal is een verlopen wachtwoord of ingetrokken toegang de oorzaak. Je hoort pas weer over deze bron als die herstelt en daarna opnieuw faalt.
email-detail-source = Agendabron:
email-detail-failing-since = Faalt sinds:
email-host-reminder-subject = Herinnering: { $event }, { $guest } ({ $date })
email-host-reminder-heading = Komende boeking
email-host-reminder-message = Herinnering: je hebt binnenkort een boeking met { $guest }.
email-host-cancel-subject = Geannuleerd: { $event }, { $guest } ({ $date })
email-host-cancel-message-by-host = Je hebt deze boeking geannuleerd.
email-host-cancel-message-by-guest = { $guest } heeft de boeking geannuleerd.
email-host-approval-subject = Actie vereist: { $event }, { $guest } ({ $date })
email-host-approval-heading = Actie vereist
email-host-approval-message = { $guest } wil een afspraak met je boeken.
email-host-approval-reminder-subject = Herinnering: { $guest } wacht op je antwoord ({ $event }, { $date })
email-host-approval-reminder-heading = Herinnering: actie vereist
email-host-approval-reminder-message = { $guest } wacht nog op je antwoord. Als je niet reageert, wordt de aanvraag op { $deadline } automatisch afgewezen.
email-host-approval-footer = Je kunt dit ook vanuit je dashboard beheren.
email-host-expired-subject = Verlopen: { $event } met { $guest } ({ $date })
email-host-expired-heading = Boekingsaanvraag verlopen
email-host-expired-message = Niemand heeft de aanvraag van { $guest } voor de deadline beantwoord. Daarom is ze automatisch afgewezen en is de gast gevraagd een ander tijdstip te kiezen.
email-host-expired-footer = Hoe lang aanvragen wachten, wijzig je in de instellingen van het afspraaktype.
email-host-reschedule-request-subject = Verzetverzoek: { $event }, { $guest } ({ $email })
email-host-reschedule-request-message = { $guest } wil de boeking verzetten.
email-watcher-claim-subject = Over te nemen: { $event }, { $guest } ({ $date })
email-watcher-claim-message = Er is een nieuwe boeking die je kunt overnemen. Klik hieronder om als deelnemer aan te sluiten.
email-watcher-claim-link = Deze boeking overnemen: { $url }
email-watcher-claim-footer = Je kunt hem ook vanuit je dashboard overnemen.
email-claim-confirmed-subject = Boeking overgenomen: { $event }, { $guest } ({ $date })
email-claim-confirmed-footer = Je wordt als deelnemer aan deze afspraak toegevoegd.
email-attendee-greeting = Hoi,
email-attendee-message = Je bent als deelnemer aan een boeking toegevoegd.
email-attendee-rescheduled-message = Een boeking waaraan je deelneemt is verzet naar een nieuw tijdstip.
email-attendee-cancelled-message = Een boeking waaraan je deelneemt is geannuleerd.
email-attendee-cancelled-subject = Geannuleerd: { $event }
email-original-booking-cancelled = Je oorspronkelijke boeking is ook geannuleerd.
email-pick-new-time-subject = Verzetten: { $event }, kies een nieuw tijdstip
email-pick-new-time-message = { $host } moet je boeking verzetten. Kies een nieuw tijdstip.
email-pick-new-time-link = Kies een nieuw tijdstip: { $url }
email-rescheduled-message = Je boeking is verzet door { $host }.
email-reschedule-approved-message = { $host } heeft het nieuwe tijdstip van je boeking bevestigd.
email-rescheduled-ics-attached-plain = Er is een bijgewerkte agenda-uitnodiging bijgevoegd.
email-rescheduled-ics-attached-html = Er is een bijgewerkte agenda-uitnodiging bij deze e-mail gevoegd.
email-need-to-reschedule = Wil je verzetten? { $url }
email-invite-subject = { $host } heeft je uitgenodigd om te boeken: { $event }
email-invite-message = { $host } heeft je uitgenodigd om te boeken: { $event }
email-invite-link = Klik op de link hieronder om een tijdstip te kiezen:
email-invite-action = Kies een tijdstip
email-cancel-reason-calendar-deleted = De organisator heeft het agenda-evenement verwijderd.

# Embed snippet generator (templates/embed.html)
embed-page-title = Insluiten — { $title }
embed-heading = Insluiten: { $title }
embed-subtitle = Genereer een codefragment om dit afspraaktype op je eigen website in te sluiten.
embed-warn-internal = Let op: dit afspraaktype is <strong>intern</strong>. Bezoekers van de ingesloten pagina hebben een geldige uitnodigingslink nodig om te boeken — het gegenereerde fragment bevat die niet. Overweeg de zichtbaarheid op openbaar te zetten om direct in te sluiten.
embed-warn-private = Let op: dit afspraaktype is <strong>privé</strong>. Bezoekers van de ingesloten pagina hebben een geldige uitnodigingslink nodig om te boeken — het gegenereerde fragment bevat die niet. Overweeg de zichtbaarheid op openbaar te zetten om direct in te sluiten.
embed-warn-private-team = Let op: dit team is <strong>privé</strong>. Bezoekers van de ingesloten pagina kunnen niet boeken zonder de uitnodigingslink van het team — het gegenereerde fragment bevat die niet. Overweeg het team openbaar te maken om direct in te sluiten.
embed-need-username = Je moet een <strong>gebruikersnaam</strong> instellen voordat je dit afspraaktype kunt insluiten — de insluitlink is { $path }. { $link } en kom daarna terug.
embed-need-username-link = Stel een gebruikersnaam in bij de instellingen
embed-username-placeholder = gebruikersnaam
embed-tab-inline = Inline
embed-tab-floating = Zwevende knop
embed-tab-element = Klik op element
embed-layout = Indeling
embed-theme = Thema
embed-theme-auto = Automatisch (volgt bezoeker)
embed-brand-color = Merkkleur
embed-use-org-color = Kleur van de organisatie gebruiken
embed-button-text = Knoptekst
embed-button-default = Afspraak maken
embed-button-position = Knoppositie
embed-pos-bottom-right = Rechtsonder
embed-pos-bottom-left = Linksonder
embed-pos-top-right = Rechtsboven
embed-pos-top-left = Linksboven
embed-button-colors = Knopkleuren
embed-button-fg-placeholder = tekst
embed-button-colors-hint = Achtergrond / tekst
embed-show-icon = Agenda-icoon tonen
embed-element-help = Plaats het fragment één keer in je site en voeg daarna de getoonde data-attributen toe aan elk klikbaar element (knop, link, div) dat het boekingsvenster moet openen.
embed-snippet = Codefragment
embed-loading = Laden…
embed-preview = Live voorbeeld
embed-preview-back = terug naar voorbeeld
embed-preview-floating-hint = Klik op de knop om het boekingsvenster te simuleren
embed-preview-element-hint = Demo-triggers — voeg op je site de data-attributen uit het fragment toe aan een willekeurig element. Klik op een van beide om het venster te proberen:
embed-preview-text-link = of klik op deze tekstlink
embed-snippet-attr-comment = Voeg deze attributen toe aan een klikbaar element op je pagina:

# Leftovers in partly translated pages (host reschedule, impersonation banner, slots, team settings)
host-resched-when = { $date } om { $start } – { $end }
base-impersonating = Je handelt als { $name }
slots-rescheduling-banner = Verzetten: { $title } (nu { $date } om { $time })
slots-invite-required-title = Voor boeken is een uitnodigingslink nodig
team-settings-slug-url = Gebruikt in de team-URL: { $url }
team-event-type-no-teams = Er zijn nog geen teams.
team-event-type-not-team-admin = Je bent van geen enkel team beheerder.

# Google Calendar connection errors (src/web/sources.rs)
google-error-not-configured = De Google Agenda-koppeling is niet ingesteld. Vraag je beheerder om de Google OAuth2-gegevens in te stellen in het beheerpaneel.
google-error-no-base-url = De openbare basis-URL is niet ingesteld. Stel die in via de omgevingsvariabele CALRS_BASE_URL of in het beheerpaneel onder Systeeminstellingen voordat je OAuth2-stromen gebruikt.
google-error-authorization = Google-autorisatie mislukt: { $error }
google-error-token-exchange = Tokenuitwisseling mislukt: { $error }
google-error-account-email = Kan het e-mailadres van het Google-account niet ophalen: { $error }

# Admin panel notices (src/web/admin.rs)
admin-resource-feed-invalid = Ongeldige feed-URL: { $error }
admin-resource-feed-check-failed = Feedcontrole mislukt: { $error }
admin-resource-caldav-invalid = Ongeldige CalDAV-URL: { $error }
admin-resource-credentials-failed = Kan de inloggegevens niet opslaan: { $error }
admin-resource-added = Resource "{ $name }" toegevoegd. Afspraken in cache: { $count }.
admin-resource-updated = Resource bijgewerkt. Afspraken in cache: { $count }.
admin-resource-deleted = Resource verwijderd.
admin-resource-not-found = Resource niet gevonden.
admin-resource-synced = Feed gesynchroniseerd. Afspraken in cache: { $count }.
admin-resource-sync-error = Synchronisatie mislukt: { $error }
admin-resource-no-caldav-url = Er is geen CalDAV-URL ingesteld en er kon er ook geen worden afgeleid uit de feed-URL.
admin-resource-no-write-credentials = Geen schrijfgegevens beschikbaar (geen serviceaccount, en geen lid dat toestemming gaf heeft een passende agendabron).
admin-resource-write-passed = Schrijftest geslaagd met { $user } (aangemaakt: { $created }, opgeruimd: { $cleaned }).
admin-resource-write-verified = geverifieerd
admin-resource-write-unverified = niet geverifieerd
admin-resource-write-check-calendar = controleer de resource-agenda
admin-resource-write-failed = Schrijftest mislukt. Laatste fout: { $error }
admin-base-url-invalid = De basis-URL moet een absolute http(s)-URL zijn, bijv. https://cal.example.com
admin-company-link-invalid = De bedrijfslink moet beginnen met http:// of https://
admin-smtp-host-required = De SMTP-host is verplicht.
admin-smtp-from-invalid = Voer een geldig afzenderadres in.
admin-smtp-port-invalid = De SMTP-poort moet een getal tussen 1 en 65535 zijn.
admin-smtp-tls-invalid = De TLS-modus moet "starttls", "tls" of "none" zijn.
admin-smtp-password-required = Bij de eerste SMTP-configuratie is een wachtwoord verplicht.
admin-sms-switch-needs-secret = Om van sms-gateway te wisselen moet je de inloggegevens van die gateway invoeren.
admin-sms-settings-invalid = Sms-instellingen: { $error }.
admin-sms-sender-format = Een numerieke sms-afzender moet in internationaal formaat zijn, bijv. +31612345678.
admin-sms-not-configured = Sms is niet ingesteld.
admin-sms-load-failed = Kan de sms-configuratie niet laden.
admin-sms-check-accepted = Inloggegevens geaccepteerd door de gateway.
admin-sms-bad-number = Dat is geen bruikbaar telefoonnummer.
admin-sms-test-body = Testbericht van calrs. Je sms-gateway werkt.
admin-sms-test-accepted = Testbericht geaccepteerd door de gateway.

## Local multi-factor authentication

mfa-title = Tweestapsverificatie
mfa-invalid = Verificatie mislukt. Controleer je wachtwoord of code. Elke code werkt maar één keer; wacht zo nodig op een nieuwe code. Wacht na te veel pogingen 15 minuten. Is het instellen verlopen, begin dan opnieuw.
mfa-restart = Terug naar inloggen
mfa-manage = Tweestapsverificatie beheren
mfa-recovery-help = Bewaar deze herstelcodes op een veilige plek. Ze worden maar één keer getoond. Elke code vervangt één keer een code uit je authenticator-app. Nieuwe herstelcodes vervangen alle eerdere codes.
mfa-continue = Verder naar het dashboard
mfa-enabled = Tweestapsverificatie is ingeschakeld.
mfa-disabled = Tweestapsverificatie is uitgeschakeld.
mfa-required-help = Je beheerder vereist tweestapsverificatie voor lokale accounts. Je kunt het niet uitschakelen.
mfa-code = Authenticator-code of herstelcode
mfa-regenerate = Nieuwe herstelcodes maken
mfa-disable = Tweestapsverificatie uitschakelen
mfa-enable = Tweestapsverificatie instellen
mfa-changes-help = Voor wijzigingen is je wachtwoord nodig. Voor uitschakelen of het vervangen van herstelcodes is ook een code nodig, en je andere sessies worden dan afgemeld.
mfa-back = Terug naar instellingen
mfa-setup-help = Scan deze QR-code met je authenticator-app, of voer de sleutel hieronder handmatig in. Voer een code van zes cijfers in om het instellen af te ronden. Het instellen verloopt na 10 minuten.
mfa-qr-alt = QR-code voor het instellen van de authenticator-app
mfa-login-help = Voer een code uit je authenticator-app in, of gebruik een van je bewaarde herstelcodes. Deze aanmelding verloopt na 10 minuten.
mfa-totp-code = Authenticator-code van zes cijfers
mfa-verify = Verifiëren
mfa-policy-help = Dit beleid geldt voor lokale accounts. SSO gebruikt de MFA van de identiteitsprovider. Lokale beheerders moeten MFA instellen voordat ze dit beleid wijzigen. MFA verplicht maken meldt lokale sessies zonder MFA-controle af en vereist het instellen bij de volgende aanmelding.
mfa-policy = MFA-beleid voor lokale accounts
mfa-optional = Optioneel
mfa-required = Verplicht
mfa-save-policy = MFA-beleid opslaan

# First day of the week (#330)
week-start-label = Eerste dag van de week
week-start-inherit-site = Standaard van de site gebruiken
week-start-inherit-owner = Standaard van de eigenaar gebruiken

## Booking directory and personal favorites

directory-title = Afsprakengids
directory-help = Vind de afspraaklink van een collega of team. Voeg afspraken die je vaak deelt toe aan je favorieten om ze op je dashboard te zien.
directory-search = Zoek op persoon, team of afspraaktitel
directory-find = Zoeken
directory-favorites-only = Alleen favorieten
directory-favorites = Favoriete afspraken
directory-browse = Gids bekijken
directory-generate = Uitnodigingslink maken
directory-invite-help = Eenmalige uitnodiging, 7 dagen geldig.
directory-add = { $title } toevoegen aan favorieten
directory-remove = { $title } verwijderen uit favorieten
directory-added = Toegevoegd aan favorieten.
directory-removed = Verwijderd uit favorieten.
directory-error = Deze actie kon niet worden voltooid. Vernieuw de pagina en probeer het opnieuw.
directory-manual-copy = Kopieer deze link handmatig:
directory-empty = Geen afspraken gevonden. Probeer een andere zoekopdracht.
directory-favorites-empty = Geen favorieten gevonden. Voeg afspraken uit de gids toe aan je favorieten om ze hier te zien.
directory-with-hosts = Met { $names }

## ilia fork: TON payments (docs/ton.md)

# Price on the slot and booking pages
ton-slots-price = Prijs: $ { $usd }
ton-slots-price-approx = (≈ { $ton } TON)
ton-slots-price-note = betalen in TON na het kiezen van een tijd

# Event type form
ton-etf-section = Betaling in TON
ton-etf-off-note = TON-betalingen zijn niet ingesteld op deze server. Een betaald afspraaktype kan pas geboekt worden als ze zijn ingesteld.
ton-etf-enabled = Betaalde boeking
ton-etf-enabled-help = Gasten betalen in TON direct na het kiezen van een tijd. De tijd blijft voor hen vastgehouden tijdens het betalen, en een onbetaalde boeking wordt automatisch vrijgegeven.
ton-etf-price = Prijs
ton-etf-price-help = In USD: van $ 1 tot $ 10000; het bedrag in TON wordt berekend met de wisselkoers op het moment van boeken.
ton-etf-window = Betaaltermijn (minuten)
ton-etf-window-help = Van 10 tot 240. Laat leeg voor de standaardwaarde van de server ({ $minutes } min).
ton-form-error-price = Vul een prijs in Amerikaanse dollars in van 1 tot 10000, met hoogstens twee decimalen.
ton-form-error-window = De betaaltermijn moet tussen 10 en 240 minuten liggen.
ton-form-error-team = Team-afspraaktypes kunnen niet betaald zijn.

# Booking error pages
ton-bae-title-unavailable = Deze tijd kan niet geboekt worden
ton-bae-body-unavailable = Betalen is tijdelijk niet mogelijk, dus deze tijd kan nu niet geboekt worden. Probeer het over een paar minuten opnieuw.
ton-bae-body-too-late = Er is te weinig tijd over tot deze afspraak om ervoor te betalen. Kies een latere tijd.
ton-bae-body-already-open = Je hebt met dit e-mailadres al een onbetaalde boeking. We kunnen je de links ervan mailen: daar kun je haar betalen of annuleren om de tijd vrij te geven, en daarna opnieuw boeken.
ton-bae-body-personal-only = Deze afspraak kan alleen op de eigen pagina van de host geboekt worden.
ton-bae-title-unpaid = Boeking niet betaald
ton-bae-body-unpaid = Deze boeking is niet betaald en kan dus niet goedgekeurd worden. Gebruik “Bevestigen zonder betaling” op de boekingenpagina van je dashboard om haar kosteloos te bevestigen.
ton-bae-body-reschedule-unpaid = Deze boeking is niet betaald en kan dus niet verplaatst worden. Betaal haar, of annuleer haar en boek een andere tijd.
ton-bae-body-pay-link-invalid = Deze betaallink is ongeldig.

# Payment page
ton-pay-page-title = Betaling: { $title }
ton-pay-held-until = De tijd wordt voor je vastgehouden tot { $time } ({ $zone }).
ton-pay-left = nog %1
ton-pay-amount-label = Te betalen bedrag
ton-pay-rate = ≈ $ { $usd } · 1 TON = $ { $rate } ({ $sources }, { $time } UTC)
ton-pay-open-wallet = Andere wallet
ton-pay-open-tonkeeper = Openen in Tonkeeper
ton-pay-qr-alt = QR-code met de betaalgegevens
ton-pay-scan = Of scan deze code met de app van je wallet.
ton-pay-qr-alt-tonkeeper = QR-code voor Tonkeeper met de betaalgegevens
ton-pay-scan-tonkeeper = Of scan deze code met Tonkeeper of de camera van je telefoon.
ton-pay-qr-other = QR-code voor een andere wallet (bijv. MyTonWallet)
ton-pay-qr-tonkeeper = QR-code voor Tonkeeper
ton-pay-address = Adres
ton-pay-comment = Opmerking
ton-pay-copy = Kopiëren
ton-pay-copied = Gekopieerd
ton-pay-warn-comment = De opmerking is verplicht: daaraan herkennen we je betaling.
ton-pay-warn-encrypt = Versleutel de opmerking niet.
ton-pay-warn-token = Betaal in TON, niet met tokens.
ton-pay-warn-exchange = Betaal je vanaf een exchange? Zet de code hierboven in het memoveld (opmerking).
ton-pay-warn-deadline = Verstuur de betaling vóór { $time } ({ $zone }). Een later verstuurde betaling telt misschien niet mee.
ton-pay-check = Ik heb betaald, nu controleren
ton-pay-email = Stuur me deze link per e-mail
ton-pay-emailed = We hebben deze link naar het e-mailadres van je boeking gestuurd.
ton-pay-email-unavailable = De link kon niet verstuurd worden. Houd deze pagina open of bewaar het adres ervan.
ton-pay-checking-now = We controleren je betaling. Deze pagina werkt zichzelf bij.
ton-pay-state-waiting = We wachten op je betaling.
ton-pay-state-seen = Je betaling is zichtbaar op het netwerk. We wachten tot ze definitief is, wat meestal een paar seconden duurt.
ton-pay-state-partial = We hebben { $paid } TON ontvangen. Betaal de resterende { $rest } TON met dezelfde opmerking.
ton-pay-state-checking = De betaaltermijn is verstreken. Als je de betaling al hebt verstuurd, controleren we die. Deze pagina werkt zichzelf bij.
ton-pay-state-paid = Betaling ontvangen. Je boeking wordt bevestigd…
ton-pay-state-confirmed = Betaald. Je boeking is bevestigd en de bevestigingsmail is onderweg.
ton-pay-state-waived = Voor deze boeking is geen betaling nodig. De host bevestigt haar.
ton-pay-state-busy = Betaling ontvangen, maar deze tijd is niet meer vrij. We nemen contact met je op.
ton-pay-state-expired = De betaaltermijn is verstreken en de tijd is vrijgegeven. Heb je toch betaald, schrijf ons dan vanaf het e-mailadres van je boeking en we betalen het terug.
ton-pay-state-late = Je betaling kwam na de termijn binnen, dus de boeking is niet bevestigd. We nemen contact op over de terugbetaling. Je kunt ons ook schrijven vanaf het e-mailadres van je boeking.
ton-pay-state-closed = Deze boeking is niet meer actief.
ton-pay-state-revoked = Dit betaaladres wordt niet meer gebruikt. Stuur er niets naartoe en boek opnieuw.
ton-pay-rebook = Een andere tijd boeken

# Leaving the embedded widget
ton-exit-title = Tijd vastgehouden
ton-exit-body = De tijd wordt voor je vastgehouden tot { $time } ({ $zone }). Rond de betaling af om je boeking te bevestigen.
ton-exit-go = Naar de betaling

# Dashboard
ton-dash-badge-awaiting = wacht op betaling · { $amount } TON · tot { $time }
ton-dash-badge-partial = deels betaald · { $paid } van { $amount } TON · tot { $time }
ton-dash-badge-paid = betaald · { $amount } TON
ton-dash-badge-overpaid = betaald · { $excess } TON te veel
ton-dash-badge-busy = betaald, tijd niet meer vrij
ton-dash-badge-confirm-failed = betaald, niet bevestigd · bevestig met de hand
ton-dash-badge-refund = { $amount } TON betaald · terugbetaling nodig
ton-dash-badge-waived = bevestigd zonder betaling
ton-dash-badge-unpaid = niet betaald
ton-dash-waive = Bevestigen zonder betaling
ton-dash-error-unpaid = Deze boeking is niet betaald en kan dus niet bevestigd of verplaatst worden. Gebruik “Bevestigen zonder betaling” om haar kosteloos te bevestigen, of wijs haar af.
ton-dash-error-waive-failed = De boeking kon niet zonder betaling bevestigd worden: ze wacht niet meer op betaling.
ton-waive-page-title = Bevestigen zonder betaling
ton-waive-heading = Deze boeking bevestigen zonder betaling?
ton-waive-body = Deze boeking is NIET betaald ({ $amount } TON, te betalen vóór { $time }). Als je haar bevestigt, krijgt de gast de afspraak gratis, en een betaling die later binnenkomt moet je met de hand terugbetalen.
ton-waive-submit = Kosteloos bevestigen
ton-waive-back = Terug naar boekingen

# Guest emails
ton-email-link-subject = Rond je boeking af: { $event }
ton-email-link-message = Je bent begonnen met het boeken van { $event } met { $host }. Rond je boeking af op de pagina hieronder voordat de tijd om is.
ton-email-link-action = Boeking afronden
ton-email-link-line = Rond je boeking af: { $url }
ton-email-expired-subject = Niet geboekt: { $event }, { $date }
ton-email-expired-message = De betaling voor { $event } is niet op tijd binnengekomen, dus de tijd is vrijgegeven. Je kunt gerust een andere tijd kiezen.
ton-email-expired-partial = We hebben een deel van de betaling ontvangen. We nemen contact op over de terugbetaling.
ton-email-rebook-line = Kies een andere tijd: { $url }

# Owner emails
ton-email-row-amount = Bedrag:
ton-email-row-paid = Ontvangen:
ton-email-row-excess = Te veel betaald:
ton-email-row-transfer = Overboeking:
ton-email-row-sender = Afzender:
ton-email-row-tx = Transactie:
ton-email-row-detail = Details:
ton-email-amount-value = { $amount } TON ≈ $ { $usd } (1 TON = $ { $rate })
ton-email-sender-wallet = { $address } (een wallet)
ton-email-sender-other = { $address } (geen wallet: een exchange of een contract)
ton-email-dashboard-line = Boekingen: { $url }
ton-email-dashboard-action = Boekingen openen
ton-email-refund-rule = Terugbetalingen doe je met de hand vanuit je wallet: naar het adres van de afzender als dat een wallet is; anders alleen naar een adres waar de gast om vraagt vanaf het e-mailadres van de boeking, in een bericht met diens betaal- of annuleerlink. De boekingscode, het bedrag en de transactie zijn openbaar en bewijzen niets.
ton-email-received-subject = Betaling ontvangen: { $event }, { $guest } ({ $date })
ton-email-received-heading = Betaling ontvangen
ton-email-received-message = { $guest } heeft { $amount } TON betaald. De boeking is bevestigd.
ton-email-alert-subject = TON-betaling vraagt aandacht: { $what }
ton-email-alert-heading = Een betaling vraagt je aandacht
ton-alert-partial = gedeeltelijke betaling
ton-alert-partial-body = Een deel van het verschuldigde bedrag is binnengekomen. De boeking blijft onbevestigd tot de rest binnenkomt; anders wordt ze op de termijn vrijgegeven en moet het ontvangen bedrag terugbetaald worden.
ton-alert-overpaid = te veel betaald
ton-alert-overpaid-body = Er is een betaling binnengekomen voor een boeking die al betaald is. Betaal haar terug als dat de moeite waard is.
ton-alert-late = late betaling
ton-alert-late-body = Er kwam een betaling binnen na de termijn. De boeking was al vrijgegeven en kan niet meer worden bevestigd. Betaal het bedrag terug, of vraag de gast opnieuw te boeken en bevestig de nieuwe boeking zonder betaling.
ton-alert-unmatched = onbekende betaling
ton-alert-unmatched-body = Er is een betaling binnengekomen zonder bekende boekingscode. Zoek de bijbehorende boeking, of betaal haar terug.
ton-alert-failed = mislukte overboeking
ton-alert-failed-body = Een overboeking naar het betaaladres is op het netwerk mislukt. Het geld staat misschien nog in je wallet: controleer de transactie in de explorer.
ton-alert-not-text = overboeking zonder leesbare opmerking
ton-alert-not-text-body = Er is een overboeking binnengekomen met een versleutelde opmerking, een token of andere inhoud. Die telt nooit automatisch mee: controleer haar met de hand.
ton-alert-other = ongebruikelijke overboeking
ton-alert-other-body = Een overboeking kon niet automatisch verwerkt worden. Controleer haar in het betaaloverzicht.
ton-email-problem-subject = Probleem met TON-betaling: { $event } ({ $date })
ton-email-problem-urgent-subject = Dringend: een betaalde boeking vraagt een beslissing: { $event } ({ $date })
ton-email-problem-heading = Een betaalde boeking vraagt je aandacht
ton-problem-host-busy = De gast heeft betaald, maar de tijd is niet meer vrij (een agenda-item of een andere boeking). De tijd blijft vastgehouden tot de afspraak begint. Vraag de gast om een nieuwe tijd, bevestig de boeking met de hand, of wijs haar af en betaal terug.
ton-problem-partial = Er is maar een deel van het bedrag betaald, en daarna is de boeking verlopen, geannuleerd of afgewezen, of zonder betaling bevestigd. Betaal het ontvangen bedrag terug, tenzij je iets anders met de gast hebt afgesproken.
ton-problem-overpaid = De boeking is betaald en bevestigd met meer dan het verschuldigde bedrag. Betaal het teveel terug als dat de moeite waard is.
ton-problem-refund-due = De boeking is betaald en daarna geannuleerd of afgewezen. Betaal terug.
ton-problem-confirmed-unpaid = Een onbetaalde boeking blijkt bevestigd. Dat mag nooit gebeuren: controleer hoe ze bevestigd is.
ton-problem-address-revoked = Het betaaladres van deze boeking is ingetrokken, en de boeking wordt vrijgegeven.
ton-email-health-subject = TON-betalingen vragen aandacht ({ $kind })
ton-email-health-heading = TON-betalingen vragen aandacht
ton-health-indexer-down = De betalingsbewaking kan de indexer niet bereiken. Betalingen worden niet gecontroleerd en nieuwe betaalde boekingen worden geweigerd tot hij weer bereikbaar is.
ton-health-auth-failed = De indexer heeft de API-sleutel geweigerd (fout, of voor het andere netwerk). Controleer CALRS_TON_API_KEY.
ton-health-wrong-network = De indexer antwoordt voor een ander netwerk dan het ingestelde. Tot dit opgelost is, wordt niets bijgeschreven en verloopt niets.
ton-health-wallet-inactive = De ontvangende wallet is niet actief. Betaalde boekingen worden geweigerd tot er minstens één uitgaande overboeking vanaf is gedaan.
ton-health-lagging = De betalingsbewaking loopt achter op de overboekingen naar het adres (misschien spam). Onbetaalde boekingen verlopen pas als ze is bijgewerkt.
ton-health-stuck-finality = Een overboeking is al lang niet definitief. De bewaking wacht erop voordat ze verder leest.
ton-health-watcher-crashed = De betalingsbewaking is meerdere keren gecrasht en start steeds opnieuw. Controleer het serverlogboek.
ton-health-disabled-with-open = TON-betalingen staan uit terwijl er onbetaalde boekingen open staan. Hun tijden worden op hun termijn vrijgegeven.
ton-health-other-network-open = Er staan boekingen open op een ander TON-netwerk dan het ingestelde. Ze worden op hun termijn vrijgegeven.
ton-health-rate-unavailable = Er is geen bruikbare TON-wisselkoers (bronnen ontbreken, wijken af of vallen buiten je bandbreedte). Betaalde boekingen worden geweigerd tot er een is.
ton-health-capacity = De limiet van open of dagelijkse onbetaalde boekingen is bereikt. Nieuwe betaalde boekingen worden voorlopig geweigerd.
ton-health-other = De betalingsbewaking heeft een probleem gemeld. Controleer het serverlogboek.

# TON follow-ups: releases, refunds, contact, resend
ton-resend-submit = Mail mij de links
ton-resend-title = Kijk in je mailbox
ton-resend-done = Als er een onbetaalde boeking voor dit e-mailadres is, hebben we de betaal- en annuleerlinks ernaartoe gestuurd.
ton-pay-state-taken = Deze tijd werd bezet voordat de boeking was betaald, dus de boeking is vrijgegeven. Er hoeft niets betaald te worden. Kies een andere tijd.
ton-pay-state-refund = Deze boeking is beëindigd zonder bevestiging. De { $paid } TON die je betaalde wordt terugbetaald; de host neemt contact met je op.
ton-pay-contact = Vragen? Schrijf naar { $host }: { $email }.
ton-pay-copy-address = Adres kopiëren
ton-pay-copy-comment = Opmerking kopiëren
ton-pay-copy-manual = Kopiëren lukte niet. De tekst is geselecteerd: kopieer hem met de hand.
ton-email-row-pay-by = Betalen vóór:
ton-email-taken-message = De tijd die je voor { $event } koos werd bezet voordat de boeking was betaald, dus de boeking is vrijgegeven. Er hoeft niets betaald te worden. Je kunt gerust een andere tijd kiezen.
ton-email-refund-message = We hebben je betaling voor { $event } ontvangen, maar de tijd kon niet worden bevestigd, dus de boeking is vrijgegeven. { $host } betaalt het bedrag terug en neemt contact met je op.
ton-problem-host-busy-unpaid = De tijd van deze onbetaalde boeking werd bezet (er is een aanvraag overheen goedgekeurd, of er kwam een agenda-item bij), dus de boeking is vóór de betaling vrijgegeven. Als er al iets is ontvangen, betaal het terug.
ton-problem-confirm-failed = De gast heeft betaald, maar de boeking kon na meerdere pogingen niet automatisch worden bevestigd (een fout, geen bezette host: zie het serverlog). De tijd blijft vastgehouden tot de afspraak begint. Bevestig de boeking met de hand, of wijs haar af en betaal de betaling terug.
ton-health-capacity-near = Een nieuwe betaalde boeking nam de laatste open plek of het laatste dagbudget in. Volgende worden geweigerd tot boekingen betaald of vrijgegeven zijn. Als dit geen echte vraag is, houdt iemand misschien tijden vast zonder te betalen.
ton-health-payment-alerts-held = Er zijn meer overboekingen binnengekomen voor boekingen waarover je het afgelopen uur al een e-mail kreeg. Ze zijn vastgelegd zonder aparte e-mail: bekijk het betaaloverzicht.
ton-waive-body-partial = Deze boeking is maar deels betaald ({ $paid } van { $amount } TON ontvangen, te betalen vóór { $time }). Als je haar bevestigt, krijgt de gast de afspraak zonder de resterende { $rest } TON te betalen. Betaal het ontvangen bedrag terug, tenzij anders afgesproken.

## Telegram (ilia fork)

tg-when = { $date }, { $time } ({ $tz })
tg-in-minutes =
    { $count ->
        [one] over { $count } minuut
       *[other] over { $count } minuten
    }
tg-in-hours =
    { $count ->
        [one] over { $count } uur
       *[other] over { $count } uur
    }
tg-in-days =
    { $count ->
        [one] over { $count } dag
       *[other] over { $count } dagen
    }
tg-reason = Reden: { $reason }
tg-link-line = { $label }: { $url }
tg-guest-confirmed = Boeking bevestigd: { $event }, { $when }. Gastheer: { $host }.
tg-guest-pending = Aanvraag verstuurd: { $event }, { $when }. Je krijgt het antwoord van de gastheer hier.
tg-guest-approved = Je aanvraag is bevestigd: { $event }, { $when }.
tg-guest-reschedule-approved = Je nieuwe tijd is bevestigd: { $event }, { $when }.
tg-guest-declined = Je aanvraag is afgewezen: { $event }, { $when }.
tg-guest-cancelled-by-host = De gastheer heeft geannuleerd: { $event }, { $when }.
tg-guest-cancelled-by-guest = Je hebt je boeking geannuleerd: { $event }, { $when }.
tg-guest-rescheduled = Boeking verplaatst: { $event } is nu { $when }.
tg-guest-reschedule-pending = Nieuwe tijd aangevraagd: { $event }, { $when }. Je krijgt het antwoord van de gastheer hier.
tg-guest-reschedule-requested = De gastheer vraagt je een nieuwe tijd te kiezen voor { $event }. Was: { $when }.
tg-guest-reminder = Herinnering: { $event } begint { $in }, { $when }.
tg-guest-expired = Aanvraag verlopen: { $event }, { $when }. De gastheer heeft niet op tijd gereageerd; boek een andere tijd.
tg-button-cancel = Boeking annuleren
tg-button-reschedule = Verplaatsen
tg-button-calendar = Toevoegen aan agenda
tg-button-pick-time = Nieuwe tijd kiezen
tg-button-rebook = Andere tijd boeken
tg-button-bookings = Boekingen openen
tg-host-new-booking = Nieuwe boeking: { $event }, { $when }. Gast: { $guest }, { $email }.
tg-host-approval-request = Aanvraag om goed te keuren: { $event }, { $when }. Gast: { $guest }, { $email }.
tg-host-reschedule-approval = Nieuwe tijd om goed te keuren: { $event }, { $when }. Gast: { $guest }, { $email }.
tg-host-rescheduled = { $guest } heeft { $event } verplaatst naar { $when }.
tg-host-cancelled = { $guest } heeft geannuleerd: { $event }, { $when }.
tg-host-guest-linked = { $guest } krijgt Telegram-meldingen voor { $event }, { $when }: { $username }.
tg-host-guest-linked-differs = Ingevuld op het formulier: { $entered }.
tg-host-no-username = geen gebruikersnaam
tg-bot-help = Deze bot stuurt meldingen over boekingen op { $site }. Open de link op je bevestigingspagina of in je e-mail om ze aan te zetten; bij een betaalde boeking staat de link op de betaalpagina. /stop zet ze uit. Zodra een boeking aan deze chat gekoppeld is, kun je hier ook de gastheer schrijven.
tg-bot-linked-guest = Klaar: meldingen over deze boeking komen hier binnen.
tg-bot-linked-host = Klaar: gastheermeldingen voor { $name } komen hier binnen.
tg-bot-link-invalid = Deze link werkt niet meer: hij geldt maar in één Telegram-chat en alleen tot het einde van de afspraak.
tg-bot-link-replaced = Deze link is vervangen. Open de betaalpagina van je boeking opnieuw en gebruik de Telegram-link daar.
tg-bot-rate-limited = Te veel pogingen. Probeer het later opnieuw.
tg-bot-stopped = Meldingen staan uit. Open voor een boeking de link uit de bevestigingsmail opnieuw om ze weer aan te zetten; gastheren maken een nieuwe link aan in Profiel en instellingen → Telegram.
tg-bot-not-linked = Deze chat krijgt geen meldingen.
tg-cmd-help = Hoe deze bot werkt
tg-cmd-stop = Meldingen uitzetten
tg-book-label = Telegram
tg-book-placeholder = @gebruikersnaam
tg-book-help-optional = Na het boeken kun je meldingen in Telegram aanzetten.
tg-book-help-required = Open na het boeken de Telegram-link om meldingen te krijgen.
tg-book-invalid-title = Controleer je Telegram-gebruikersnaam
tg-book-invalid = Vul een Telegram-gebruikersnaam in zoals @naam (letters, cijfers en _, 4 tot 32 tekens).
tg-book-required = Vul je Telegram-gebruikersnaam in.
tg-confirmed-title = Meldingen in Telegram
tg-confirmed-body = Open de bot en druk op de knop onderaan de chat: meldingen over deze boeking komen daar binnen.
tg-confirmed-button = Telegram openen
tg-email-button = Meldingen in Telegram
tg-email-plain = Meldingen in Telegram: { $url }
tg-settings-nav = Telegram-meldingen
tg-settings-title = Telegram
tg-settings-intro = Krijg gastheermeldingen in je eigen Telegram-chat: nieuwe boekingen, aanvragen om goed te keuren, annuleringen en verplaatsingen door gasten.
tg-settings-off = Telegram is op deze server niet ingesteld.
tg-settings-no-bot-name = De bot is nog niet klaar. Probeer het over een minuut opnieuw.
tg-settings-linked = Gastheermeldingen gaan naar { $username }. Gekoppeld: { $date }.
tg-settings-not-linked = Je Telegram is niet gekoppeld.
tg-settings-link = Mijn Telegram koppelen
tg-settings-link-open = Telegram openen
tg-settings-link-expires = Druk in de chat op de knop onderaan. De link werkt één keer binnen 15 minuten.
tg-settings-link-limited = Te veel links dit uur. Probeer het later opnieuw.
tg-settings-unlink = Ontkoppelen
tg-settings-modes = Telegram van de gast per afspraaktype
tg-settings-modes-help = Optioneel of verplicht: het formulier vraagt naar de Telegram van de gast en de bevestiging biedt de link naar de bot.
tg-settings-modes-none = Geen afspraaktypes om in te stellen.
tg-settings-mode-off = Uit
tg-settings-mode-optional = Optioneel
tg-settings-mode-required = Verplicht
tg-settings-save = Opslaan
tg-settings-saved = Opgeslagen.
tg-settings-team = Team { $team }
tg-dashboard-label = Telegram { $username }
tg-dashboard-entered = (ingevuld: { $entered })
tg-dashboard-not-linked = (nog niet gekoppeld)
tg-bot-already-linked = Staat al aan: meldingen over deze boeking komen in deze chat.
tg-bot-link-invalid-host = Deze link is verlopen: links uit de instellingen werken één keer binnen 15 minuten. Maak een nieuwe aan in Profiel en instellingen → Telegram.
tg-settings-linked-no-username = Gastheermeldingen gaan naar je Telegram-chat. Gekoppeld: { $date }.
tg-bot-description = Deze bot stuurt meldingen over je boekingen op { $site }: bevestigingen, wijzigingen en herinneringen. Druk op de knop hieronder om ze voor je boeking aan te zetten. /stop zet ze uit.
tg-bot-short-description = Boekingsmeldingen van { $site }.
tg-button-join = Deelnemen

## Telegram: TON payments (ilia fork; docs/ton.md)

tg-guest-payment-due = Je boeking wacht op betaling: { $event }, { $when }. Bedrag: { $amount } TON (≈ $ { $usd }). Betaal vóór { $deadline }, anders wordt de tijd vrijgegeven.
tg-guest-payment-partial = Een deel van de betaling voor { $event }, { $when } is binnen: { $paid } van { $amount } TON. Betaal de resterende { $rest } TON vóór { $deadline }, anders wordt de tijd vrijgegeven.
tg-guest-payment-received = Betaling ontvangen, je boeking is bevestigd: { $event }, { $when }. Gastheer: { $host }.
tg-guest-payment-late = Een betaling van { $value } TON voor { $event }, { $when } kwam na de deadline binnen en heeft de boeking dus niet bevestigd. { $host } neemt contact met je op over de terugbetaling.
tg-guest-payment-expired = De tijd om { $event }, { $when } te betalen is verstreken, dus de tijd is vrijgegeven. Je kunt een andere tijd boeken.
tg-guest-payment-taken = De tijd die je voor { $event }, { $when } koos, werd bezet voordat de boeking betaald was, dus de boeking is vrijgegeven. Je hoeft niets te betalen. Je kunt een andere tijd boeken.
tg-guest-payment-refund = We hebben je betaling voor { $event }, { $when } ontvangen, maar de tijd kon niet worden bevestigd, dus de boeking is vrijgegeven. { $host } betaalt het bedrag terug en neemt contact met je op.
tg-guest-payment-refund-part = Een deel van de betaling was al binnen: { $host } neemt contact met je op over de terugbetaling.
tg-guest-payment-held = We hebben je betaling voor { $event }, { $when } ontvangen, maar de boeking kon niet automatisch worden bevestigd. { $host } neemt contact met je op.
tg-button-pay = Betalen
tg-host-payment-received = Betaald: { $event }, { $when }. Gast: { $guest }, { $email }. { $paid } TON ontvangen (≈ $ { $usd }); de boeking is bevestigd.
tg-host-payment-problem = Betaling vraagt aandacht: { $event }, { $when }. Gast: { $guest }, { $email }.
tg-host-payment-transfer = Overboeking: { $value } TON.
tg-host-payment-amounts = Ontvangen: { $paid } van { $amount } TON.
tg-host-payment-guest-telegram = Telegram van de gast: { $username }.

## ilia fork: booking flow v2 (docs/ton.md, docs/operations.md)
troubleshoot-label-busy = Bezet
ton-etf-currency = Valuta
ton-etf-currency-ton = TON
ton-etf-currency-usd = USD (omgerekend naar TON)
ton-etf-price-help-ton = In TON: van 1 tot 100000, de gast betaalt precies dit bedrag (100 TON als het veld leeg blijft).
ton-etf-usd-no-rates = Op deze server zijn geen wisselkoersen ingesteld (CALRS_TON_RATE_MIN_USD en CALRS_TON_RATE_MAX_USD), dus een prijs in USD kan niet worden geboekt. Geef de prijs op in TON.
ton-form-error-price-ton = Vul een prijs in TON in van 1 tot 100000, met hoogstens twee decimalen.
ton-form-error-currency = Kies TON of USD als valuta van de prijs.
ton-form-error-telegram = Kies voor de Telegram van de gast: uit, optioneel of verplicht.
ton-etf-telegram = Telegram van de gast
ton-etf-telegram-help = Standaard verplicht bij een betaalde boeking: de gast krijgt dan betalingsherinneringen en is bereikbaar via Telegram. Je kunt het optioneel maken of uitzetten.
ton-slots-price-ton = Prijs: { $ton } TON
ton-email-amount-value-ton = { $amount } TON
tg-guest-payment-due-ton = Je boeking wacht op betaling: { $event }, { $when }. Bedrag: { $amount } TON. Betaal vóór { $deadline }, anders wordt de tijd vrijgegeven.
tg-host-payment-received-ton = Betaald: { $event }, { $when }. Gast: { $guest }, { $email }. { $paid } TON ontvangen; de boeking is bevestigd.
ton-pay-renewed = De termijn is verlengd ({ $n } van { $max }).
ton-dash-badge-renewed = termijn verlengd { $n } van { $max }
ton-email-renewed-subject = Je tijd wordt nog vastgehouden: { $event }
ton-email-renewed-message = We hebben de betaling voor { $event } met { $host } nog niet ontvangen, dus je tijd wordt vastgehouden tot { $deadline }. Heb je al betaald, dan hoef je niets te doen: de betaling wordt meegeteld. Betaal anders vóór die tijd op de pagina hieronder, of annuleer de boeking als je plannen zijn veranderd.
tg-guest-payment-renewed = De betaling voor { $event }, { $when } is nog niet binnen, dus de tijd wordt voor je vastgehouden tot { $deadline }. Betaal de resterende { $rest } TON vóór die tijd, anders wordt de tijd vrijgegeven. Heb je al betaald, dan hoef je niets te doen: de betaling wordt meegeteld.

## ilia fork: messages with guests (docs/operations.md)
msg-page-title = Berichten
msg-back = Boekingen
msg-guest = Gast: { $guest }
msg-sent = Bericht verzonden.
msg-empty = Nog geen berichten.
msg-label = Bericht aan de gast
msg-help-email = De gast krijgt het per e-mail van het systeemadres; zijn antwoorden verschijnen hier en in je Telegram.
msg-help-no-email = E-mail is op deze server niet ingesteld.
msg-help-telegram = De gast heeft Telegram gekoppeld en krijgt het daar ook.
msg-help-no-telegram = De gast heeft Telegram niet gekoppeld.
msg-help-limits = Maximaal 20 berichten per boeking per dag, waarvan 5 per e-mail.
msg-send = Versturen
msg-link = Berichten
msg-link-count = Berichten ({ $count })
msg-from-host = Gastheer
msg-from-guest = Gast
msg-from-attendee = Deelnemer: { $address }
msg-from-other = Ander adres, niet dat van de gast: { $address }
msg-via-dashboard = dashboard
msg-via-mcp = agent
msg-via-telegram = Telegram
msg-status-email-queued = e-mail in de wachtrij
msg-status-email-sent = e-mail verzonden
msg-status-email-failed = e-mail niet bezorgd
msg-status-telegram-queued = Telegram in de wachtrij
msg-status-telegram-sent = naar Telegram verzonden
msg-status-telegram-failed = niet bezorgd op Telegram
msg-error-empty = Schrijf eerst een bericht.
msg-error-too-long = Een bericht mag maximaal 2000 tekens hebben.
msg-error-limit = De daglimiet voor berichten bij deze boeking is bereikt. Probeer het morgen opnieuw.
msg-error-no-channel = De gast is niet bereikbaar: e-mail is op deze server niet ingesteld en de gast heeft Telegram niet gekoppeld.
msg-error-closed = Berichten kunnen alleen bij openstaande of bevestigde boekingen, tot een week na de afspraak.
msg-email-subject = Bericht van { $host }: { $event }, { $date }
msg-email-intro = { $host } heeft je een bericht gestuurd over { $event }:
msg-email-footer = Beantwoord deze e-mail om { $host } te antwoorden.
tg-guest-host-message = Bericht van { $host } over { $event }, { $when }:
tg-guest-reply-hint = Schrijf je antwoord hier om te reageren.
tg-host-guest-message = Bericht van { $guest } ({ $email }) over { $event }, { $when }:
tg-host-reply-hint = Beantwoord dit bericht om de gast te antwoorden.
tg-bot-message-sent = Je bericht is doorgegeven aan de gastheer.
tg-bot-message-pick-booking = Er zijn meerdere boekingen aan deze chat gekoppeld. Beantwoord een bericht over de boeking die je bedoelt.
tg-bot-message-closed = Deze boeking neemt geen berichten meer aan.
tg-bot-message-after-payment = Berichten aan de gastheer zijn mogelijk zodra de boeking betaald is.
tg-bot-message-limit = Te veel berichten. Probeer het later opnieuw.
tg-bot-reply-sent = Naar de gast verzonden.
tg-bot-reply-not-linked = Deze gast heeft Telegram niet gekoppeld, dus het antwoord is niet verzonden. Schrijf vanuit de boeking in het dashboard.
tg-bot-reply-not-allowed = Antwoord niet verzonden: de boeking is niet van jou of neemt geen berichten meer aan.
msg-error-internal = Het bericht kon niet worden opgeslagen, dus er is niets verzonden. Probeer het opnieuw.
msg-link-new = Berichten ({ $count } nieuw)
msg-recent-title = Recente afspraken
msg-recent-help = Afspraken die zijn begonnen of in de afgelopen week zijn afgelopen. Je kunt nog berichten uitwisselen met hun gasten.
tg-bot-message-sent-shortened = Je bericht is doorgegeven aan de gastheer, ingekort tot { $max } tekens.
tg-bot-reply-too-long = Antwoord niet verzonden: een bericht mag hoogstens { $max } tekens hebben.
tg-bot-reply-empty = Antwoord niet verzonden: er staat geen zichtbare tekst in.
tg-bot-reply-failed = Het antwoord kon niet worden opgeslagen en is dus niet verzonden. Probeer het opnieuw.
troubleshoot-not-synced = Agenda's worden alleen gelezen van { $from } tot { $until }; afspraken buiten deze dagen worden niet geladen en zijn hier dus niet te zien.
event-type-form-horizon-capped =
    { $days ->
        [one] Deze server biedt tijden aan tot hoogstens { $days } dag vooruit, ongeacht de boekingshorizon.
       *[other] Deze server biedt tijden aan tot hoogstens { $days } dagen vooruit, ongeacht de boekingshorizon.
    }

## ilia fork: the system mailbox (src/inbox)
msg-help-email-no-inbox = De gast krijgt het per e-mail van het systeemadres. Antwoorden per e-mail worden op deze server niet gelezen.
msg-via-email = e-mail
msg-email-footer-no-reply = Het boekingssysteem stuurde dit bericht namens { $host }; antwoorden worden niet gelezen.
tg-host-guest-email = E-mail van { $guest } ({ $email }) over { $event }, { $when }:
tg-host-attendee-email = E-mail van { $sender }, extra deelnemer aan de boeking van { $guest } ({ $email }), over { $event }, { $when }:
tg-host-other-email = E-mail van een ander adres, niet dat van de gast: { $sender }, in het gesprek over de boeking van { $guest } ({ $email }) over { $event }, { $when }:
booking-blocked-body-system = De organisator vraagt minstens { $minutes } minuten van tevoren bericht. Kun je niet aanwezig zijn? Schrijf dan naar <a href="{ $href }">{ $address }</a>.
booking-blocked-body-no-reply = De organisator vraagt minstens { $minutes } minuten van tevoren bericht. Kun je niet aanwezig zijn? Laat het de organisator dan op een andere manier weten.

## ilia fork: passkeys and security keys (docs/passkeys-plan.md)

ilia-passkey-sign-in = Inloggen met passkey
ilia-mfa-login-help-passkey = Bevestig deze aanmelding met je passkey of beveiligingssleutel. De aanmelding verloopt na 10 minuten.
ilia-mfa-use-totp = Code uit de app gebruiken
ilia-mfa-use-recovery = Herstelcode gebruiken
ilia-mfa-use-passkey = Passkey gebruiken
ilia-mfa-use-passkey-instead = In plaats daarvan een passkey of beveiligingssleutel gebruiken
ilia-mfa-totp-login-help = Voer de zescijferige code uit je authenticator-app in. De aanmelding verloopt na 10 minuten.
ilia-mfa-recovery-code = Herstelcode
ilia-mfa-recovery-login-help = Voer een van de herstelcodes in die je hebt bewaard. Elke code werkt maar één keer. De aanmelding verloopt na 10 minuten.
ilia-passkey-unsupported = Deze browser ondersteunt geen passkeys of beveiligingssleutels. Kies hieronder een andere methode of open deze pagina in een actuele browser.
ilia-passkey-noscript = Passkeys hebben JavaScript nodig. Zet het aan voor deze site of kies hieronder een andere methode.
ilia-passkey-cancelled = Het verzoek is geannuleerd of verlopen. Probeer het opnieuw of kies een andere methode.
ilia-passkey-invalid = De passkey of beveiligingssleutel kon niet worden gecontroleerd, of het verzoek is verlopen. Probeer het opnieuw of kies een andere methode.
ilia-passkey-key-type = Deze browser kan dit type sleutel niet lezen, dus hij kan hier niet worden toegevoegd. Probeer een andere browser, zoals een actuele versie van Firefox of Chrome, of kies een andere methode.
ilia-passkey-wrong-site = Passkeys werken alleen op { $origin }. Open deze pagina daar en probeer het opnieuw, of kies een andere methode.
ilia-mfa-enroll-help = Kies hoe je elke aanmelding bevestigt. Een passkey of beveiligingssleutel is de veiligste keuze: die werkt alleen op deze site, dus een nagemaakte kopie van de pagina kan hem niet gebruiken.
ilia-passkey-add-security-key = Beveiligingssleutel toevoegen
ilia-passkey-add-device = Passkey op dit apparaat of deze telefoon toevoegen
ilia-passkey-kinds-help = Een beveiligingssleutel is een USB- of NFC-sleutel, zoals een YubiKey. Een passkey op dit apparaat of deze telefoon gebruikt Touch ID, Face ID, Windows Hello of de schermvergrendeling van de telefoon en kan worden gesynchroniseerd met je andere apparaten.
ilia-mfa-use-app-instead = In plaats daarvan een authenticator-app gebruiken
ilia-passkey-name = Naam van de sleutel
ilia-passkey-name-help = Optioneel, bijvoorbeeld ‘YubiKey aan sleutelbos’ of ‘MacBook’. Maximaal 64 tekens.
ilia-passkey-default-security-key = Beveiligingssleutel { $n }
ilia-passkey-default-device = Passkey { $n }
ilia-passkey-register-help = Druk op een knop en volg je browser: steek de beveiligingssleutel in en raak hem aan, of bevestig met Touch ID, Face ID, Windows Hello of je telefoon.
ilia-passkey-exists = Deze sleutel is al geregistreerd.
ilia-mfa-recovery-help = Bewaar deze herstelcodes op een veilige plek; ze worden maar één keer getoond. Elke code vervangt bij één aanmelding een passkey of een code uit de app. Nieuwe codes vervangen alle vorige.
ilia-mfa-status-on = Tweestapsverificatie staat aan.
ilia-mfa-status-off = Tweestapsverificatie staat uit.
ilia-mfa-setup-intro = Voeg aan elke aanmelding een tweede stap toe: een passkey of beveiligingssleutel (YubiKey, Touch ID, Face ID, Windows Hello, je telefoon) of een authenticator-app.
ilia-passkey-list = Passkeys en beveiligingssleutels
ilia-passkey-none = Nog geen passkeys of beveiligingssleutels.
ilia-passkey-kind-hardware = Hardwaresleutel
ilia-passkey-kind-synced = Gesynchroniseerde passkey
ilia-passkey-added = Toegevoegd op { $date }
ilia-passkey-last-used = Laatst gebruikt op { $date }
ilia-passkey-never-used = Nog niet gebruikt
ilia-passkey-other-site = Geregistreerd voor { $rp_id }; werkt niet op dit adres van de site.
ilia-passkey-rename = Hernoemen
ilia-passkey-delete = Verwijderen
ilia-passkey-add = Passkey of beveiligingssleutel toevoegen
ilia-passkey-new-name = Nieuwe naam
ilia-passkey-limit = Je hebt het maximumaantal sleutels bereikt ({ $max }). Verwijder er een om een andere toe te voegen.
ilia-passkey-backup-advice = Voeg een tweede sleutel toe en bewaar die op een veilige plek, zodat je niet buitengesloten raakt als je er een kwijtraakt.
ilia-passkey-unavailable = Passkeys staan uit: het openbare adres van de server (CALRS_BASE_URL) is geen https://-adres. Vraag het de beheerder. Eerder toegevoegde sleutels staan in de lijst en kunnen worden verwijderd.
ilia-mfa-totp-title = Authenticator-app
ilia-mfa-totp-on = Er is een authenticator-app ingesteld.
ilia-mfa-totp-off = Er is geen authenticator-app ingesteld.
ilia-mfa-totp-add = Authenticator-app instellen
ilia-mfa-totp-remove = Authenticator-app verwijderen
ilia-mfa-recovery-title = Herstelcodes
ilia-mfa-recovery-left =
    { $count ->
        [one] Nog { $count } herstelcode over
       *[other] Nog { $count } herstelcodes over
    }
ilia-mfa-last-factor = Tweestapsverificatie is verplicht, dus je laatste aanmeldmethode kan niet worden verwijderd. Voeg eerst een andere toe.
ilia-mfa-last-factor-off = Dit is je laatste methode: als je die verwijdert, gaat tweestapsverificatie uit en worden je herstelcodes gewist.
ilia-mfa-changes-help = Elke wijziging hier vraagt om je wachtwoord en een passkey of code, en meldt je andere sessies af.
ilia-mfa-confirm-title = Bevestig dat jij het bent
ilia-mfa-confirm-help = Voer je wachtwoord in en bevestig daarna met een passkey of een code.
ilia-passkey-confirm = Bevestigen met passkey
ilia-mfa-confirm-code = Bevestigen met code
ilia-mfa-action-passkey-add = Je voegt een passkey of beveiligingssleutel toe.
ilia-mfa-action-passkey-rename = Je hernoemt de sleutel ‘{ $name }’.
ilia-mfa-action-passkey-delete = Je verwijdert de sleutel ‘{ $name }’.
ilia-mfa-action-totp-add = Je stelt een authenticator-app in.
ilia-mfa-action-totp-remove = Je verwijdert de authenticator-app.
ilia-mfa-action-regenerate = Je vervangt alle herstelcodes.
ilia-mfa-action-policy = Je stelt het tweestapsbeleid voor lokale accounts in op:
ilia-mfa-policy-continue = Doorgaan

## ilia fork: the instance name (CALRS_BRAND_NAME) and the error page (src/web/error_page.rs)
ilia-login-subtitle = Log in op je { $brand }-account
ilia-register-subtitle = Registreer een nieuw { $brand }-account
ilia-error-not-found = Niet gevonden
ilia-error-not-found-body = Er is niets op dit adres. Controleer de link.
ilia-error-gone = Deze link is niet meer geldig
ilia-error-bad-request = Ongeldig verzoek
ilia-error-forbidden = Toegang beperkt

# ilia fork: TON price by demand (the ladder)
ton-slots-ladder-now = Nu: { $ton } TON
ton-slots-ladder-now-free = Nu: gratis, de host bevestigt het verzoek
ton-slots-ladder-locked = De prijs ligt vast zodra je boekt.
ton-slots-ladder-summary-free = Gratis zolang de agenda leeg is, daarna { $from } tot { $to } TON naarmate hij voller wordt.
ton-slots-ladder-summary = { $from } tot { $to } TON, afhankelijk van hoe vol de agenda is.
ton-slots-ladder-all-free = Gratis; de host bevestigt elk verzoek.
ton-slots-ladder-fee = De netwerkkosten betaalt je wallet.
ton-quote-title = De prijs is veranderd
ton-quote-was = Je kreeg { $ton } TON te zien.
ton-quote-was-free = Je kreeg een gratis boeking te zien.
ton-quote-now = De prijs nu: { $ton } TON.
ton-quote-now-free = Nu is het gratis: de host bevestigt het verzoek.
ton-quote-why = De agenda of de prijzen zijn gewijzigd sinds de pagina werd geladen. Er is nog niets geboekt.
ton-quote-why-free-used = Je had onlangs al een gratis boeking, dus voor deze boeking betaal je. Er is nog niets geboekt.
ton-quote-confirm-help = Bevestig de prijs. Er is nog niets geboekt.
ton-quote-confirm = Boeken voor { $ton } TON
ton-quote-confirm-free = Gratis aanvragen
ton-bae-body-free-used = Je had onlangs al een gratis boeking, dus deze kan niet gratis zijn. Probeer het later opnieuw.
ton-form-error-mode = Kies een vaste prijs of de prijs naar vraag.
ton-etf-mode = Prijs
ton-etf-mode-fixed = Vaste prijs
ton-etf-mode-ladder = Naar vraag (de prijsladder van de site, in TON)
ton-etf-ladder-now = Huidige ladder:
ton-etf-ladder-help = De prijs is de trede voor het aantal komende boekingen in de komende dagen (bevestigde of betaalde, gratis aanvragen die op je goedkeuring wachten en teamboekingen die jij leidt; onbetaalde tellen niet): eerst gratis (je bevestigt elke gratis boeking zelf), daarna hoger naarmate je agenda voller wordt. Gasten zien de huidige prijs en betalen de getoonde prijs. De ladder stelt de beheerder in.

# ilia fork: the TON settings page (/dashboard/admin/ton)
nav-admin-ton = TON-betalingen
ton-etf-ladder-edit = Ladder bewerken
## ilia-ov-ton
ton-admin-debug-tg-saved-locked = Niets gewijzigd: de Telegram-gebruikersnaam van de gast is vergrendeld.
dbg-err-requote = De prijs is tijdens het boeken twee keer veranderd. Er is niets geboekt: probeer het opnieuw.
ton-admin-page-title = TON-betalingen
ton-admin-heading = TON-betalingen
ton-admin-intro = Deze instellingen gelden voor TON-betalingen op deze server. Een waarde uit de omgeving van de server gaat voor, dan een die hier of met config apply is opgeslagen, dan de standaardwaarde.
ton-admin-saved-live = Opgeslagen. De nieuwe instellingen gelden meteen; openstaande facturen houden hun bedragen en termijnen.
ton-admin-saved-off = Opgeslagen. Betalingen staan uit op deze server (CALRS_TON_NETWORK is niet ingesteld); de instellingen gelden zodra ze aan staan.
ton-admin-saved-restart = Opgeslagen. De instellingen gelden na een herstart van calrs.
ton-admin-saved-invalid = Opgeslagen, maar de instellingen van de server zijn als geheel ongeldig, dus betalingen blijven uit. Zie de status hieronder.
ton-admin-saved-none = Er is niets veranderd.
ton-admin-errors = Er is niets opgeslagen: verbeter de velden die hieronder zijn gemarkeerd.
ton-admin-status = Status
ton-admin-status-on = Betalingen staan aan
ton-admin-status-off = Betalingen staan uit
ton-admin-network = Netwerk
ton-admin-address = Walletadres
ton-admin-address-raw = Ruwe vorm
ton-admin-indexer = Indexer
ton-admin-watcher = Watcher
ton-admin-watcher-ready = klaar

## ilia-ov3-ton
ton-admin-notready-wallet = de ontvangende wallet is niet actief (of nog niet gecontroleerd)
ton-admin-notready-network = de indexer zit op een ander netwerk (of nog niet gecontroleerd)
ton-admin-notready-stale = de watcher heeft de laatste 2 minuten geen stap voltooid
ton-admin-off-unset = CALRS_TON_NETWORK is niet ingesteld

ton-admin-watcher-not-ready = niet klaar: { $reason }
ton-admin-last-step = laatste stap
ton-admin-open-invoices = Openstaande facturen
ton-admin-api-key = API-sleutel van de indexer
ton-admin-coingecko-key = CoinGecko-sleutel
ton-admin-secret-credentials = ingesteld (systemd-credentials)
ton-admin-secret-env = ingesteld (omgeving)
ton-admin-not-set = niet ingesteld
ton-admin-env-only = Het netwerk, het walletadres, de indexer en de API-sleutels worden alleen in de omgeving van de server en in systemd-credentials ingesteld, nooit op deze pagina.
ton-admin-src-env = ingesteld via de omgeving ({ $var })
ton-admin-src-apply = ingesteld via config apply
ton-admin-src-panel = hier ingesteld
ton-admin-src-default = standaard
ton-admin-locked-env = Om het hier te wijzigen, haal je { $var } uit de omgeving van de server en herstart je calrs.
ton-admin-locked-apply = Wijzig het in het document van config apply, of laat het weg uit de ton-sectie om het aan deze pagina over te laten.
ton-admin-default-hint = Laat leeg voor de standaardwaarde: { $value }.
ton-admin-blank-default = Laat leeg voor geen waarde.
ton-admin-group-pricing = Prijs naar vraag
ton-admin-group-timing = Termijnen van facturen
ton-admin-group-rate = Wisselkoers (prijzen in USD)
ton-admin-group-alerts = Meldingen
ton-admin-f-ladder = Ladder (TON naar het aantal boekingen)
ton-admin-f-free_per_guest_days = Eén gratis boeking per gast binnen (dagen, 0 = geen limiet)
ton-admin-f-window_min = Betaaltermijn (minuten)
ton-admin-f-grace_min = Uitloop na de termijn (minuten)
ton-admin-f-fuse_hours = Veiligheidsmarge na de uitloop (uren)
ton-admin-f-renew_min_lead_hours = Een onbetaalde termijn verlengen als de afspraak minstens zo ver weg is (uren)
ton-admin-f-renew_window_min = Verlengde termijn (minuten)
ton-admin-f-max_renewals = Verlengingen per factuur
ton-admin-f-underpay_tolerance_ton = Toegestane onderbetaling (TON)
ton-admin-f-rate_min_usd = Laagst toegestane koers (USD per TON)
ton-admin-f-rate_max_usd = Hoogst toegestane koers (USD per TON)
ton-admin-f-rate_max_spread_pct = Grootste verschil tussen de bronnen (%)
ton-admin-f-rate_max_age_min = Oudste toegestane koers (minuten)
ton-admin-f-rate_max_jump_pct = Grootste sprong ten opzichte van de vorige koers (%)
ton-admin-f-alert_email = E-mail voor meldingen (leeg: alle beheerders)
ton-admin-ladder-help = Treden gescheiden door komma's: de prijs bij 0, 1, 2… bevestigde boekingen in de komende dagen; de laatste trede is het plafond. "free" alleen aan het begin: een gratis boeking wacht op de goedkeuring van de eigenaar.
ton-admin-ladder-row = Boekingen: %1
ton-admin-ladder-add = Trede toevoegen
ton-admin-ladder-remove = Verwijderen
ton-admin-free = gratis
ton-admin-preview = Voorbeeld
ton-admin-preview-button = Voorbeeld van de ladder
ton-admin-previewing = Een voorbeeld van de ladder die hierboven is ingevuld. Er is nog niets opgeslagen.
ton-admin-ladder-invalid = De geldende ladder is ongeldig: { $error }
ton-admin-preview-n = Komende boekingen
ton-admin-preview-price = Prijs
ton-admin-preview-or-more = { $n } of meer
ton-admin-my-demand = Je komende boekingen nu: { $n }. Je ladderprijs nu: { $price }.
ton-admin-types = Afspraaktypes met een prijs
ton-admin-types-none = Geen enkel afspraaktype heeft een prijs.
ton-admin-type = Afspraaktype
ton-admin-type-owner = Eigenaar
ton-admin-type-price = Prijs
ton-admin-type-now = Nu
ton-admin-mode-ladder = naar vraag
ton-admin-type-off = prijs uitgeschakeld
ton-admin-type-below-ceiling = Op of onder het plafond van de ladder: een prijs voor "altijd" ligt normaal gesproken hoger.
ton-admin-save = Opslaan
ton-admin-audit = Recente wijzigingen
ton-admin-audit-none = Nog geen wijzigingen.
ton-admin-audit-at = Wanneer
ton-admin-audit-who = Wie
ton-admin-audit-key = Instelling
ton-admin-audit-change = Wijziging
ton-admin-audit-default = (standaard)
ton-admin-via-apply = config apply

## ilia fork: the debug booking page (CALRS_DEBUG_BOOKING, src/web/debug_booking.rs)

nav-admin-debug-booking = Testboeking
dbg-title = Testboeking
dbg-intro = Boek een testafspraak zoals een gast dat zou doen, zonder elke keer het gastformulier in te vullen. De boeking volgt hetzelfde pad als de openbare boekingspagina: de controles, de prijs en de TON-factuur, het vasthouden van de tijd en de e-mails. De betaling wordt nooit overgeslagen of als ontvangen gemarkeerd: betaal de factuur vanuit je wallet of annuleer de boeking.
dbg-switch-note = Zet deze pagina uit zodra het debuggen klaar is, met de schakelaar bovenaan de pagina:
dbg-cancelled = Geannuleerde testboekingen: { $count }.
dbg-no-types = Je hebt geen ingeschakeld persoonlijk afspraaktype.
dbg-event-type = Afspraaktype
dbg-priced = betaald
dbg-private-short = privé
dbg-choose = Tonen
dbg-price = Prijs
dbg-price-now-free = nu gratis (jij keurt het verzoek goed)
dbg-price-now-ton = nu { $ton } TON
dbg-price-ladder = naar vraag, vastgelegd bij het boeken
dbg-price-free-used = deze gast had onlangs al een gratis boeking, dus geldt de eerste prijs
dbg-free = gratis
dbg-private = Dit afspraaktype is niet openbaar: voor elke testboeking wordt een eenmalige uitnodiging voor je eigen adres gemaakt.
dbg-ton-off = TON-betalingen staan op deze server uit, dus een betaald afspraaktype kan nu niet worden geboekt.
dbg-captcha-on = Captcha staat aan op de boekingsformulieren. Deze pagina kan het niet oplossen, dus zolang captcha aan staat weigert het boekingspad testboekingen.
dbg-err-username = Je account heeft geen gebruikersnaam en dus geen boekingspagina's.
dbg-slot = Tijd
dbg-tz = Tijden in { $tz }, de tijdzone van je profiel.
dbg-no-slots = Geen vrije tijd in de komende 31 dagen.
dbg-name = Naam
dbg-email = E-mail
dbg-email-help = Alleen je eigen adressen: je inlog-e-mail en je boekings-e-mail.
dbg-telegram-username = Telegram-gebruikersnaam
dbg-telegram-required = verplicht voor dit afspraaktype
dbg-notes = Notities
dbg-notes-default = Testboeking vanaf de debugpagina.
dbg-own-chat = Telegram: berichten voor de gast in mijn eigen gekoppelde Telegram-chat afleveren
dbg-own-chat-off = Telegram staat op deze server uit (geen bottoken).
dbg-own-chat-linked = Je gekoppelde chat: { $who }.
dbg-own-chat-help = De gastkant van de boeking (wat te betalen, de bevestiging, de herinnering, een annulering) komt in de chat waarin de bot je al schrijft, alsof je daar de link van de gast had geopend. Zonder vinkje toont de betaalpagina de link van de gast zoals gewoonlijk.
dbg-own-chat-not-linked = Je Telegram-chat is niet gekoppeld. Koppel hem eerst en kom dan hier terug:
dbg-own-chat-no-username = geen gebruikersnaam
dbg-test-note = Dit is een echte boeking, gemarkeerd als test. Zolang ze bestaat, houdt ze de tijd vast, telt ze mee voor de prijs naar vraag en stuurt ze de gebruikelijke e-mails en berichten naar je eigen adres. Annuleer haar als je klaar bent.
dbg-submit = Boeken als gast
dbg-list = Mijn testboekingen
dbg-list-none = Nog geen testboekingen.
dbg-col-when = Wanneer
dbg-col-type = Afspraaktype
dbg-col-status = Status
dbg-col-payment = Betaling
dbg-col-telegram = Telegram
dbg-open-pay = betaalpagina
dbg-own-chat-yes = mijn chat
dbg-cancel-all = Al mijn testboekingen annuleren ({ $count })
dbg-cancel-all-help = Annuleert elke testboeking van jou die nog staat (een openstaande wordt afgewezen) zoals het boekingsoverzicht dat doet: open facturen worden gesloten en de gebruikelijke meldingen verstuurd. Andere boekingen blijven onaangeroerd.
dbg-cancel-reason = Testboeking geannuleerd vanaf de debugpagina.
dbg-impersonating = Stop met het overnemen van een andere gebruiker om de testboekingspagina te gebruiken.
dbg-err-type = Kies een van je eigen afspraaktypes.
dbg-err-email = Gebruik een van je eigen adressen.
dbg-err-slot = Kies een tijd uit de lijst.
dbg-err-own-chat = Koppel eerst je Telegram-chat of haal het Telegram-vinkje weg.
dbg-own-chat-failed = De testboeking is gemaakt, maar de berichten voor de gast konden niet naar je Telegram-chat worden geleid: de chat is ontkoppeld, de bot is gestopt of de link is intussen verlopen. De boeking blijft staan en houdt de tijd vast: open de betaalpagina in de lijst hieronder, of annuleer de boeking.
dbg-st-pending = in afwachting
dbg-st-confirmed = bevestigd
dbg-st-cancelled = geannuleerd
dbg-st-declined = afgewezen
dbg-inv-open = wacht op betaling
dbg-inv-paid = betaald
dbg-inv-confirmed = betaald en bevestigd
dbg-inv-expired = verlopen
dbg-inv-closed = gesloten
dbg-inv-waived = bevestigd zonder betaling
dbg-banner = DEBUG · testboeking
dbg-badge-test = TEST
ton-admin-debug-title = Debug
ton-admin-debug-intro = Testboeking: boek je eigen afspraaktypes zoals een gast dat doet (de echte prijs, factuur, betaalpagina en Telegram) zonder het gastformulier in te vullen. Alleen voor beheerders. Zolang hij uit staat, antwoordt de pagina met 404 en linkt niets ernaar.
ton-admin-debug-state-on = Testboeking: aan
ton-admin-debug-state-off = Testboeking: uit
ton-admin-debug-turn-on = Aanzetten
ton-admin-debug-turn-off = Uitzetten
ton-admin-debug-open = Testboeking openen
ton-admin-debug-locked-env = Vastgelegd door de omgeving van de server ({ $var }={ $value }). Om het hier te schakelen, haal je { $var } uit de omgeving en herstart je calrs.
ton-admin-debug-locked-apply = Vastgelegd door config apply (site.ton.debug_booking). Wijzig het in het document van config apply, of laat de sleutel weg om het aan deze pagina over te laten.
ton-admin-debug-saved-on = De testboeking staat aan.
ton-admin-debug-saved-off = De testboeking staat uit.
ton-admin-debug-saved-locked = Niets gewijzigd: de schakelaar is vergrendeld.
ton-admin-f-debug_booking = Testboeking
dbg-telegram-from-chat = Ingevuld vanuit je gekoppelde Telegram-chat.
dbg-telegram-from-setting = Ingevuld vanuit het Debug-blok van
dbg-telegram-none = Niets om in te vullen: stel een naam in het Debug-blok in van
dbg-timing = Termijnen voor deze testboeking
dbg-timing-normal = Normaal: de instellingen van de site (pagina ‘TON-betalingen’)
dbg-timing-fast = Snelle test: { $window } min om te betalen; verlengingen: { $renewals }, elk { $renew } min (de gast wordt opnieuw om betaling gevraagd); daarna verloopt de factuur en wordt de boeking automatisch geannuleerd. Respijt { $grace } min, marge { $margin } min.
dbg-timing-custom = Eigen, in minuten (een leeg veld neemt de snelle waarde):
dbg-timing-window = Betaalvenster, min
dbg-timing-renew-window = Verlengd venster, min
dbg-timing-renewals = Verlengingen
dbg-timing-help = Alleen voor de factuur van deze testboeking; mag korter zijn dan de site toestaat (minstens { $min } min). Het venster eindigt nog steeds vóór de minimale aankondigingstijd van de afspraak. Onbetaald wordt de boeking automatisch geannuleerd: de watcher controleert ondertussen ongeveer elke 10 seconden (minder vaak alleen als de TON-indexer afremt of hem weigert), en calrs geeft de tijd binnen een minuut na het verlopen van de factuur vrij. De betaling wordt nooit als ontvangen gemarkeerd.
dbg-err-timing = Kies de termijnen. Eigen minuten: venster 2–240, verlengd venster 2–1440, verlengingen 0–5.
dbg-col-timing = Termijnen
dbg-timing-site = instellingen van de site
dbg-timing-summary = test: { $window } min, verlengingen { $renewals } × { $renew } min
dbg-renewals = verlengd { $n } van { $max }
dbg-cd-until = betalen vóór { $expires } (een overboeking telt tot { $settle })
dbg-cd-pay = nog { $time } om te betalen
dbg-cd-grace = venster voorbij; een overboeking in de komende { $time } telt nog
dbg-cd-wait = venster voorbij: verlengd of verlopen bij de volgende stap van de watcher
dbg-list-live = Zolang een testboeking in afwachting is, ververst deze lijst zich elke 10 seconden.
dbg-guest-lang = Taal van de gast
dbg-guest-lang-help = De taal van de pagina's, e-mails en Telegram-berichten van de gast voor deze testboeking. Standaard de taal van je eigen interface. E-mails aan hosts blijven in de taal van elke host.
dbg-err-lang = Kies een taal van de gast uit de lijst.
dbg-col-lang = Taal
dbg-no-chat-title = Je Telegram-chat is niet gekoppeld
dbg-no-chat-body = ‘Berichten voor de gast in mijn eigen gekoppelde Telegram-chat afleveren’ is pas beschikbaar als je je chat koppelt in Profiel en instellingen → Telegram. Koppel hem daar en kom dan terug naar deze pagina.
dbg-no-chat-link = Mijn Telegram-chat koppelen
ton-admin-debug-tg-label = Telegram-gebruikersnaam van de gast voor testboekingen
ton-admin-debug-tg-save = Opslaan
ton-admin-debug-tg-help = Ingevuld op de testboekingspagina als je gekoppelde Telegram-chat geen gebruikersnaam heeft (voor afspraaktypen die erom vragen). Zonder @; leeg: geen.
ton-admin-debug-tg-locked-apply = Vergrendeld door config apply (site.ton.debug_booking_telegram). Wijzig het in het document van config apply, of laat de sleutel weg om het aan deze pagina over te laten.
ton-admin-debug-tg-saved = Telegram-gebruikersnaam van de gast opgeslagen.
ton-admin-debug-tg-invalid = Niets opgeslagen: geen Telegram-gebruikersnaam (een Latijnse letter, dan letters, cijfers en _, 4 tot 32 tekens).
ton-admin-f-debug_booking_telegram = Testboeking: Telegram-gebruikersnaam van de gast

## ilia fork: event type translations (src/web/event_texts.rs)
etf-translations = Vertalingen
etf-translations-help = Gasten die de boekingspagina's in een van deze talen lezen, zien deze titel en beschrijving in plaats van die hierboven, ook in hun e-mails en berichten. Alle andere talen tonen die hierboven.
etf-translation-language = Taal
etf-translation-description-help = Ondersteunt **vet**, *cursief* en [links](url). Laat je hem leeg, dan toont deze taal geen beschrijving.
etf-translation-add = Taal toevoegen
etf-translation-remove = Verwijderen
form-error-translation-language = Kies een taal voor elke vertaling.
form-error-translation-duplicate = Elke taal kan maar één vertaling hebben.
form-error-translation-title-required = Elke vertaling heeft een titel nodig.
form-error-translation-title-length = Een vertaalde titel mag hoogstens 255 tekens lang zijn.

## ilia fork: 3CX video meetings (src/web/threecx.rs)
admin-threecx = 3CX-videovergaderingen
admin-threecx-help = Elke bevestigde boeking van een afspraaktype met als locatie ‘3CX-vergadering’ krijgt een eigen geplande vergadering op deze 3CX-telefooncentrale. Gasten krijgen de vergaderlink, hosts de organisatorlink in hun agenda en e-mails. Verplaatsen of annuleren van de boeking verplaatst of verwijdert de vergadering. Het e-mailadres van de gast gaat nooit naar 3CX, dus 3CX stuurt de gast niets.
admin-threecx-base-url = Adres van de centrale
admin-threecx-extension = Toestel van de organisator
admin-threecx-extension-help = Het toestel waar de vergaderingen bij horen, bijvoorbeeld 100.
admin-threecx-api-key = Sleutel van de videoconferentie-API
admin-threecx-api-key-help = De sleutel van de 3CX-videoconferentie-API, uit de 3CX-beheerconsole. Hij wordt versleuteld opgeslagen en nooit meer getoond.
admin-threecx-display-name-help = Wat gasten als locatie zien, bijvoorbeeld ‘3CX’. Leeg: ‘Videogesprek’.
admin-threecx-save = 3CX-instellingen opslaan
admin-threecx-test = Verbinding testen
admin-threecx-clear = 3CX-instellingen verwijderen
admin-threecx-notice-saved = 3CX-instellingen opgeslagen.
admin-threecx-notice-cleared = 3CX-instellingen verwijderd, de sleutel ook.
admin-threecx-notice-ok = 3CX antwoordde: het adres en de sleutel werken.
admin-threecx-notice-auth = 3CX weigerde de sleutel. Aanroepen naar 3CX pauzeren een uur, zodat de centrale deze server niet blokkeert; een nieuwe sleutel opslaan of opnieuw testen beëindigt de pauze.
admin-threecx-notice-transport = 3CX antwoordde niet. Controleer het adres en of deze server het kan bereiken.
admin-threecx-notice-service = 3CX antwoordde met een fout. Controleer of videoconferenties voor het toestel aanstaan.
admin-threecx-notice-bad-link = 3CX gaf een vergaderlink op een ander adres. Controleer het adres van de centrale.
admin-threecx-notice-not-set = Vul eerst het adres, het toestel en de sleutel in.
admin-threecx-notice-bad-base-url = Niet opgeslagen: het adres moet met https:// beginnen en alleen de host bevatten, zonder pad.
admin-threecx-notice-bad-extension = Niet opgeslagen: het toestel moet 1 tot 10 cijfers hebben.
admin-threecx-notice-bad-api-key = Niet opgeslagen: de sleutel moet 16 tot 256 tekens zonder spaties hebben.
admin-threecx-notice-bad-display-name = Niet opgeslagen: het label mag hoogstens 100 tekens hebben.
admin-threecx-notice-bad-pair = Niet opgeslagen: het adres en het toestel horen bij elkaar, en een sleutel heeft beide nodig.
admin-threecx-notice-key-env = Niet opgeslagen: de sleutel wordt ingesteld door de uitrol (CALRS_THREECX_API_KEY). Wijzig hem daar.
admin-threecx-api-key-env = Ingesteld door de uitrol (CALRS_THREECX_API_KEY). Wijzig hem in de geheimen van de uitrol en rol opnieuw uit.
admin-threecx-api-key-env-bad = CALRS_THREECX_API_KEY is ingesteld maar is geen sleutel (16 tot 256 tekens zonder spaties): 3CX-vergaderingen staan uit tot de uitrol dit herstelt.
admin-threecx-api-key-env-placeholder = Ingesteld door de uitrol
etf-location-threecx = 3CX-vergadering (voor elke boeking gemaakt)
etf-threecx-hint = Voor elke bevestigde boeking wordt een 3CX-vergadering gemaakt die verplaatsingen en annulering volgt. Stel 3CX in op de beheerpagina.
