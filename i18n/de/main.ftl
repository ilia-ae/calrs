# Booking confirmation page (templates/confirmed.html)

confirmed-page-title-pending = Buchung ausstehend
confirmed-page-title-booked = Buchung bestätigt

confirmed-heading-reschedule-requested = Verschiebung angefordert
confirmed-heading-rescheduled = Verschoben!
confirmed-heading-pending = Bestätigung ausstehend
confirmed-heading-booked = Termin gebucht!

confirmed-subtitle-reschedule-requested = Deine Anfrage zur Terminverschiebung wurde an { $host } gesendet. Du erhältst eine E-Mail an { $email }, sobald sie genehmigt ist.
confirmed-subtitle-rescheduled = Deine Buchung wurde verschoben. Eine Bestätigungs-E-Mail wurde an { $email } gesendet.
confirmed-subtitle-pending = Deine Buchungsanfrage wurde an { $host } gesendet. Du erhältst eine E-Mail an { $email }, sobald sie bestätigt ist.
confirmed-subtitle-booked = Eine Bestätigungs-E-Mail wurde an { $email } gesendet.

confirmed-detail-event = Termin:
confirmed-detail-date = Datum:
confirmed-detail-time = Uhrzeit:
confirmed-detail-with = Mit:
confirmed-detail-location = Ort:
confirmed-detail-notes = Notizen:
confirmed-detail-additional-guests = Weitere Teilnehmer:

confirmed-book-another = Weiteren Termin buchen

confirmed-add-to-calendar = Zum Kalender hinzufügen

# Slot picker (templates/slots.html)

slots-location-video = Videoanruf
slots-location-phone = Telefonanruf
slots-location-google-meet = Google Meet

slots-tz-label = Deine Zeitzone
slots-time-format-label = Zeitformat
tz-group-common = Häufige Zeitzonen
tz-region-africa = Afrika
tz-region-america = Amerika
tz-region-antarctica = Antarktis
tz-region-arctic = Arktis
tz-region-asia = Asien
tz-region-atlantic = Atlantik
tz-region-australia = Australien
tz-region-europe = Europa
tz-region-indian = Indischer Ozean
tz-region-pacific = Pazifik
slots-offset-note-day = Die Zeiten am %1 sind in %2 (Zeitumstellung am %3).
slots-offset-note-range = Zeitumstellung am %1: ab dann sind die Zeiten in %2.

slots-view-month = Monatsansicht
slots-view-week = Wochenansicht
slots-view-column = Listenansicht

slots-weekday-mon = Mo
slots-weekday-tue = Di
slots-weekday-wed = Mi
slots-weekday-thu = Do
slots-weekday-fri = Fr
slots-weekday-sat = Sa
slots-weekday-sun = So

slots-weekday-mon-short = M
slots-weekday-tue-short = D
slots-weekday-wed-short = M
slots-weekday-thu-short = D
slots-weekday-fri-short = F
slots-weekday-sat-short = S
slots-weekday-sun-short = S

slots-select-date = Datum auswählen
slots-loading-availability = Verfügbarkeit wird geladen...
slots-click-highlighted = Klicke auf ein hervorgehobenes Datum, um verfügbare Zeiten zu sehen
slots-bookable-range-both = Buchbar vom { $from } bis { $until }
slots-bookable-range-from = Buchbar ab { $from }
slots-bookable-range-until = Buchbar bis { $until }
slots-no-longer-bookable = Dieser Termin kann nicht mehr gebucht werden.
slots-no-times-month = Keine verfügbaren Zeiten in diesem Monat
slots-no-times-day = Keine verfügbaren Zeiten an diesem Tag
slots-no-availability-participants = Keine gemeinsame Verfügbarkeit aller Teilnehmer in diesem Monat
slots-week-more = mehr

# Booking form (templates/book.html)

book-page-title = { $title } buchen
book-back-to-times = Zurück zu den Zeiten
book-name-label = Dein Name
book-name-placeholder = Max Mustermann
book-email-label = E-Mail
book-email-placeholder = max@example.com
book-email-invalid = Bitte gib eine vollständige E-Mail-Adresse samt Domain an (z. B. jane@example.com).
book-notes-label = Notizen
book-notes-optional = (optional)
book-notes-placeholder = Möchtest du etwas Bestimmtes besprechen?
book-additional-guests-label = Weitere Teilnehmende
book-additional-guests-hint = (optional, bis zu { $max })
book-add-guest-btn = + Teilnehmer hinzufügen
book-guest-email-placeholder = kollege@example.com
book-phone-label = Telefonnummer
book-phone-placeholder = 0151 23456789
book-phone-help = Lokale Nummern sind in Ordnung; ohne führendes + wird { $country } angenommen.
book-phone-optional-consequence = Lass das Feld leer, wenn du keine SMS zu dieser Buchung erhalten möchtest.
book-phone-required = Für diese Buchung ist eine Telefonnummer erforderlich.
book-phone-invalid-title = Ungültige Telefonnummer
book-phone-invalid = Bitte gib eine per SMS erreichbare Nummer an oder lass das Feld leer.
book-phone-country-search = Suchen
book-phone-country-label = Land auswählen
book-phone-country-none = Kein Land ausgewählt
book-phone-country-no-results = Keine Länder entsprechen dieser Suche
captcha-label = Sicherheitsüberprüfung
captcha-initial-state = Bestätige, dass du ein Mensch bist
captcha-verifying = Überprüfung läuft...
captcha-solved = Du bist ein Mensch
captcha-error = Fehler
captcha-troubleshooting = Fehlerbehebung
captcha-wasm-disabled = WASM aktivieren für deutlich schnellere Lösung
captcha-verify-aria = Klicke, um zu bestätigen, dass du ein Mensch bist
captcha-verifying-aria = Überprüfung läuft, bitte warten
captcha-verified-aria = Bestätigt
captcha-required = Bitte bestätige, dass du ein Mensch bist
captcha-error-aria = Ein Fehler ist aufgetreten, bitte versuche es erneut
book-confirm-button = Buchung bestätigen

# SMS notifications (src/sms/message.rs).
#
# These are text messages, billed per 160-character segment (70 if the text
# contains any character outside the GSM-7 alphabet, which includes most
# accented letters). Keep them short and plain.

sms-confirmed = Buchung bestätigt: { $event }, { $date } um { $time } ({ $tz }).
sms-cancelled = Buchung storniert: { $event }, { $date } um { $time } ({ $tz }).
sms-rescheduled = Buchung verschoben: { $event } findet jetzt am { $date } um { $time } statt ({ $tz }).
sms-reminder = Erinnerung: { $event } beginnt am { $date } um { $time } ({ $tz }).

# Shared labels used across the cancel / decline / approve / reschedule / claim flows

common-detail-guest = Gast:
common-detail-reason = Grund:
common-reason-optional = (optional)
common-close-page = Du kannst diese Seite schließen.

# Cancel flow (booking_cancel_form.html, booking_cancelled_guest.html)

cancel-page-title = Buchung stornieren
cancel-heading = Buchung stornieren
cancel-subtitle = Du bist dabei, deine Buchung zu stornieren.
cancel-reason-label = Grund
cancel-reason-placeholder-host = Teile dem Gastgeber den Grund mit...
cancel-button = Buchung stornieren
cancelled-heading = Buchung storniert
cancelled-subtitle = Deine Buchung wurde storniert und der Gastgeber wurde benachrichtigt.

# Decline flow (booking_decline_form.html, booking_declined.html)

decline-page-title = Buchung ablehnen
decline-heading = Buchung ablehnen
decline-subtitle = Du bist dabei, diese Buchungsanfrage abzulehnen.
decline-reason-placeholder-guest = Teile dem Gast den Grund mit...
decline-button = Buchung ablehnen
declined-heading = Buchung abgelehnt
declined-subtitle = Die Buchung wurde abgelehnt und der Gast wurde benachrichtigt.

# Approve flow (booking_approve_form.html, booking_approved.html)

approve-page-title = Buchung genehmigen
approve-heading = Buchung genehmigen
approve-subtitle = Du bist dabei, diese Buchungsanfrage zu genehmigen.
approve-button = Buchung genehmigen
approved-heading = Buchung genehmigt
approved-subtitle = Die Buchung wurde bestätigt und eine Bestätigungs-E-Mail wurde an { $email } gesendet.

# Claim flow (booking_claim_form.html, booking_claimed.html, booking_already_claimed.html)

claim-page-title = Buchung übernehmen
claim-heading = Buchung übernehmen
claim-subtitle = Du bist dabei, diese Buchung zu übernehmen. Du wirst als Teilnehmer hinzugefügt.
claim-assigned-to = Zugewiesen an:
claim-button = Diese Buchung übernehmen
claimed-page-title = Buchung übernommen
claimed-heading = Buchung übernommen
claimed-subtitle = Du hast diese Buchung übernommen. Eine Kalendereinladung wurde an deine E-Mail-Adresse gesendet.
already-claimed-page-title = Bereits übernommen
already-claimed-heading = Bereits übernommen
already-claimed-subtitle = Diese Buchung wurde bereits von { $name } übernommen.

# Generic error page (booking_action_error.html)

action-error-page-title = Fehler bei der Buchungsaktion

# Host-initiated reschedule (booking_host_reschedule.html)

host-resched-page-title = Buchung verschieben — calrs
host-resched-heading = Buchung verschieben
host-resched-subtitle = { $guest } erhält eine E-Mail mit der Bitte, einen neuen Termin auszuwählen.
host-resched-currently = Aktuell:
host-resched-button = Verschiebungsanfrage senden
host-resched-cancel-link = Abbrechen

# Guest reschedule confirmation (booking_reschedule_confirm.html)

resched-confirm-page-title = Verschiebung bestätigen
resched-confirm-heading = Verschiebung bestätigen
resched-confirm-subtitle = Du bist dabei, deine Buchung auf einen neuen Termin zu verschieben.
resched-was = Vorher:
resched-new = Neu:
resched-button = Verschiebung bestätigen
resched-back-to-picker = Zurück zur Terminauswahl

# Base layout chrome (templates/base.html)

base-loader-checking = Verfügbarkeit wird geprüft
base-loader-please-wait = Bitte warte, die neuesten Kalenderdaten werden geladen...
base-stop-impersonating = Identitätswechsel beenden
base-theme-toggle = Design wechseln
base-language-choose = Wähle deine Sprache
base-powered-by = Läuft mit

# Profile (templates/profile.html)

profile-pick-event-type-invite = Wähle eine Terminart, um einen Termin zu buchen.
profile-no-event-type = Noch keine Terminarten verfügbar.

# Month and weekday names + per-locale date format patterns.
# Used by server-side date formatters in src/i18n.rs.

common-month-1 = Januar
common-month-2 = Februar
common-month-3 = März
common-month-4 = April
common-month-5 = Mai
common-month-6 = Juni
common-month-7 = Juli
common-month-8 = August
common-month-9 = September
common-month-10 = Oktober
common-month-11 = November
common-month-12 = Dezember

common-weekday-long-mon = Montag
common-weekday-long-tue = Dienstag
common-weekday-long-wed = Mittwoch
common-weekday-long-thu = Donnerstag
common-weekday-long-fri = Freitag
common-weekday-long-sat = Samstag
common-weekday-long-sun = Sonntag

# Format patterns are parametric per locale to handle word order. Translators
# pick where each placeholder lands. Example outputs:
#   EN: April 2026  /  Tuesday, March 12, 2026
#   FR: avril 2026  /  mardi 12 mars 2026
#   ES: abril 2026  /  martes, 12 de marzo de 2026
common-format-month-year = { $month } { $year }
common-format-long-date = { $weekday }, { $day }. { $month } { $year }
common-format-date = { $day }. { $month } { $year }

# Email signatures and shared bits (src/email.rs)

email-signature = — calrs
email-action-reschedule = Verschieben
email-action-cancel-booking = Buchung stornieren

# Email: guest booking confirmation

# Kept to "event — date": Exchange titles the guest appointment after the
# email Subject header, not the ICS SUMMARY (#157).
email-confirm-subject = { $event } — { $date }
email-confirm-greeting = Hallo { $name },
email-confirm-headline = Deine Buchung wurde bestätigt!
email-confirm-ics-attached-plain = Eine Kalendereinladung ist beigefügt.
email-confirm-ics-attached-html = Eine Kalendereinladung ist dieser E-Mail beigefügt.
email-confirm-need-to-cancel = Stornieren? { $url }

# Email: guest reminder

email-reminder-subject = Erinnerung: { $event } um { $time }
email-reminder-headline = Dein Termin steht bevor.

# Email: guest cancellation

email-cancel-subject = Storniert: { $event } — { $date }
email-cancel-headline-by-host = Deine Buchung wurde von { $host } storniert.
email-cancel-headline-by-guest = Deine Buchung wurde storniert.
email-cancel-ics-attached-plain = Eine Kalenderstornierung ist beigefügt.
email-cancel-ics-attached-html = Eine Kalenderstornierung ist dieser E-Mail beigefügt.

# Confirmation email: notice-window policy lines (src/email.rs)

email-confirm-cancel-notice = Hinweis: Für eine Stornierung sind mindestens { $minutes } Minuten Vorlaufzeit nötig.
email-confirm-reschedule-notice = Hinweis: Für eine Verschiebung sind mindestens { $minutes } Minuten Vorlaufzeit nötig.

# Event type form: cancel/reschedule minimum notice (templates/event_type_form.html)

# Google Meet (templates/event_type_form.html, src/google_meet.rs)
event-type-form-location-google-meet = Google Meet (automatisch erzeugter Link)
event-type-form-location-google-meet-hint = Bei der Bestätigung wird ein eigener Google-Meet-Link erstellt, im Namen des zugewiesenen Gastgebers. Jeder Gastgeber (du oder jedes berechtigte Teammitglied) muss Google Kalender verbunden und einen Schreibkalender ausgewählt haben.
google-meet-prereq-no-host = Google Meet setzt einen Gastgeber mit verbundenem Google Kalender voraus.
google-meet-prereq-no-eligible = Google Meet setzt mindestens ein berechtigtes Teammitglied mit verbundenem Google Kalender voraus.
google-meet-prereq-missing = Google Meet setzt voraus, dass jeder Gastgeber Google Kalender verbunden und einen Schreibkalender ausgewählt hat. Es fehlen noch: { $names }. Verbinde sie unter Dashboard → Kalenderquellen.
google-meet-unavailable-title = Google Meet ist nicht verfügbar
google-meet-dynamic-group-unavailable = Der Gastgeber braucht einen verbundenen Google Kalender mit ausgewähltem Schreibkalender.

event-type-form-cancel-notice-label = Mindestvorlaufzeit für Stornierungen
event-type-form-reschedule-notice-label = Mindestvorlaufzeit für Verschiebungen
event-type-form-notice-help = Leer lassen für keine Einschränkung.
event-type-form-resources-label = Erforderliche Ressourcen
event-type-form-resources-hint = Zeitfenster werden nur angeboten, wenn die ausgewählten Ressourcen gemäß dem Modus unten verfügbar sind.
event-type-form-resources-mode-all = Alle ausgewählten Ressourcen müssen frei sein
event-type-form-resources-mode-round-robin = Eine freie Ressource genügt (sie wird der Buchung zugewiesen)
event-type-form-notice-unit-minutes = Minuten
event-type-form-notice-unit-hours = Stunden
event-type-form-notice-unit-days = Tage
event-type-form-booking-horizon-label = Buchungshorizont
event-type-form-booking-horizon-help = Wie viele Tage im Voraus Gäste buchen können. Leer lassen für kein Limit, 0 für nur heute.
event-type-form-bookable-range-label = Buchbarer Zeitraum
event-type-form-bookable-from = von
event-type-form-bookable-until = bis
event-type-form-bookable-range-help = Nur an diesen Tagen kann gebucht werden. Leer lassen für kein Limit. Für andere Zeiten an einzelnen Tagen nutze Ausnahmen.
event-type-form-bookable-range-past-warning = Dieser Zeitraum ist bereits vorbei. Der Termintyp wird nicht mehr öffentlich angezeigt.
event-type-form-bookable-range-too-far-warning = Dieser Zeitraum beginnt in mehr als einem Jahr. Gäste können höchstens ein Jahr im Voraus buchen, die Buchung öffnet daher am { $date }.
event-type-form-bookable-range-horizon-warning = Der Buchungshorizont endet, bevor dieser Zeitraum beginnt. Gäste können ab { $date } buchen.
event-type-form-bookable-range-bookings-outside =
    { $count ->
        [one] 1 bestehende Buchung liegt außerhalb des buchbaren Zeitraums.
       *[other] { $count } bestehende Buchungen liegen außerhalb des buchbaren Zeitraums.
    }
event-type-form-bookable-range-bookings-link = Buchungen ansehen
event-type-form-saved = Gespeichert.

# Booking confirmation: cancel/reschedule policy notices (templates/confirmed.html)

confirmed-cancel-notice-info = Eine Stornierung erfordert mindestens { $minutes } Minuten Vorlaufzeit vor dem Termin.
confirmed-reschedule-notice-info = Eine Verschiebung erfordert mindestens { $minutes } Minuten Vorlaufzeit vor dem Termin.

# Booking action blocked page (templates/booking_action_blocked.html)

booking-blocked-title-cancel = Diese Buchung kann online nicht mehr storniert werden
booking-blocked-title-reschedule = Diese Buchung kann online nicht mehr verschoben werden
booking-blocked-body = Der Gastgeber verlangt mindestens { $minutes } Minuten Vorlaufzeit. Falls du nicht teilnehmen kannst, schreib bitte direkt an <a href="mailto:{ $host_email }">{ $host_email }</a>.

# Dashboard event types listing (templates/dashboard_event_types.html)

dashboard-event-types-copy = Kopieren
dashboard-event-types-copied = Kopiert!
dashboard-event-types-copy-title = Buchungslink kopieren
dashboard-event-types-copy-failed = Kopieren fehlgeschlagen

# Dashboard sidebar and shared chrome (templates/dashboard_base.html)

nav-section-scheduling = Terminplanung
nav-overview = Übersicht
nav-event-types = Terminarten
nav-bookings = Buchungen
nav-teams = Teams
nav-section-shared-links = Geteilte Links
nav-invite-links = Einladungslinks
nav-section-calendars = Kalender
nav-sources = Quellen
nav-section-personal = Persönlich
nav-settings = Profil & Einstellungen
nav-troubleshoot = Diagnose
nav-section-admin = Administration
nav-admin-panel = Administrationsbereich
nav-sign-out = Abmelden
nav-release-notes = Versionshinweise ansehen

# Timezone mismatch banner (templates/dashboard_base.html)

tz-banner-text = Die Zeitzone deines Browsers ist { $detected }, deine Buchungszeitzone ist jedoch auf { $current } eingestellt.
tz-banner-update = Aktualisieren
tz-banner-dismiss = Ausblenden

# Markdown editor toolbar (templates/dashboard_base.html)

editor-link-prompt = URL eingeben:
editor-link-default-label = Linktext
editor-placeholder-text = Text
editor-nothing-to-preview = Keine Vorschau verfügbar

# Dashboard overview (templates/dashboard_overview.html)

overview-page-title = Dashboard
overview-welcome = Willkommen, { $name }
overview-public-page = Öffentliche Seite:
overview-avail-banner-title = Standardverfügbarkeit
overview-avail-banner-body = Deine Standardarbeitszeiten wurden auf Mo–Fr, 9:00–17:00 Uhr gesetzt. Sie gelten, wenn andere dich in dynamische Gruppentermine einbeziehen.
overview-avail-banner-cta = Verfügbarkeit prüfen
overview-dismiss = Ausblenden
overview-getting-started = Erste Schritte
overview-getting-started-help = Folge diesen Schritten, um Buchungen anzunehmen.
overview-step-connect-calendar = Kalender verbinden
overview-step-first-event-type = Erste Terminart anlegen
overview-step-share-link = Buchungslink teilen
overview-pending-approval = Warten auf Bestätigung
overview-booking-with = { $title } mit { $guest }
overview-badge-pending = ausstehend
overview-guest-booked = Vom Gast gebucht:
overview-confirm = Bestätigen
overview-decline = Ablehnen
overview-stat-event-types = Terminarten
overview-stat-upcoming = Anstehende Buchungen
overview-stat-pending = Warten auf Bestätigung
overview-stat-sources = Kalenderquellen
overview-quick-actions = Neue Terminart anlegen
overview-action-public-title = Öffentliche Buchungsseite
overview-action-public-desc = Teile einen Link — jeder kann ein Zeitfenster wählen und Zeit mit dir buchen.
overview-action-team-title = Teamplanung
overview-action-team-desc = Verteile Buchungen auf Teammitglieder oder finde einen Termin, an dem alle frei sind.
overview-action-team-desc-empty = Lege zuerst ein Team an und richte dann gemeinsame Terminarten ein.
overview-action-private-title = Privat, nur auf Einladung
overview-action-private-desc = Erzeuge Einmal-Links für bestimmte Kontakte. Niemand sonst kann buchen.
overview-action-shared-title = Geteilte Einladungslinks
overview-action-shared-desc = Alle Kolleginnen und Kollegen im Team können Buchungslinks erzeugen und extern teilen.
overview-action-reason-calendar = Verbinde zuerst einen Kalender
overview-action-reason-ask-admin = Bitte die Administration, ein Team anzulegen
overview-action-reason-team-admin = Erfordert ein Team — lege zuerst eines an
overview-action-reason-team-member = Erfordert ein Team — frage die Administration

# Dashboard bookings (templates/dashboard_bookings.html)

bookings-page-title = Buchungen
bookings-pending-approval = Warten auf Bestätigung
bookings-available-to-claim = Zur Übernahme verfügbar
bookings-upcoming = Anstehende Buchungen
bookings-with = { $title } mit { $guest }
bookings-guest-booked = Vom Gast gebucht:
bookings-resource = Ressource:
bookings-confirm = Bestätigen
bookings-reschedule = Verschieben
bookings-decline = Ablehnen
bookings-claim = Übernehmen
bookings-badge-awaiting-reschedule = Verschiebung ausstehend
bookings-badge-writeback-failed = nicht in deinem Kalender
bookings-writeback-failed-detail = Schreiben in den Kalender fehlgeschlagen: { $error }
bookings-cancel = Stornieren
bookings-reason-placeholder = Grund (optional)
bookings-confirm-cancel = Stornierung bestätigen
bookings-back = Zurück
bookings-empty = Noch keine anstehenden Buchungen.<br>Teile deine { $link }, damit andere Zeit mit dir buchen können.
bookings-empty-link-label = Terminart-Links

# Dashboard teams listing (templates/dashboard_teams.html)

teams-page-title = Teams
teams-heading = Teams
teams-new = Neu
teams-badge-public = öffentlich
teams-badge-private = privat
teams-settings = Einstellungen
teams-view = Ansehen
teams-empty = Noch keine Teams.
teams-empty-admin = { $link }, um mit deinem Team zusammenzuarbeiten.
teams-empty-admin-link-label = Lege eines an
teams-empty-member = Teams werden von der Administration angelegt. Bitte sie, eines anzulegen und dich als Mitglied hinzuzufügen.

# Dashboard invite links (templates/dashboard_internal.html)

invite-links-page-title = Einladungslinks
invite-links-heading = Einladungslinks
invite-links-new = Neuer interner Termin
invite-links-help = Erzeuge Einmal-Buchungslinks für interne Terminarten. Alle angemeldeten Kolleginnen und Kollegen können hier Links erstellen und teilen.
invite-links-duration = { $minutes } Min.
invite-links-hosted-by = Gastgeber: { $host }
invite-links-get-link = Link erzeugen
invite-links-invites = Einladungen
invite-links-empty = Noch keine internen Terminarten.<br>{ $link } mit der Sichtbarkeit „Intern“, damit alle Kolleginnen und Kollegen Buchungslinks erzeugen können.
invite-links-empty-link-label = Lege eine Terminart an
invite-links-js-generating = Wird erzeugt...
invite-links-js-copied = Kopiert!
invite-links-js-error = Fehler

teams-member-count =
    { $count ->
        [one] { $count } Mitglied
       *[other] { $count } Mitglieder
    }

# Dashboard calendar sources (templates/dashboard_sources.html)

sources-page-title = Kalenderquellen
sources-heading = Kalenderquellen
sources-add = Hinzufügen
sources-last-sync = Letzte Synchronisierung:
sources-sync-failing = Synchronisierung schlägt seit { $since } UTC fehl: { $error }
sources-sync = Synchronisieren
sources-full-resync = Vollständige Neusynchronisierung
sources-full-resync-title = Cache leeren und alle Termine erneut vom Server laden
sources-test = Testen
sources-reconnect = Neu verbinden
sources-reconnect-title = Die Google-Zustimmung erneut durchlaufen
sources-edit = Bearbeiten
sources-remove = Entfernen
sources-remove-confirm = Quelle „{ $name }“ entfernen? Dabei werden alle von dieser Quelle synchronisierten Termine gelöscht.
sources-no-write-calendar = Kein Schreibkalender ausgewählt. Bestätigte Buchungen bleiben in calrs und werden nicht in diesen Kalender übertragen. Wähle unten einen aus, um das Rückschreiben zu aktivieren.
sources-write-bookings-to = Buchungen schreiben nach:
sources-write-none = Keiner (nicht schreiben)
sources-empty = Keine Kalenderquellen verbunden. { $link }, um die Verfügbarkeit zu prüfen.
sources-empty-link-label = Füge eine hinzu
sources-calendars = Kalender
sources-calendars-synced = Synchronisierte Kalender: { $synced } von { $total }
source-calendars-page-title = Zu synchronisierende Kalender
source-calendars-heading = Zu synchronisierende Kalender
source-calendars-back = Zurück zu den Kalenderquellen
source-calendars-help = Wähle, welche Kalender von { $name } calrs synchronisiert. Nicht ausgewählte Kalender werden nie abgerufen und blockieren deine Verfügbarkeit nicht.
source-calendars-select-all = Alle auswählen
source-calendars-select-none = Keine auswählen
source-calendars-write-badge = Buchungen werden hier eingetragen
source-calendars-write-hint = Bestätigte Buchungen werden in diesen Kalender geschrieben, ob er synchronisiert wird oder nicht.
source-calendars-save = Speichern
source-calendars-saved = Gespeichert. Neu ausgewählte Kalender werden bei der nächsten Synchronisierung abgerufen; mit „Synchronisieren“ auf der Seite der Quellen holst du sie sofort.
source-calendars-empty = Für diese Quelle wurden noch keine Kalender gefunden. Starte eine Synchronisierung, um sie zu finden.

# Dashboard event types listing (templates/dashboard_event_types.html)

event-types-page-title = Terminarten
event-types-heading = Terminarten
event-types-new = Neu
event-types-badge-disabled = deaktiviert
event-types-badge-internal = intern
event-types-badge-private = privat
event-types-badge-resources = Ressourcen
event-types-badge-active = aktiv
event-types-badge-expired = abgelaufen
event-types-badge-range-from = ab { $date }
event-types-badge-range-until = bis { $date }
event-types-send-invites = Einladungen senden
event-types-duration = { $minutes } Min.
event-types-mode-collective = kollektiv
event-types-mode-round-robin = Reihum
event-types-edit = Bearbeiten
event-types-disable = Deaktivieren
event-types-enable = Aktivieren
event-types-embed = Einbetten
event-types-overrides = Ausnahmen
event-types-team-settings = Teameinstellungen
event-types-invites = Einladungen
event-types-view-public = Öffentliche Seite ansehen
event-types-view-page = Seite ansehen
event-types-delete = Löschen
event-types-delete-confirm = Terminart „{ $title }“ löschen? Dies kann nicht rückgängig gemacht werden.
event-types-duplicate = Duplizieren
event-types-duplicate-title = { $title } (Kopie)
event-types-empty = Noch keine Terminarten. { $link }, um Buchungen anzunehmen.
event-types-empty-link-label = Lege eine an

# Markdown editor toolbar (templates/settings.html, templates/team_form.html)

editor-bold = Fett (Strg+B)
editor-italic = Kursiv (Strg+I)
editor-strikethrough = Durchgestrichen
editor-code = Inline-Code
editor-link = Link einfügen (Strg+K)
editor-toggle-preview = Vorschau ein-/ausblenden
editor-preview = Vorschau

# Profile and settings (templates/settings.html)

settings-page-title = Einstellungen
settings-heading = Profil & Einstellungen
settings-public-page-label = Deine öffentliche Buchungsseite
settings-copy = Kopieren
settings-copied = Kopiert!
settings-open = Öffnen
settings-avatar = Avatar
settings-upload = Hochladen
settings-remove = Entfernen
settings-display-name = Anzeigename
settings-display-name-placeholder = Dein Name
settings-username = Benutzername
settings-username-hint = (wird in deiner Buchungs-URL verwendet)
settings-username-pattern-title = Nur Kleinbuchstaben, Ziffern und Bindestriche
settings-username-help = Deine öffentliche Buchungsseite:
settings-title = Funktion
settings-title-placeholder = z. B. Softwareentwicklerin, Produktmanager
settings-title-help = Wird auf deinem öffentlichen Profil und in der Seitenleiste angezeigt.
settings-bio = Kurzprofil
settings-bio-placeholder = Erzähle ein wenig über dich...
settings-bio-help = Wird auf deiner öffentlichen Buchungsseite angezeigt. Unterstützt **fett**, *kursiv*, ~~durchgestrichen~~, `Code` und [Links](url).
settings-booking-email = Buchungs-E-Mail
settings-booking-email-help = Diese Adresse erscheint auf deinen öffentlichen Buchungsseiten und in E-Mail-Benachrichtigungen. Leer lassen, um deine Anmeldeadresse zu verwenden.
settings-booking-email-warning = Stelle sicher, dass diese Adresse bei deinem Mailanbieter existiert. Andernfalls werden Benachrichtigungen nicht zugestellt.
settings-timezone = Zeitzone
settings-timezone-help = Deine Verfügbarkeitsregeln und Buchungszeiten werden in dieser Zeitzone berechnet.
settings-language = Sprache
settings-language-auto = Automatisch (Browsereinstellung)
settings-language-help = Wähle eine Oberflächensprache oder belasse es bei „Automatisch“, um der Browsereinstellung zu folgen.
settings-public-page-language = Sprache der öffentlichen Buchungsseiten
settings-public-page-language-visitor = Sprache des jeweiligen Besuchers
settings-public-page-language-host = Immer meine Sprache
settings-public-page-language-help = Sprache deines öffentlichen Profils, deiner Buchungsseiten und der E-Mails an deine Gäste. „Immer meine Sprache“ verwendet die oben gewählte Sprache.
settings-error-public-page-language = Wähle eine gültige Sprache für die öffentlichen Buchungsseiten.
settings-dynamic-group = Anderen erlauben, mich in dynamische Gruppenlinks einzubeziehen
settings-dynamic-group-help = Wenn aktiviert, können andere Benutzer spontane Gruppentermin-URLs erstellen, die dich einschließen (z. B. { $example }).
settings-lend-resource = Meinen Kalenderzugang für Ressourcenreservierungen bereitstellen
settings-lend-resource-help = Wenn eine Buchung eine geteilte Ressource (Demolabor, Besprechungsraum) reservieren muss, in die dein Kalenderkonto schreiben darf, erlaube calrs, deine gespeicherten Kalenderzugangsdaten dafür zu verwenden.
settings-default-availability = Standardverfügbarkeit
settings-default-availability-help = Deine Standardarbeitszeiten. Werden für dynamische Gruppenlinks verwendet, wenn andere dich zu einem Termin hinzufügen.
settings-copy-to-all = Auf alle Tage übertragen
settings-copy-to-all-title = Die Zeitfenster des ersten aktivierten Tages auf alle anderen aktivierten Tage übertragen
settings-add-window = Zeitfenster hinzufügen
settings-remove-window = Zeitfenster entfernen
settings-save = Einstellungen speichern
settings-appearance = Darstellung
settings-theme-system = System
settings-theme-light = Hell
settings-theme-dark = Dunkel

# Sign in (templates/auth/login.html)

login-page-title = Anmelden
login-heading = Anmelden
login-subtitle = Melde dich bei deinem calrs-Konto an
login-sso = Mit SSO anmelden
login-or = oder
login-email = E-Mail
login-password = Passwort
login-submit = Mit E-Mail anmelden
login-no-account = Noch kein Konto? { $link }
login-register-link = Registrieren

# Registration (templates/auth/register.html)

register-page-title = Registrieren
register-heading = Konto erstellen
register-subtitle = Registriere ein neues calrs-Konto
register-domains-limited = Die Registrierung ist beschränkt auf: { $domains }
register-name = Name
register-name-placeholder = Dein Name
register-email = E-Mail
register-password = Passwort
register-password-hint = (mind. 12 Zeichen)
register-submit = Konto erstellen
register-have-account = Du hast bereits ein Konto? { $link }
register-signin-link = Anmelden

# Authentication errors (src/auth.rs)

auth-error-rate-limited = Zu viele Anmeldeversuche. Bitte versuche es später erneut.
auth-error-invalid-credentials = Ungültige E-Mail-Adresse oder ungültiges Passwort
auth-error-internal = Interner Fehler
auth-error-registration-disabled = Die Registrierung ist deaktiviert.
auth-error-name-length = Der Name muss zwischen 1 und 255 Zeichen lang sein
auth-error-email-length = Die E-Mail-Adresse muss zwischen 1 und 255 Zeichen lang sein
auth-error-email-invalid = Bitte gib eine gültige E-Mail-Adresse ein
auth-error-email-domain = E-Mail-Domain nicht zugelassen
auth-error-password-length = Das Passwort muss mindestens 12 Zeichen lang sein
auth-error-email-taken = Diese E-Mail-Adresse ist bereits registriert
auth-error-create-failed = Konto konnte nicht erstellt werden

# Calendar source test and write-back setup (templates/source_test.html, templates/source_write_setup.html)

source-test-page-title = Kalenderquelle
source-test-sync-heading = Synchronisierung: { $name }
source-test-heading = Verbindungstest
source-write-page-title = Kalender-Rückschreiben einrichten
source-write-back = Zurück zum Dashboard
source-write-heading = Wohin sollen Buchungen geschrieben werden?
source-write-help = Wenn jemand einen Termin mit dir bucht, kann calrs den Eintrag automatisch in deinem Kalender anlegen. Wähle, in welchen Kalender Buchungen für { $name } geschrieben werden.
source-write-save = Speichern
source-write-skip = Vorerst überspringen
source-write-sync-results = Ergebnisse der Synchronisierung

source-write-event-count =
    { $count ->
        [one] { $count } Termin
       *[other] { $count } Termine
    }

# Date overrides (templates/overrides.html)

overrides-page-title = Datumsausnahmen
overrides-heading = Datumsausnahmen
overrides-back-teams = Zurück zu den Teams
overrides-back-event-types = Zurück zu den Terminarten
overrides-intro = Datumsbezogene Ausnahmen für { $title } hinzufügen
overrides-add-heading = Neue Ausnahme hinzufügen
overrides-date = Datum
overrides-type = Art der Ausnahme
overrides-type-blocked = Ganzen Tag sperren
overrides-type-custom = Abweichende Zeiten
overrides-start-time = Startzeit
overrides-end-time = Endzeit
overrides-add-submit = Ausnahme hinzufügen
overrides-existing = Bestehende Ausnahmen
overrides-badge-blocked = gesperrt
overrides-badge-custom = abweichende Zeiten
overrides-delete = Löschen
overrides-delete-confirm = Diese Ausnahme löschen?
overrides-empty = Noch keine Datumsausnahmen.<br>Nutze das Formular oben, um bestimmte Tage zu sperren (Feiertage, freie Tage) oder abweichende Zeiten festzulegen.

# Public team page (templates/team_profile.html)

team-profile-subtitle = Wähle eine Terminart, um einen Termin zu buchen.
team-profile-empty = Noch keine Terminarten verfügbar.

# Availability troubleshoot (templates/troubleshoot.html, src/web/mod.rs)

troubleshoot-page-title = Diagnose
troubleshoot-empty = Keine Terminarten gefunden. { $link }, um mit der Verfügbarkeitsdiagnose zu beginnen.
troubleshoot-empty-link-label = Lege eine an
troubleshoot-subtitle = Sieh nach, warum Zeitfenster für { $title } verfügbar oder blockiert sind
troubleshoot-duration = { $minutes } Min.
troubleshoot-buffer-before = { $minutes } Min. Puffer davor
troubleshoot-buffer-after = { $minutes } Min. Puffer danach
troubleshoot-min-notice = { $minutes } Min. Vorlaufzeit
troubleshoot-blocked-override = Durch Datumsausnahme gesperrt (freier Tag)
troubleshoot-custom-hours-active = Ausnahme mit abweichenden Zeiten aktiv (ersetzt die Wochenregeln)
troubleshoot-legend-available = Verfügbar
troubleshoot-legend-calendar-event = Kalendertermin
troubleshoot-legend-booking = Buchung
troubleshoot-legend-resource = Ressource belegt
troubleshoot-legend-outside = Außerhalb der Zeiten
troubleshoot-legend-buffer = Puffer / Mindestvorlaufzeit
troubleshoot-blocked-slots = Blockierte Zeitfenster
troubleshoot-none-date-blocked = Dieses Datum ist durch eine Verfügbarkeitsausnahme gesperrt (freier Tag). Keine Zeitfenster verfügbar.
troubleshoot-none-custom-hours = Ausnahme mit abweichenden Zeiten aktiv, aber keine passenden Zeitfenster. Prüfe die Einstellungen der Ausnahme.
troubleshoot-none-no-rules = Keine Verfügbarkeitsregeln für diesen Wochentag. Diese Terminart ist am { $date } nicht buchbar.
troubleshoot-none-all-bookable = Keine blockierten Zeitfenster innerhalb der Verfügbarkeitszeiten. Alle Zeiten sind buchbar.
troubleshoot-label-outside = Außerhalb der Verfügbarkeit
troubleshoot-label-available = Verfügbar
troubleshoot-label-min-notice = Mindestvorlaufzeit ({ $minutes } Min.)
troubleshoot-label-beyond-horizon = Jenseits des Buchungshorizonts ({ $days } Tage)
troubleshoot-label-outside-date-range = Außerhalb des buchbaren Zeitraums
troubleshoot-label-buffer = Puffer ({ $minutes } Min.)
troubleshoot-label-resource-busy = Ressource belegt: { $names }
troubleshoot-detail-around = Rund um: { $label }
troubleshoot-detail-around-booking = Rund um die Buchung von { $guest }
troubleshoot-reason-calendar-event = Kalendertermin: { $label }
troubleshoot-reason-booking = Buchung: { $label }

# Invite management (templates/invite_form.html)

invites-heading = Einladungen
invites-back-teams = Zurück zu den Teams
invites-back-event-types = Zurück zu den Terminarten
invites-intro = Einladungslinks für { $title } versenden
invites-capped = <strong>Die Eingabe wurde auf { $max } Empfänger pro Absendung begrenzt.</strong> Sende den Rest in einem weiteren Durchgang.
invites-failed-hint = — Einzelheiten findest du in den Serverprotokollen.
invites-quick-link = Schnelllink
invites-quick-link-help = Erzeuge einen Einmal-Link und kopiere ihn in die Zwischenablage.
invites-get-link = Link erzeugen
invites-or-email = Oder per E-Mail senden
invites-recipients = Empfänger
invites-recipients-hint = (eine Adresse pro Zeile, höchstens { $max })
invites-message = Persönliche Nachricht
invites-message-hint = (optional, geht an alle Empfänger)
invites-message-placeholder = Ich freue mich darauf, dir eine Demo zu zeigen...
invites-expires-in = Läuft ab in
invites-expires-days = { $days } Tagen
invites-expires-never = Nie
invites-allow-multiple = Mehrere Buchungen pro Empfänger erlauben
invites-send = Einladungen senden
invites-sent-heading = Gesendete Einladungen
invites-badge-expired = abgelaufen
invites-badge-used = verwendet
invites-badge-active = aktiv
invites-sent-by = Gesendet von { $name }
invites-uses = { $used }/{ $max } Verwendungen
invites-expires-at = Läuft ab am { $date }
invites-copy-link = Link kopieren
invites-delete = Löschen
invites-delete-confirm = Diese Einladung löschen?
invites-empty = Noch keine Einladungen gesendet. Nutze das Formular oben, um jemandem einen Buchungslink zu schicken.
invites-js-generating = Wird erzeugt...
invites-js-copied = Kopiert!
invites-js-error = Fehler

invites-sent-count =
    { $count ->
        [one] { $count } Einladung gesendet.
       *[other] { $count } Einladungen gesendet.
    }

invites-skipped-invalid =
    { $count ->
        [one] { $count } ungültige Zeile übersprungen:
       *[other] { $count } ungültige Zeilen übersprungen:
    }

invites-skipped-duplicate =
    { $count ->
        [one] { $count } doppelte Zeile übersprungen:
       *[other] { $count } doppelte Zeilen übersprungen:
    }

invites-failed =
    { $count ->
        [one] { $count } Einladung fehlgeschlagen (DB oder SMTP):
       *[other] { $count } Einladungen fehlgeschlagen (DB oder SMTP):
    }

# Calendar source form (templates/source_form.html)

source-form-title-edit = Kalenderquelle bearbeiten
source-form-title-add = Kalender hinzufügen
source-form-heading-edit = Kalenderquelle bearbeiten
source-form-heading-add = Kalender verbinden
source-form-subtitle-edit = Aktualisiere die Verbindung. Lass das Passwort leer, um das bestehende beizubehalten. Führe nach einer Änderung von URL oder Benutzername eine Synchronisierung durch, um die Kalenderliste zu aktualisieren.
source-form-subtitle-add = Verbinde einen CalDAV-Server oder Microsoft Exchange (EWS), damit calrs bei Buchungen die Verfügbarkeit prüfen kann.
source-form-backend = Backend
source-form-preset = Voreinstellung
source-form-connect-google = Mit Google verbinden
source-form-google-unavailable = Google Kalender ist nicht verfügbar. Wende dich an deine Administration.
source-form-name = Anzeigename
source-form-name-placeholder = Mein Kalender
source-form-url-caldav = CalDAV-URL
source-form-url-ews = EWS-Endpunkt-URL
source-form-username = Benutzername
source-form-password = Passwort
source-form-password-keep = Leer lassen, um das bestehende beizubehalten
source-form-password-placeholder = App-Passwort oder Kontopasswort
source-form-skip-test = Verbindungstest überspringen
source-form-skip-test-help = Nutze dies, wenn der Test hängt (kommt bei manchen BlueMind-/Zimbra-Installationen vor). Du kannst die Verbindung später testen.
source-form-save = Änderungen speichern
source-form-add = Kalenderquelle hinzufügen
source-form-help-google-configured = Klicke auf die Schaltfläche unten, um calrs den Zugriff auf deinen Google Kalender zu erlauben.
source-form-help-google-unconfigured = Die Google-Kalender-Integration ist noch nicht eingerichtet. Bitte deine Administration, im Administrationsbereich OAuth2-Zugangsdaten für Google zu hinterlegen.

# Calendar source form: provider help (templates/source_form.html)

source-form-help-bluemind = <strong>BlueMind</strong> — Verwende den DAV-Endpunkt deines BlueMind-Servers.<br> Üblicherweise: <code>https://mail.yourcompany.com/dav/</code><br> Der Benutzername ist deine <strong>E-Mail-Adresse</strong> (z. B. <code>alice@yourcompany.com</code>), nicht nur der Anmeldename.<br> Wenn der Verbindungstest hängt, aktiviere „Verbindungstest überspringen“ und synchronisiere direkt.
source-form-help-nextcloud = <strong>Nextcloud</strong> — Verwende die WebDAV-Wurzel, nicht die URL eines einzelnen Kalenders.<br> Üblicherweise: <code>https://cloud.example.com/remote.php/dav</code>
source-form-help-fastmail = <strong>Fastmail</strong> — Verwende deine vollständige Adresse im URL-Pfad.<br> Beispiel: <code>https://caldav.fastmail.com/dav/calendars/user/you@fastmail.com/</code><br> Verwende ein App-Passwort (Settings &rarr; Privacy &amp; Security &rarr; Integrations).
source-form-help-icloud = <strong>iCloud</strong> — Verwende <code>https://caldav.icloud.com/</code><br> Du benötigst ein App-Passwort von <a href="https://appleid.apple.com" target="_blank" style="color: var(--accent);">appleid.apple.com</a> (Sicherheit &rarr; App-spezifische Passwörter).
source-form-help-zimbra = <strong>Zimbra</strong> — Verwende den DAV-Endpunkt deines Zimbra-Servers.<br> Üblicherweise: <code>https://mail.example.com/dav/</code>
source-form-help-sogo = <strong>SOGo</strong> — Verwende den DAV-Endpunkt von SOGo.<br> Üblicherweise: <code>https://mail.example.com/SOGo/dav/</code>
source-form-help-radicale = <strong>Radicale</strong> — Verwende die Wurzel-URL des Servers.<br> Üblicherweise: <code>https://cal.example.com/</code>
source-form-help-baikal = <strong>Baikal</strong>: Verwende den DAV-Endpunkt deines Baikal-Servers.<br> Üblicherweise: <code>https://cal.example.com/dav.php/</code>
source-form-help-exchange = <strong>Microsoft Exchange (EWS)</strong>. Verwende den SOAP-Endpunkt:<br> <code>https://mail.example.com/EWS/Exchange.asmx</code><br> Der Benutzername ist die Postfachadresse; das Passwort muss HTTP Basic über TLS zulassen (bei deaktiviertem Basic im Tenant an einem Dienstpostfach aktivieren).<br> Wähle außerdem oben im Backend-Menü <strong>Microsoft Exchange (EWS)</strong>.
source-form-help-google = <strong>Google Kalender</strong>: Verbindung über OAuth2. Kein Passwort nötig.<br>
source-form-help-other = Gib die <strong>DAV-Wurzel-URL</strong> deines CalDAV-Servers an — nicht die eines einzelnen Kalenders oder einen öffentlichen Link.<br> calrs findet deine Kalender automatisch per PROPFIND (RFC 4791).

# Markdown editor toolbar, short labels (templates/team_form.html, templates/team_settings.html)

editor-bold-short = Fett
editor-italic-short = Kursiv
editor-link-short = Link einfügen

# Team creation (templates/team_form.html)

team-form-heading = Neues Team
team-form-name = Teamname
team-form-name-placeholder = Entwicklung
team-form-slug = Kürzel
team-form-slug-hint = (URL-tauglicher Bezeichner)
team-form-slug-pattern-title = Nur Kleinbuchstaben, Ziffern und Bindestriche
team-form-description = Beschreibung
team-form-optional = (optional)
team-form-description-placeholder = Worum es in diesem Team geht...
team-form-description-help = Wird auf der Teamseite angezeigt. Unterstützt **fett**, *kursiv* und [Links](url).
team-form-visibility = Sichtbarkeit
team-form-public = Öffentlich
team-form-private = Privat
team-form-visibility-help = Private Teams erhalten ein Einladungstoken zum Teilen. Öffentliche Teams erscheinen auf der Teamprofilseite.
team-form-members = Mitglieder
team-form-members-help = Du wirst automatisch als Teamadministrator hinzugefügt. Füge einzelne Benutzer hinzu oder verknüpfe OIDC-Gruppen.
team-form-search-placeholder = Benutzer oder Gruppen suchen...
team-form-search-users = Benutzer
team-form-search-groups = OIDC-Gruppen
team-form-you = (du)
team-form-submit = Team erstellen

# Team settings (templates/team_settings.html)

team-settings-page-title = Einstellungen
team-settings-subtitle = Teameinstellungen — Teamadministratoren können sie bearbeiten.
team-settings-public-url = Öffentliche URL
team-settings-public-url-help = Über diesen Link kann jeder buchen.
team-settings-invite-link = Einladungslink
team-settings-invite-link-help = Teile diesen Link, um Zugang zur Buchungsseite dieses privaten Teams zu geben.
team-settings-avatar = Team-Avatar
team-settings-profile = Profil
team-settings-description-placeholder = Beschreibe dieses Team...
team-settings-description-help = Wird auf der öffentlichen Buchungsseite des Teams angezeigt. Unterstützt **fett**, *kursiv* und [Links](url).
team-settings-visibility-help = Öffentliche Teams erscheinen auf der Teamprofilseite. Für private Teams ist ein Einladungslink erforderlich.
team-settings-members-help = Verwalte die Mitglieder dieses Teams. Füge einzelne Benutzer hinzu oder verknüpfe OIDC-Gruppen für die automatische Synchronisierung.
team-settings-role-member = Mitglied
team-settings-role-admin = Administrator
team-settings-oidc-group = OIDC-Gruppe
team-settings-remove = Entfernen
team-settings-save = Änderungen speichern
team-settings-danger-zone = Gefahrenbereich
team-settings-danger-help = Dieses Team endgültig löschen. Terminarten werden nur getrennt, nicht gelöscht. Dies kann nicht rückgängig gemacht werden.
team-settings-delete = Dieses Team löschen
team-settings-delete-confirm = Team „{ $name }“ löschen? Dies kann nicht rückgängig gemacht werden.

# Event type form (templates/event_type_form.html)

etf-heading-edit = Terminart bearbeiten
etf-heading-new = Neue Terminart
etf-disabled-banner = Diese Terminart ist deaktiviert, daher kann sie noch niemand buchen.
etf-team = Team
etf-team-hint = (optional — leer lassen für eine persönliche Terminart)
etf-team-personal = Persönlich
etf-scheduling-mode = Planungsmodus
etf-mode-round-robin = Reihum — einem verfügbaren Mitglied zuweisen
etf-mode-collective = Kollektiv — alle Mitglieder müssen verfügbar sein
etf-scheduling-mode-help = „Reihum“ weist die Buchung einem verfügbaren Mitglied zu (dem am wenigsten ausgelasteten zuerst). „Kollektiv“ verlangt, dass alle Mitglieder gleichzeitig frei sind.
etf-title = Titel
etf-title-placeholder = 30-minütiges Kennenlerngespräch
etf-slug = Kürzel
etf-slug-placeholder = wird aus dem Titel erzeugt
etf-description-placeholder = Ein kurzes Kennenlerngespräch, um zu besprechen...
etf-description-help = Wird auf der Buchungsseite angezeigt. Unterstützt **fett**, *kursiv* und [Links](url).
etf-location = Ort
etf-location-link = Videoanruf (feste URL)
etf-location-jitsi = Jitsi (automatisch erzeugter Raum)
etf-location-webhook = Webhook (eigener Anbieter)
etf-location-phone = Telefon
etf-location-in-person = Vor Ort
etf-location-custom = Benutzerdefiniert
etf-location-details = Details
etf-location-details-placeholder = https://meet.example.com/mein-raum
etf-pattern-placeholder = Leer lassen, um das Standardmuster der Organisation zu verwenden
etf-duration = Dauer (Minuten)
etf-slot-interval = Zeitfensterabstand (Minuten)
etf-slot-interval-placeholder = Wie die Dauer
etf-slot-interval-help = Wie oft Zeitfenster beginnen. Leer lassen, um der Dauer zu folgen.
etf-required-members = Erforderliche Mitglieder
etf-required-members-help = Alle angehakten Mitglieder müssen frei sein, damit ein Zeitfenster angeboten wird. Nimm den Haken bei Mitgliedern weg, die ausgeschlossen werden sollen (ihre Verfügbarkeit wird dann ignoriert).
etf-member-priority = Mitgliederpriorität
etf-member-priority-help = Mitglieder mit höherer Priorität erhalten Buchungen zuerst, sofern verfügbar. Bei gleicher Priorität entscheidet die Zahl der jüngsten Buchungen.
etf-member-timezone-title = Zeitzone des Mitglieds. Seine persönlichen Arbeitszeiten werden in dieser Zeitzone ausgelegt.
etf-priority-high = Hoch
etf-priority-medium = Mittel
etf-priority-low = Niedrig
etf-section-availability = Verfügbarkeit
etf-timezone-help = Die Zeiten unten werden in dieser Zeitzone ausgelegt. Wähle bei Team-Terminarten die Arbeitszeitzone des Teams (nicht zwingend die der erstellenden Person).
etf-reset-default = Auf meine Standardwerte zurücksetzen
etf-reset-default-title = Diese Zeiten durch die Standardverfügbarkeit aus deinem Profil ersetzen
etf-availability-prefilled = Vorbelegt aus deiner { $link }. Du kannst sie hier für diese Terminart überschreiben.
etf-availability-prefilled-link = Standardverfügbarkeit
etf-section-buffers = Puffer & Vorlaufzeit
etf-buffer-before = Puffer davor (Min.)
etf-buffer-after = Puffer danach (Min.)
etf-min-notice = Mindestvorlaufzeit
etf-min-notice-help = Wie lange im Voraus gebucht werden muss.
etf-section-limits = Buchungslimits
etf-first-slot-only = Ein Zeitfenster pro Tag
etf-first-slot-only-help = Nur die früheste verfügbare Zeit je Tag anzeigen.
etf-freq-limit = Buchungshäufigkeit begrenzen
etf-freq-limit-help = Begrenzen, wie oft dieser Termin pro Zeitraum gebucht werden kann.
etf-add-limit = Limit hinzufügen
etf-section-options = Buchungsoptionen
etf-requires-confirmation = Bestätigung erforderlich
etf-requires-confirmation-help = Buchungen bleiben ausstehend, bis du sie im Dashboard bestätigst.
etf-sms = SMS-Benachrichtigungen
etf-sms-off = Aus, keine Telefonnummer abfragen
etf-sms-optional = Optional, Gäste können eine Nummer angeben
etf-sms-required = Pflicht, Gäste müssen eine Nummer angeben
etf-sms-help = Schickt dem Gast zusätzlich zur E-Mail eine SMS, wenn seine Buchung bestätigt, verschoben oder storniert wird oder kurz bevorsteht. Gäste, die das Feld leer lassen, erhalten einfach keine SMS. Erfordert ein SMS-Gateway im { $link }.
etf-admin-panel-link = Administrationsbereich
etf-additional-guests = Weitere Teilnehmende
etf-guests-none = Gäste können niemanden hinzufügen
etf-additional-guests-help = Der buchenden Person erlauben, weitere Teilnehmende einzuladen, die die Kalendereinladung erhalten.
etf-default-view = Standard-Kalenderansicht
etf-view-month = Monat — Kalenderraster mit Zeitfensterliste
etf-view-week = Woche — Spalten für 7 Tage mit Zeitfenstern
etf-view-column = Spalte — Tage mit direkt aufgeführten Zeitfenstern
etf-view-week-short = Wochen-
etf-view-column-short = Spalten-
etf-default-view-help = Die Ansicht, die Gäste zuerst sehen. Gäste können jederzeit wechseln.
etf-conflict-calendars = Kalender für Konfliktprüfung
etf-conflict-calendars-help = Wähle, welche Kalender auf Konflikte geprüft werden. Ohne Auswahl werden alle Kalender verwendet.
etf-no-resources = Noch keine geteilten Ressourcen eingerichtet. Lege im { $link } eine an (Demolabor, Besprechungsraum), um sie hier zu verlangen.
etf-section-access = Zugriff & Benachrichtigungen
etf-visibility-public = Öffentlich — auf deinem Profil sichtbar
etf-visibility-internal = Intern — alle Kolleginnen und Kollegen können Einladungslinks erzeugen
etf-visibility-private = Privat — nur per Einladungslink
etf-visibility-help = Legt fest, wer diese Terminart sehen und buchen kann.
etf-vis-internal = Intern
etf-reminder = Buchungserinnerung
etf-reminder-none = Keine Erinnerung
etf-reminder-help = Vor dem Termin eine Erinnerungs-E-Mail an dich und deinen Gast senden.
etf-dynamic-group = Dynamischer Gruppenlink
etf-dynamic-group-help = Erstelle einen spontanen Terminlink, der die Verfügbarkeit von dir und anderen Benutzern prüft.
etf-dynamic-group-search = Nach einem Benutzer zum Hinzufügen suchen...
etf-dynamic-group-note = Es werden nur Benutzer angezeigt, die dynamische Gruppenlinks erlauben.
etf-dynamic-group-url = URL des Gruppenlinks
etf-watcher-teams = Beobachtende Teams
etf-watcher-teams-help = Ausgewählte Teams werden bei neuen Buchungen benachrichtigt. Mitglieder können eine Buchung übernehmen, um daran teilzunehmen.
etf-save = Änderungen speichern
etf-create = Terminart erstellen
etf-js-loading = Wird geladen...
etf-js-no-default = Kein Standard gesetzt
etf-js-reset-done = Zurückgesetzt!
etf-js-error = Fehler
etf-js-remove-limit = Limit entfernen
etf-period-day = Pro Tag
etf-period-week = Pro Woche
etf-period-month = Pro Monat
etf-period-year = Pro Jahr

# Event type form: runtime summary hints (templates/event_type_form.html)

# %1 and %2 are substituted client-side; the values are only known once a field is edited.

etf-hint-no-days = Keine Tage festgelegt
etf-hint-every-day = Täglich
etf-fmt-day-one = %1 Tag
etf-fmt-day-other = %1 Tage
etf-fmt-hours = %1 Std.
etf-fmt-minutes = %1 Min.
etf-hint-buffer-both = %1 Min. davor, %2 Min. danach
etf-hint-buffer-before = %1 Min. Puffer davor
etf-hint-buffer-after = %1 Min. Puffer danach
etf-hint-notice = %1 Vorlaufzeit
etf-hint-no-buffers = Keine Puffer, jederzeit buchbar
etf-hint-max = Max. %1
etf-hint-period-day = /Tag
etf-hint-period-week = /Woche
etf-hint-period-month = /Monat
etf-hint-period-year = /Jahr
etf-hint-no-limits = Keine Limits
etf-hint-confirmation-required = Bestätigung erforderlich
etf-hint-auto-confirmed = Automatisch bestätigt
etf-hint-extra-guests-one = bis zu %1 weitere Person
etf-hint-extra-guests-other = bis zu %1 weitere Personen
etf-hint-view = %1Ansicht
etf-hint-reminder = Erinnerung %1 vorher
etf-hint-no-reminder = keine Erinnerung

etf-guests-up-to =
    { $count ->
        [one] Bis zu { $count } weitere Person
       *[other] Bis zu { $count } weitere Personen
    }

etf-reminder-hours =
    { $count ->
        [one] { $count } Stunde vorher
       *[other] { $count } Stunden vorher
    }

etf-reminder-days =
    { $count ->
        [one] { $count } Tag vorher
       *[other] { $count } Tage vorher
    }

# Event type form: preset banners and meeting-pattern help (templates/event_type_form.html)
# Literal braces are escaped as {"{"} because Fluent reads a bare { as a placeable.

etf-preset-public = Du erstellst eine <strong>öffentliche</strong> Terminart &mdash; jeder mit dem Link kann buchen.
etf-preset-private = Du erstellst eine <strong>private</strong> Terminart &mdash; nur eingeladene Personen können buchen.
etf-preset-internal = Du erstellst eine <strong>interne</strong> Terminart &mdash; alle Kolleginnen und Kollegen können den Buchungslink teilen.
etf-preset-team = Du erstellst eine <strong>Team-Terminart</strong> &mdash; Buchungen werden auf die Teammitglieder verteilt.
etf-pattern-hint = Optionales eigenes Muster. Platzhalter: <code>{"{"}username{"}"}</code>, <code>{"{"}event{"}"}</code>, <code>{"{"}date{"}"}</code>, <code>{"{"}random{"}"}</code>. Leer lassen, um den von der Administration gesetzten Organisationsstandard zu verwenden.
etf-pattern-random-warning = Dieses Muster enthält keinen <code>{"{"}random{"}"}</code>-Platzhalter. Zwei Buchungen dieser Terminart am selben Tag teilen sich denselben Raum, und der zweite Gast kann in das Gespräch des ersten geraten. Verwende feste Räume nur, wenn du genau das möchtest.
etf-webhook-hint = Die Termin-URL je Buchung wird von dem Webhook geholt, den die Administration unter Administration &rarr; Termin-Webhook eingerichtet hat. Hier ist keine URL nötig.

# Admin panel (templates/admin.html)

admin-page-title = Administration
admin-heading = Administrations-Dashboard
admin-action-refused = Aktion abgelehnt:
admin-error-last-admin = Das ist der letzte aktive Admin. Stufe zuerst einen anderen Benutzer hoch oder aktiviere einen anderen Admin.
admin-logo = Firmenlogo
admin-logo-help = Wird auf öffentlichen Buchungsseiten angezeigt. PNG, JPEG, GIF oder WebP, max. 2 MB.
admin-logo-refused = Das Logo wurde nicht gespeichert. Lade ein PNG-, JPEG-, GIF- oder WebP-Bild mit höchstens 2 MB hoch.
admin-favicon = Favicon
admin-favicon-help = Erscheint im Browser-Tab und in Lesezeichen. Empfohlen: ein quadratisches PNG ab 32×32 Pixel, max. 512 KB. Ohne eigenes Favicon erscheint die calrs-Krabbe.
admin-favicon-refused = Das Favicon wurde nicht gespeichert. Lade ein PNG-, JPEG-, GIF- oder WebP-Bild mit höchstens 512 KB hoch.
admin-upload-save-failed = Die Datei konnte nicht im Datenverzeichnis des Servers gespeichert werden. Details stehen im Server-Log.

# User and team avatar uploads (#307)
avatar-upload-refused = Das Bild wurde nicht gespeichert. Lade ein PNG-, JPEG-, GIF- oder WebP-Bild mit höchstens 2 MB hoch.
avatar-upload-save-failed = Das Bild konnte auf dem Server nicht gespeichert werden. Versuche es später erneut oder bitte deinen Administrator, das Server-Log zu prüfen.
admin-company-link = Firmenlink
admin-company-link-help = Das Logo verlinkt auf öffentlichen Buchungsseiten auf diese URL. Leer lassen für keinen Link.
admin-theme = Design
admin-theme-help = Wähle ein Farbdesign für alle Seiten. Die Umschaltung zwischen Hell und Dunkel ist davon unabhängig — die Designs passen sich beiden Modi an.
admin-theme-default = Standard
admin-theme-default-desc = Klares Blau
admin-theme-nord-desc = Arktischer Frost
admin-theme-dracula-desc = Dunkles Violett
admin-theme-gruvbox-desc = Warmes Retro
admin-theme-solarized-desc = Ethans Klassiker
admin-theme-tokyo-desc = Neon-Stadtbild
admin-theme-custom = Benutzerdefiniert
admin-theme-custom-desc = Deine Farben
admin-custom-colors = Eigene Farben
admin-color-accent = Akzent
admin-color-accent-hover = Akzent beim Überfahren
admin-color-bg = Hintergrund
admin-color-surface = Oberfläche
admin-color-text = Text
admin-save-theme = Design speichern
admin-users = Benutzer ({ $count })
admin-user-filter = Nach Name oder E-Mail filtern…
admin-badge-admin = Administrator
admin-badge-disabled = deaktiviert
admin-impersonate = Identität annehmen
admin-demote = Herabstufen
admin-promote = Hochstufen
admin-disable = Deaktivieren
admin-enable = Aktivieren
admin-delete = Löschen
admin-no-users-match = Keine Benutzer entsprechen dem Filter.
admin-no-users = Noch keine Benutzer.
admin-groups = Gruppen ({ $count })
admin-group-filter = Nach Gruppenname filtern…
admin-group-name = Gruppenname
admin-weight = Gewicht:
admin-no-groups-match = Keine Gruppen entsprechen dem Filter.
admin-no-groups = Noch keine Gruppen synchronisiert. Gruppen werden automatisch von deinem OIDC-Anbieter übernommen.
admin-auth-settings = Anmeldeeinstellungen
admin-registration-enabled = Registrierung aktiviert
admin-allowed-domains = Zugelassene E-Mail-Domains
admin-allowed-domains-hint = (kommagetrennt, leer lassen für alle)
admin-save-auth = Anmeldeeinstellungen speichern
admin-system-settings = Systemeinstellungen
admin-base-url = Basis-URL
admin-base-url-help = Öffentliche URL dieser Instanz. Wird für OIDC-Weiterleitungen und Links in E-Mails verwendet (Bestätigen/Ablehnen, Stornieren, Erinnerungen).
admin-private-hosts = Positivliste privater Hosts
admin-private-hosts-help = Kommagetrennte Hostnamen, die für CalDAV-/EWS-Quellen auf private oder reservierte IP-Adressen zeigen dürfen (Ausnahme vom SSRF-Schutz). Trage nur Hosts ein, die du kontrollierst (etwa einen Kalenderserver im selben Docker-Netz). Leer lassen, um den Schutz für alle Hosts aktiv zu halten.
admin-german-formal = Anrede auf Deutsch
admin-german-formal-du = Informell (du)
admin-german-formal-sie = Formell (Sie)
admin-german-formal-help = Wie Seiten und E-Mails Besucher ansprechen, deren Browser Deutsch anfordert. Wer unter Profil & Einstellungen eine Sprache gewählt hat, behält diese.
admin-unset-env = Entferne die Umgebungsvariable, um dies hier zu bearbeiten.
admin-save-system = Systemeinstellungen speichern
admin-status = Status:
admin-status-enabled = aktiviert
admin-status-disabled = deaktiviert
admin-status-disabled-paren = (deaktiviert)
admin-status-configured = konfiguriert
admin-status-not-configured = nicht konfiguriert
admin-via-environment = (über die Umgebung)
admin-issuer = Aussteller:
admin-client-id = Client-ID:
admin-instance = Instanz:
admin-oidc-settings = OIDC-Einstellungen
admin-oidc-enabled = OIDC aktiviert
admin-issuer-url = Aussteller-URL
admin-client-id-label = Client-ID
admin-client-secret = Client-Secret
admin-keep-current-hint = (leer lassen, um den aktuellen Wert zu behalten)
admin-keep-current-set-hint = (leer lassen, um den aktuellen Wert zu behalten — derzeit gesetzt)
admin-keep-unchanged = Leer lassen, um nichts zu ändern
admin-oidc-auto-register = Neue Benutzer aus OIDC automatisch registrieren
admin-save-oidc = OIDC-Einstellungen speichern
admin-google = Google Kalender (OAuth2)
admin-save-google = Google-OAuth2-Einstellungen speichern
admin-captcha = Captcha
admin-instance-url = Instanz-URL
admin-site-key = Site-Key
admin-secret = Secret
admin-widget-url = URL des Widget-Skripts
admin-widget-url-help = Überschreiben, falls das CDN blockiert ist. Änderungen wirken sofort nach dem Speichern.
admin-wasm-url = URL der Widget-WASM-Datei
admin-wasm-url-help = Von dort lädt das Widget seinen Löser. Trage die WASM-Datei deines Cap-Servers ein, damit Buchungsseiten cdn.jsdelivr.net nicht mehr kontaktieren. Leer lassen für den Standardwert.
admin-captcha-wasm-url-invalid = Die URL der Widget-WASM-Datei muss eine absolute http(s)-URL sein.
admin-captcha-disable-help = Lass Instanz-URL, Site-Key und Secret leer, um das Captcha auf Buchungsseiten zu deaktivieren.
admin-save-captcha = Captcha-Einstellungen speichern
admin-resources = Ressourcen
admin-resources-help = Geteilte buchbare Ressourcen (Demolabor, Besprechungsräume) auf Basis eines Kalender-Feeds. An Terminarten gebunden blockiert eine belegte Ressource Buchungen.
admin-resource-stats = Termine im Cache: { $events } &middot; Gebunden an { $attached } Terminart(en)
admin-never = nie
admin-resource-sync-failed = (letzter Versuch fehlgeschlagen: { $error })
admin-writeback-enabled = Rückschreiben: aktiviert ({ $via })
admin-writeback-readonly = Rückschreiben: nur Lesen
admin-teams-allowed = Zugelassene Teams:
admin-teams-allowed-none = keine (nur globale Administratoren)
admin-sync-now = Jetzt synchronisieren
admin-test-write = Schreiben testen
admin-delete-resource-confirm = Diese Ressource löschen? Terminarten, die sie nutzen, prüfen sie dann nicht mehr.
admin-name = Name
admin-name-help = Leer lassen, um den Namen aus dem Feed zu übernehmen.
admin-feed-url = URL des ICS-Feeds (Veröffentlichungsadresse)
admin-feed-url-help = BlueMind: die öffentliche oder private Kalenderadresse des Ressourcenkalenders.
admin-caldav-url = URL der CalDAV-Sammlung (für das Rückschreiben)
admin-caldav-url-help = Optional. Bei BlueMind wird sie automatisch aus der Feed-URL abgeleitet.
admin-caldav-username = CalDAV-Benutzername
admin-caldav-password = CalDAV-Passwort
admin-resource-teams = Teams, die diese Ressource nutzen dürfen
admin-resource-teams-help = Teamadministratoren dieser Teams können die Ressource an ihre Team-Terminarten binden. Leer: nur globale Administratoren.
admin-no-teams = Noch keine Teams.
admin-save-resource = Ressource speichern
admin-add-resource = Ressource hinzufügen
admin-jitsi = Jitsi (automatisch erzeugte Terminlinks)
admin-jitsi-help = Wenn der Ort einer Terminart auf „Jitsi (automatisch erzeugter Raum)“ gesetzt ist, baut calrs für jede Buchung eine frische Raum-URL, indem es das Muster unten an deine Jitsi-Basis-URL anhängt. Ein externer API-Aufruf ist nicht nötig.
admin-display-name = Anzeigename
admin-jitsi-display-name-placeholder = z. B. Meet DYB
admin-jitsi-display-name-help = Wird Gästen in der Zeitfensterauswahl und im Buchungsformular angezeigt. Standard ist „Videoanruf“, wenn leer.
admin-room-pattern = Muster für Raumnamen
admin-jitsi-disable-help = Lass die Basis-URL leer, um die automatische Jitsi-Erzeugung zu deaktivieren.
admin-save-jitsi = Jitsi-Einstellungen speichern
admin-meeting-webhook = Termin-Webhook (eigener Anbieter)
admin-webhook-url = Webhook-URL
admin-webhook-display-name-placeholder = z. B. Zoom, Whereby, Custom Meet
admin-webhook-display-name-help = Wird Gästen statt des allgemeinen Hinweises „Videoanruf“ angezeigt.
admin-authentication = Authentifizierung
admin-auth-none = Keine
admin-auth-hmac = HMAC-SHA256 (Header X-Calrs-Signature)
admin-shared-secret = Gemeinsames Secret
admin-webhook-disable-help = Lass die URL leer, um den Termin-Webhook zu deaktivieren.
admin-save-webhook = Webhook-Einstellungen speichern
admin-smtp = SMTP-Einstellungen
admin-smtp-test-sent = Test-E-Mail gesendet.
admin-smtp-test-failed = Die Test-E-Mail konnte nicht gesendet werden. Prüfe die Serverprotokolle und deine SMTP-Einstellungen.
admin-smtp-env-error = Fehler in der SMTP-Konfiguration aus der Umgebung:
admin-smtp-host = Host:
admin-smtp-from = Absender:
admin-smtp-enabled = SMTP aktiviert
admin-host = Host
admin-port = Port
admin-tls-mode = TLS-Modus
admin-tls-starttls = STARTTLS (Port 587)
admin-tls-implicit = Implizites TLS (Port 465)
admin-tls-none = Keines, unverschlüsselt (nur lokaler MTA)
admin-smtp-username-hint = (leer lassen für ein Relay ohne Authentifizierung)
admin-from-email = Absenderadresse
admin-from-name = Absendername
admin-save-smtp = SMTP-Einstellungen speichern
admin-send-test-email = Test-E-Mail senden an
admin-send-test-email-hint = (standardmäßig deine Kontoadresse)
admin-send-test-email-btn = Test-E-Mail senden
admin-smtp-clear-confirm = Die in der Datenbank gespeicherte SMTP-Konfiguration löschen?
admin-clear-db-config = Datenbankkonfiguration löschen
admin-sms = SMS-Einstellungen
admin-sms-help = Optional. SMS werden nur für Buchungen von Terminarten gesendet, bei denen „SMS-Benachrichtigungen“ aktiviert ist, und nur wenn der Gast eine Telefonnummer hinterlassen hat.
admin-sms-test-sent = Testnachricht gesendet.
admin-sms-test-checked = Zugangsdaten akzeptiert.
admin-sms-test-error = Das SMS-Gateway hat die Anfrage abgelehnt.
admin-sms-captcha-warning = Das Buchungsformular ist öffentlich und die Empfängernummer kommt vom Gast. SMS ohne Captcha sind daher ein offenes Relay, das jemand anderes auf deine Kosten nutzen kann. Richte oben das Captcha ein und beschränke die Zielländer in den Einstellungen deines Gateways.
admin-sms-sent-today = Heute gesendet:
admin-sms-of-cap = von { $cap }
admin-sms-config-error = Fehler in der SMS-Konfiguration:
admin-sms-gateway = Gateway:
admin-sms-account = Konto:
admin-sms-sender = Absender:
admin-sms-enabled = SMS aktiviert
admin-sms-gateway-label = Gateway
admin-required-on-switch = Beim Wechsel des Gateways erforderlich
admin-sms-docs = API-Dokumentation von { $provider }
admin-sms-country = Standard-Ländervorwahl
admin-sms-country-hint = (wird verwendet, wenn Gäste eine lokale Telefonnummer eingeben)
admin-sms-daily-cap = Tageslimit
admin-sms-daily-cap-hint = (Nachrichten pro Tag für die gesamte Instanz, 0 für kein Limit)
admin-sms-daily-cap-help = Jenseits des Limits verschickt calrs keine SMS mehr und sendet weiter E-Mails, damit Buchungen nie am erschöpften SMS-Budget scheitern.
admin-save-sms = SMS-Einstellungen speichern
admin-send-test-sms = Testnachricht senden an
admin-send-test-sms-hint-check = (leer lassen, um nur die Zugangsdaten zu prüfen)
admin-send-test-sms-hint-e164 = (Format E.164)
admin-test-gateway = Gateway testen
admin-sms-clear-confirm = Die in der Datenbank gespeicherte SMS-Konfiguration löschen?
admin-sms-allow-all = Allen Benutzern erlauben, SMS für ihre Terminarten zu aktivieren
admin-sms-allow-all-help = Standardmäßig aus: SMS verbrauchen Guthaben des hier konfigurierten Kontos, daher dürfen nur Administratoren eine Terminart in einen SMS-Modus schalten.
admin-save-policy = Richtlinie speichern
admin-page-of = Seite %1 von %2
admin-show-more-js = %1 weitere anzeigen
admin-show-fewer = Weniger anzeigen

# Admin panel: strings carrying markup or literal braces (templates/admin.html)

admin-delete-user-confirm = Benutzer { $email } endgültig löschen?{"\u000A"}{"\u000A"}Damit werden sein Benutzerkonto, sein Planungsprofil, seine Kalenderquellen, seine Terminarten und alle Daten gelöscht, die ausschließlich ihm gehören. Vergangene Buchungen werden zusammen mit seinen Terminarten gelöscht.{"\u000A"}{"\u000A"}Bei OIDC-/SSO-Benutzern gilt: Ist die automatische Registrierung aktiv, wird diese Person bei der nächsten Anmeldung neu angelegt.{"\u000A"}{"\u000A"}Dies kann nicht rückgängig gemacht werden.
admin-system-settings-help = Öffentliche URL und Netzwerksicherheitseinstellungen. Sie lassen sich auch über die Umgebungsvariablen <code>CALRS_BASE_URL</code> und <code>CALRS_ALLOW_PRIVATE_HOSTS</code> setzen. Ist eine Umgebungsvariable gesetzt, hat sie <strong>Vorrang</strong> vor dem Wert unten.
admin-set-by-env = — durch die Umgebung gesetzt ({ $var }), überschreibt den gespeicherten Wert
admin-google-help = Um die Google-Kalender-Integration zu aktivieren, erstelle OAuth2-Zugangsdaten in der <a href="https://console.cloud.google.com/apis/credentials" target="_blank" style="color: var(--accent);">Google Cloud Console</a>. Aktiviere die <strong>CalDAV API</strong> (Lesen belegter Zeiten und Zurückschreiben von Buchungen) und die <strong>Google Calendar API</strong> (Google-Meet-Links) und trage dann { $redirect_uri } als zugelassene Weiterleitungs-URI ein.
admin-room-pattern-help = Verfügbare Platzhalter: <code>{"{"}username{"}"}</code> (Gastgeber), <code>{"{"}event{"}"}</code> (Kürzel der Terminart), <code>{"{"}date{"}"}</code> (JJJJMMTT), <code>{"{"}random{"}"}</code> (8 Zeichen). Standard: { $default }.
admin-room-pattern-warning = Ohne <code>{"{"}random{"}"}</code> ist der Raumname vorhersehbar: Zwei Gäste, die dieselbe Terminart am selben Tag buchen, landen im selben Raum und sehen das Gespräch des jeweils anderen. Feste Räume sind zulässig (etwa ein persönlicher Raum je Gastgeber), aktiviere das aber nur, wenn du den Kompromiss verstehst.
admin-meeting-webhook-help = Wenn der Ort einer Terminart auf „Webhook (eigener Anbieter)“ gesetzt ist, sendet calrs die Buchungsdaten bei der Bestätigung per POST an diese URL und erwartet als Antwort einen JSON-Body <code>{"{"}"url": "https://..."{"}"}</code>.
admin-auth-hmac-help = Mit HMAC sendet calrs <code>X-Calrs-Signature: sha256=&lt;hex&gt;</code> über den unveränderten Anfragetext.
admin-tls-none-warning = Wähle <strong>Keines</strong> nur für ein Relay auf diesem Rechner, das kein STARTTLS anbietet oder dessen Zertifikat selbstsigniert ist. E-Mails und etwaige Zugangsdaten gehen dann unverschlüsselt über die Leitung.
admin-smtp-env-error-help = Korrigiere die Umgebungsvariablen <code>CALRS_SMTP_*</code> oder entferne sie, um SMTP hier über die Datenbank zu verwalten.
admin-smtp-env-managed = Verwaltet über <strong>Umgebungsvariablen</strong> (haben Vorrang vor der Datenbank). Ändere die Variablen <code>CALRS_SMTP_*</code> oder entferne sie, um SMTP hier zu verwalten.
admin-smtp-env-help = Alternativ über Umgebungsvariablen konfigurieren (sie haben Vorrang): <code>CALRS_SMTP_HOST</code>, <code>CALRS_SMTP_PORT</code>, <code>CALRS_SMTP_TLS_MODE</code> (<code>starttls</code>, <code>tls</code> oder <code>none</code>), <code>CALRS_SMTP_USERNAME</code>, <code>CALRS_SMTP_PASSWORD</code>, <code>CALRS_SMTP_FROM_EMAIL</code>, <code>CALRS_SMTP_FROM_NAME</code>. Nur <code>CALRS_SMTP_HOST</code> und <code>CALRS_SMTP_FROM_EMAIL</code> sind erforderlich; lass Benutzername und Passwort weg, um ohne Authentifizierung über einen lokalen MTA zu relayen.
admin-sms-env-error-help = Korrigiere die Umgebungsvariablen <code>CALRS_SMS_*</code> oder entferne sie, um SMS hier über die Datenbank zu verwalten.
admin-sms-env-managed = Verwaltet über <strong>Umgebungsvariablen</strong> (haben Vorrang vor der Datenbank). Ändere die Variablen <code>CALRS_SMS_*</code> oder entferne sie, um SMS hier zu verwalten.
admin-sms-env-help = Alternativ über Umgebungsvariablen konfigurieren (sie haben Vorrang): <code>CALRS_SMS_PROVIDER</code>, <code>CALRS_SMS_API_KEY</code>, <code>CALRS_SMS_API_SECRET</code>, <code>CALRS_SMS_SENDER</code>, <code>CALRS_SMS_BASE_URL</code>, <code>CALRS_SMS_DAILY_CAP</code>, <code>CALRS_SMS_DEFAULT_COUNTRY_CODE</code>.
admin-sms-trial-warning = <strong>Der Twilio-Testmodus ist aktiv</strong> (<code>CALRS_SMS_TWILIO_TRIAL</code>). Gäste erhalten Twilios vordefinierte Vorlage <code>sms_appointment_reminders</code> statt der echten Nachricht, und nur in deiner Twilio-Konsole verifizierte Nummern werden erreicht. Das ist eine Testhilfe für Testkonten. Entferne die Variable, bevor du Buchungen annimmst.

admin-show-more =
    { $count ->
        [one] { $count } weitere anzeigen
       *[other] { $count } weitere anzeigen
    }

# Calendar source form: backend picker (templates/source_form.html)

source-form-backend-help = Wähle das Protokoll, das dein Server spricht. EWS richtet sich an lokal betriebene Exchange-Server 2019/2016/2013.

admin-sms-going-live = <strong>Vor dem Produktivbetrieb:</strong> Beschränke die Zielländer in deinem Gateway (bei Twilio heißt das Geo Permissions), halte das Konto auf Guthabenbasis ohne automatische Aufladung, und lass das Captcha aktiv. Diese drei Maßnahmen begrenzen zusammen, was ein SMS-Pumping-Versuch kosten kann.

troubleshoot-heading = Verfügbarkeitsdiagnose

# Host-side form validation errors (src/web/mod.rs)

form-error-team-name-slug-required = Name und Kürzel sind erforderlich.
form-error-team-name-length = Der Name darf höchstens 255 Zeichen lang sein.
form-error-team-description-length = Die Beschreibung darf höchstens 5000 Zeichen lang sein.
form-error-slug-charset = Das Kürzel darf nur Kleinbuchstaben, Ziffern und Bindestriche enthalten.
form-error-slug-reserved = Dieses Kürzel ist reserviert. Bitte wähle ein anderes.
form-error-team-slug-taken = Ein Team mit diesem Kürzel existiert bereits.
form-error-title-required = Für die Erzeugung eines Kürzels ist ein Titel erforderlich.
form-error-event-type-slug-taken = Eine Terminart mit diesem Kürzel existiert bereits.
form-error-event-type-slug-taken-team = In diesem Team existiert bereits eine Terminart mit diesem Kürzel.
form-error-event-type-title-required = Ein Titel ist erforderlich.
form-error-event-type-title-length = Der Titel darf höchstens 255 Zeichen lang sein.
form-error-event-type-duration = Die Dauer muss eine ganze Zahl von Minuten zwischen 1 und 1440 (24 Stunden) sein.
form-error-event-type-number = Puffer, Mindestvorlaufzeit, Erinnerung und weitere Teilnehmende müssen ganze Zahlen ab 0 sein.
form-error-event-type-horizon = Der Buchungshorizont muss eine ganze Zahl von Tagen ab 0 sein. Lass das Feld leer für keine Begrenzung.
form-error-location-required = Angaben zum Ort sind erforderlich (etwa ein Videoanruf-Link, eine Telefonnummer oder eine Adresse).
form-error-bookable-range-order = Das Enddatum muss am oder nach dem Startdatum liegen.
form-error-bookable-range-invalid = Gib die Daten im Format JJJJ-MM-TT ein.
form-error-not-team-admin = Du bist kein Teamadministrator dieses Teams.
form-error-no-account = Kein Planungsprofil gefunden. Bitte wende dich an die Administration.
form-error-all-fields-required = Alle Felder sind erforderlich.
form-error-encryption = Verschlüsselungsfehler.
form-error-connection-failed = Verbindung fehlgeschlagen: { $error }. Prüfe URL und Zugangsdaten, oder aktiviere „Verbindungstest überspringen“, um trotzdem zu speichern.

# Settings page flash (src/web/mod.rs)

settings-saved = Einstellungen gespeichert.

# Profile settings validation and flash messages (src/web/mod.rs)

settings-error-name-length = Der Name muss zwischen 1 und 255 Zeichen lang sein.
settings-error-username-length = Der Benutzername muss mindestens 2 Zeichen lang sein.
settings-error-username-taken = Dieser Benutzername ist bereits vergeben.
settings-error-booking-email = Bitte gib eine gültige Buchungs-E-Mail-Adresse ein.
settings-error-save-failed = Die Einstellungen konnten nicht gespeichert werden.

# Host-facing error responses (src/web/mod.rs)

error-team-not-found-or-not-admin = Team nicht gefunden, oder du bist kein Teamadministrator.
error-team-not-found = Team nicht gefunden.
error-event-type-not-found = Terminart nicht gefunden.
error-decrypt-failed = Die gespeicherten Zugangsdaten konnten nicht entschlüsselt werden.
error-source-not-found = Quelle nicht gefunden.
error-source-no-password = Für diese Quelle ist kein Passwort gespeichert.
error-oauth-invalid-state = Ungültiger State-Parameter. Bitte versuche es erneut.
error-oauth-no-code = Kein Autorisierungscode empfangen.
error-oauth-not-configured = Google OAuth2 ist nicht konfiguriert.
error-no-scheduling-account = Kein Planungsprofil gefunden.
error-private-event-type-not-found = Private Terminart nicht gefunden.
error-access-denied = Zugriff verweigert.

# Guest booking-flow errors (src/web/mod.rs)

error-slot-unavailable = Dieses Zeitfenster ist nicht mehr verfügbar.
error-slot-too-soon = Dieses Zeitfenster ist nicht mehr verfügbar (zu kurzfristig).
error-slot-beyond-horizon = Dieses Zeitfenster liegt außerhalb des Buchungszeitraums.
error-slot-not-yet-bookable = Dieser Termin ist noch nicht buchbar.
error-invite-required = Diese Terminart erfordert einen Einladungslink.
error-invite-invalid = Ungültiger Einladungslink.
error-invite-expired = Dieser Einladungslink ist abgelaufen.
error-invite-used = Dieser Einladungslink wurde bereits verwendet.
error-invalid-date = Ungültiges Datum.
error-invalid-time = Ungültige Uhrzeit.
error-invalid-date-format = Ungültiges Datumsformat.
error-invalid-time-format = Ungültiges Uhrzeitformat.
error-too-many-bookings = Zu viele Buchungsversuche. Bitte versuche es in ein paar Minuten erneut.
error-too-many-requests = Zu viele Anfragen. Bitte versuche es später erneut.
error-no-members-available = Für dieses Zeitfenster ist kein Teammitglied verfügbar.
error-dynamic-group-public-only = Dynamische Gruppenlinks gibt es nur für öffentliche Terminarten.
error-user-not-found = Benutzer nicht gefunden.

# Booking action error page: titles (templates/booking_action_error.html)

bae-title-captcha = Captcha-Prüfung fehlgeschlagen
bae-title-invalid-booking = Ungültige Buchungsangaben
bae-title-unavailable = Derzeit nicht verfügbar
bae-title-cannot-approve = Diese Buchung kann nicht bestätigt werden
bae-title-invalid-link = Ungültiger Link
bae-title-invalid-or-expired = Ungültiger oder abgelaufener Link
bae-title-booking-not-found = Buchung nicht gefunden
bae-title-already-approved = Bereits bestätigt
bae-title-already-declined = Bereits abgelehnt
bae-title-already-cancelled = Bereits storniert
bae-title-booking-cancelled = Buchung storniert
bae-title-booking-declined = Buchung abgelehnt

# Booking action error page: bodies

bae-body-go-back = Bitte geh zurück und versuche es erneut.
bae-body-unavailable = Der Gastgeber nimmt für dieses Datum keine weiteren Buchungen an. Bitte wähle ein anderes Datum oder schau später noch einmal vorbei.
bae-body-resource-gone = Eine erforderliche Ressource ist zu dieser Zeit nicht mehr verfügbar. Bitte den Gast, einen anderen Termin zu wählen.
bae-body-no-claim-token = Kein Übernahme-Token angegeben.
bae-body-claim-invalid = Dieser Übernahme-Link ist nicht mehr gültig.
bae-body-booking-gone = Diese Buchung existiert nicht mehr.
bae-body-decline-link-invalid = Dieser Ablehnungslink ist ungültig, abgelaufen, oder die Buchung wurde bereits bearbeitet.
bae-body-cancel-link-invalid = Dieser Stornierungslink ist ungültig, abgelaufen, oder die Buchung wurde bereits storniert.
bae-body-cancel-link-invalid-short = Dieser Stornierungslink ist ungültig oder abgelaufen.
bae-body-reschedule-link-invalid = Dieser Verschiebungslink ist ungültig, abgelaufen, oder die Buchung wurde bereits bearbeitet.
bae-body-approval-link-invalid = Dieser Bestätigungslink ist ungültig oder abgelaufen.
bae-body-already-approved = Diese Buchung wurde bereits bestätigt.
bae-body-already-declined = Diese Buchung wurde bereits abgelehnt.
bae-body-already-cancelled = Diese Buchung wurde bereits storniert.
bae-body-was-cancelled = Diese Buchung wurde storniert.
bae-body-declined-by-host = Diese Buchung wurde vom Gastgeber abgelehnt.

# Booking form validation (src/web/mod.rs)

validate-name-length = Der Name muss zwischen 1 und 255 Zeichen lang sein.
validate-email-length = Die E-Mail-Adresse muss zwischen 1 und 255 Zeichen lang sein.
validate-email-invalid = Bitte gib eine gültige E-Mail-Adresse ein.
validate-notes-length = Die Notizen dürfen höchstens 5000 Zeichen lang sein.
validate-date-too-far = Es kann nicht mehr als ein Jahr im Voraus gebucht werden.

# Additional guests and dynamic group links (src/web/mod.rs)

guests-not-allowed = Zusätzliche Gäste sind für diese Terminart nicht erlaubt.
guests-too-many =
    { $max ->
        [one] Du kannst höchstens einen zusätzlichen Gast hinzufügen.
       *[other] Du kannst höchstens { $max } zusätzliche Gäste hinzufügen.
    }
guests-invalid-email = Ungültige E-Mail-Adresse eines zusätzlichen Gastes: { $email }
dynamic-group-min-usernames = Dynamische Gruppenlinks brauchen mindestens zwei Benutzernamen.
dynamic-group-user-not-found = Benutzer „{ $username }“ nicht gefunden.
dynamic-group-user-opted-out = Benutzer „{ $username }“ hat dynamische Gruppenlinks nicht aktiviert.

error-slot-unavailable-member = Dieses Zeitfenster ist nicht mehr verfügbar ({ $username } hat einen Konflikt).

# Pending requests: slot hold, expiry and approval re-check (#234)

sms-expired = Anfrage abgelaufen: { $event }, { $date } um { $time } ({ $tz }). Keine Antwort erhalten, bitte wähle einen anderen Termin.
email-pending-expired-status = Anfrage abgelaufen
email-pending-expired-headline = Deine Buchungsanfrage ist abgelaufen: Der Gastgeber hat nicht rechtzeitig geantwortet. Bitte wähle einen anderen Termin.
email-pending-expired-action = Anderen Termin wählen
bae-body-host-busy = Der Gastgeber (oder ein benötigtes Teammitglied) ist zu dieser Zeit nicht mehr frei. Bitte den Gast, einen anderen Termin zu wählen.
bookings-error-generic = Etwas ist schiefgelaufen.
bookings-error-no-bookable-dates = Dieser Termintyp hat keine buchbaren Tage mehr, daher könnte der Gast keine neue Zeit wählen. Verlängere zuerst seinen Buchungszeitraum.
troubleshoot-detail-pending-request = { $title } (offene Anfrage)
etf-pending-blocks-slot = Offene Anfragen blockieren den Termin
etf-pending-blocks-slot-help = Solange eine Anfrage auf deine Antwort wartet, kann niemand sonst diese Zeit buchen, weder bei dieser noch bei deinen anderen Terminarten.
etf-pending-expiry = Ablauf von Anfragen
etf-pending-expiry-help = Eine Anfrage, die du bis dahin nicht beantwortet hast, wird automatisch abgelehnt und der Gast wird gebeten, einen anderen Termin zu wählen. Sie endet früher, wenn der Termin zu nah ist, als dass jemand anderes ihn noch buchen könnte. Zur Halbzeit bekommst du eine Erinnerung.
etf-pending-expiry-never = Nie

etf-pending-expiry-hours =
    { $hours ->
        [one] Nach { $hours } Stunde
       *[other] Nach { $hours } Stunden
    }

# Host, watcher and remaining guest emails (src/email.rs, #209)

email-footer-sent-by = Gesendet von
email-sender-via = { $host } über { $instance }
email-date-at-time = { $date } um { $time }
email-detail-phone = Telefon:
email-detail-expires = Läuft ab:
email-detail-new-date = Neues Datum:
email-detail-new-time = Neue Uhrzeit:
email-detail-previous = Bisher:
email-detail-requested = Angefragt:
email-detail-originally = Ursprünglich:
email-detail-organizer = Organisator:
email-detail-booked-by = Gebucht von:
email-detail-invited-by = Eingeladen von:
email-detail-message = Nachricht:
email-action-approve = Genehmigen
email-action-decline = Ablehnen
email-action-pick-new-time = Neuen Termin wählen
email-host-approve-line = Genehmigen: { $url }
email-host-decline-line = Ablehnen: { $url }
email-host-approval-dashboard = Melde dich in deinem Dashboard an, um diese Buchung zu bestätigen oder abzulehnen.
email-host-new-subject = Neue Buchung: { $event }, { $guest } ({ $date })
email-host-new-heading = Neue Buchung!
email-host-new-message = { $guest } hat einen Termin bei dir gebucht.
email-host-confirmed-subject = Bestätigt: { $event }, { $guest } ({ $date })
email-host-confirmed-heading = Buchung bestätigt
email-host-confirmed-message = Du hast die Buchung mit { $guest } genehmigt.
email-host-confirmed-footer = Der Termin wurde deinem Kalender hinzugefügt.
email-host-sync-failure-subject = Aktion nötig: Kalender für { $event } nicht aktualisiert ({ $date })
email-host-sync-failure-heading = Dein Kalender wurde nicht aktualisiert
email-host-sync-failure-intro = Diese Buchung wurde verschoben, aber calrs konnte den Termin in deinem Google Kalender nicht aktualisieren. Der Gast hat die neue Uhrzeit, dein Kalender zeigt noch die alte.
email-host-sync-failure-action = Öffne den Termin in Google Kalender und verschiebe ihn auf die oben genannte Uhrzeit. Verschiebe den vorhandenen Termin, statt ihn neu anzulegen, sonst funktioniert der Google-Meet-Link nicht mehr, den der Gast bereits hat.
email-writeback-failure-subject = Handlungsbedarf: { $event } ({ $date }) fehlt in deinem Kalender
email-writeback-failure-heading = Eine Buchung fehlt in deinem Kalender
email-writeback-failure-intro = calrs konnte diese Buchung auch nach mehreren Versuchen nicht in deinen Kalender eintragen. Die Buchung ist bestätigt und der Gast hat die Einladung.
email-writeback-failure-action = Trage den Termin selbst in deinen Kalender ein, damit du ihn nicht verpasst, und prüfe die Kalenderquelle in calrs. calrs versucht es noch einige Stunden weiter; du bekommst diese E-Mail für diese Buchung nicht noch einmal.
email-detail-error = Fehler:
email-source-failure-subject = Handlungsbedarf: Kalenderquelle „{ $source }“ synchronisiert nicht
email-source-failure-heading = Eine Kalenderquelle synchronisiert nicht mehr
email-source-failure-intro = calrs kann diese Kalenderquelle seit über einer Stunde nicht synchronisieren. Deine Buchungsseiten bieten weiter Termine an, die aber auf der letzten erfolgreichen Synchronisierung beruhen: Seitdem hinzugefügte oder verschobene Termine werden nicht berücksichtigt.
email-source-failure-action = Prüfe die Quelle auf der Seite Kalenderquellen: Meist ist ein abgelaufenes Passwort oder ein entzogener Zugriff die Ursache. Von dieser Quelle hörst du erst wieder, wenn sie sich erholt und danach erneut ausfällt.
email-detail-source = Kalenderquelle:
email-detail-failing-since = Fehlerhaft seit:
email-host-reminder-subject = Erinnerung: { $event }, { $guest } ({ $date })
email-host-reminder-heading = Anstehende Buchung
email-host-reminder-message = Erinnerung: Du hast bald eine Buchung mit { $guest }.
email-host-cancel-subject = Storniert: { $event }, { $guest } ({ $date })
email-host-cancel-message-by-host = Du hast diese Buchung storniert.
email-host-cancel-message-by-guest = { $guest } hat die Buchung storniert.
email-host-approval-subject = Aktion erforderlich: { $event }, { $guest } ({ $date })
email-host-approval-heading = Aktion erforderlich
email-host-approval-message = { $guest } möchte einen Termin bei dir buchen.
email-host-approval-reminder-subject = Erinnerung: { $guest } wartet auf deine Antwort ({ $event }, { $date })
email-host-approval-reminder-heading = Erinnerung: Aktion erforderlich
email-host-approval-reminder-message = { $guest } wartet noch auf deine Antwort. Wenn du nicht antwortest, wird die Anfrage am { $deadline } automatisch abgelehnt.
email-host-approval-footer = Du kannst dies auch in deinem Dashboard verwalten.
email-host-expired-subject = Abgelaufen: { $event } mit { $guest } ({ $date })
email-host-expired-heading = Buchungsanfrage abgelaufen
email-host-expired-message = Niemand hat die Anfrage von { $guest } vor Ablauf der Frist beantwortet. Sie wurde daher automatisch abgelehnt, und der Gast wurde gebeten, einen anderen Termin zu wählen.
email-host-expired-footer = Wie lange Anfragen warten, kannst du in den Einstellungen des Termintyps ändern.
email-host-reschedule-request-subject = Verschiebungsanfrage: { $event }, { $guest } ({ $email })
email-host-reschedule-request-message = { $guest } möchte die Buchung verschieben.
email-watcher-claim-subject = Zum Übernehmen: { $event }, { $guest } ({ $date })
email-watcher-claim-message = Eine neue Buchung kann übernommen werden. Klicke unten, um als Teilnehmer dazuzukommen.
email-watcher-claim-link = Diese Buchung übernehmen: { $url }
email-watcher-claim-footer = Du kannst sie auch in deinem Dashboard übernehmen.
email-claim-confirmed-subject = Buchung übernommen: { $event }, { $guest } ({ $date })
email-claim-confirmed-footer = Du wirst als Teilnehmer zu diesem Termin hinzugefügt.
email-attendee-greeting = Hallo,
email-attendee-message = Du wurdest als Teilnehmer zu einer Buchung hinzugefügt.
email-attendee-rescheduled-message = Eine Buchung, an der du teilnimmst, wurde auf einen neuen Termin verschoben.
email-attendee-cancelled-message = Eine Buchung, an der du teilnimmst, wurde storniert.
email-attendee-cancelled-subject = Storniert: { $event }
email-original-booking-cancelled = Deine ursprüngliche Buchung ist ebenfalls storniert.
email-pick-new-time-subject = Verschiebung: { $event }, bitte wähle einen neuen Termin
email-pick-new-time-message = { $host } muss deine Buchung verschieben. Bitte wähle einen neuen Termin.
email-pick-new-time-link = Neuen Termin wählen: { $url }
email-rescheduled-message = Deine Buchung wurde von { $host } verschoben.
email-reschedule-approved-message = { $host } hat den neuen Termin deiner Buchung bestätigt.
email-rescheduled-ics-attached-plain = Eine aktualisierte Kalendereinladung ist beigefügt.
email-rescheduled-ics-attached-html = Eine aktualisierte Kalendereinladung ist dieser E-Mail beigefügt.
email-need-to-reschedule = Verschieben? { $url }
email-invite-subject = { $host } hat dich eingeladen, einen Termin zu buchen: { $event }
email-invite-message = { $host } hat dich eingeladen, einen Termin zu buchen: { $event }
email-invite-link = Klicke auf den Link unten, um einen Termin zu wählen:
email-invite-action = Termin wählen
email-cancel-reason-calendar-deleted = Der Kalendertermin wurde vom Gastgeber gelöscht.

# Embed snippet generator (templates/embed.html)
embed-page-title = Einbetten — { $title }
embed-heading = Einbetten: { $title }
embed-subtitle = Erzeuge ein Code-Snippet, um diese Terminart auf deiner eigenen Website einzubetten.
embed-warn-internal = Achtung: Diese Terminart ist <strong>intern</strong>. Besucher der eingebetteten Seite brauchen einen gültigen Einladungslink, um zu buchen — das erzeugte Snippet enthält keinen. Stell die Sichtbarkeit auf öffentlich, wenn du direkt einbetten willst.
embed-warn-private = Achtung: Diese Terminart ist <strong>privat</strong>. Besucher der eingebetteten Seite brauchen einen gültigen Einladungslink, um zu buchen — das erzeugte Snippet enthält keinen. Stell die Sichtbarkeit auf öffentlich, wenn du direkt einbetten willst.
embed-warn-private-team = Achtung: Dieses Team ist <strong>privat</strong>. Besucher der eingebetteten Seite können ohne den Einladungslink des Teams nicht buchen — das erzeugte Snippet enthält keinen. Stell das Team auf öffentlich, wenn du direkt einbetten willst.
embed-need-username = Du musst einen <strong>Benutzernamen</strong> festlegen, bevor du diese Terminart einbetten kannst — der Einbettungslink lautet { $path }. { $link } und komm dann zurück.
embed-need-username-link = Leg in den Einstellungen einen Benutzernamen fest
embed-username-placeholder = benutzername
embed-tab-inline = Inline
embed-tab-floating = Schwebender Button
embed-tab-element = Klick auf Element
embed-layout = Layout
embed-theme = Design
embed-theme-auto = Automatisch (wie beim Besucher)
embed-brand-color = Markenfarbe
embed-use-org-color = Farbe der Organisation verwenden
embed-button-text = Button-Text
embed-button-default = Termin buchen
embed-button-position = Button-Position
embed-pos-bottom-right = Unten rechts
embed-pos-bottom-left = Unten links
embed-pos-top-right = Oben rechts
embed-pos-top-left = Oben links
embed-button-colors = Button-Farben
embed-button-fg-placeholder = Text
embed-button-colors-hint = Hintergrund / Text
embed-show-icon = Kalendersymbol anzeigen
embed-element-help = Füge das Snippet einmal in deine Website ein und ergänze dann die angezeigten data-Attribute an jedem klickbaren Element (Button, Link, div), das das Buchungsfenster öffnen soll.
embed-snippet = Snippet
embed-loading = Wird geladen…
embed-preview = Live-Vorschau
embed-preview-back = zurück zur Vorschau
embed-preview-floating-hint = Klick auf den Button, um das Buchungsfenster zu simulieren
embed-preview-element-hint = Demo-Auslöser — füge auf deiner Website die data-Attribute aus dem Snippet einem beliebigen Element hinzu. Klick auf einen davon, um das Fenster auszuprobieren:
embed-preview-text-link = oder klick auf diesen Textlink
embed-snippet-attr-comment = Füge diese Attribute einem beliebigen klickbaren Element deiner Seite hinzu:

# Leftovers in partly translated pages (host reschedule, impersonation banner, slots, team settings)
host-resched-when = { $date } um { $start } – { $end }
base-impersonating = Du handelst als { $name }
slots-rescheduling-banner = Verschieben: { $title } (derzeit { $date } um { $time })
slots-invite-required-title = Für die Buchung ist ein Einladungslink nötig
team-settings-slug-url = Wird in der Team-URL verwendet: { $url }
team-event-type-no-teams = Es gibt noch keine Teams.
team-event-type-not-team-admin = Du bist in keinem Team Admin.

# Google Calendar connection errors (src/web/sources.rs)
google-error-not-configured = Die Google-Kalender-Integration ist nicht eingerichtet. Bitte deine Administratorin oder deinen Administrator, die Google-OAuth2-Zugangsdaten im Admin-Bereich einzutragen.
google-error-no-base-url = Die öffentliche Basis-URL ist nicht konfiguriert. Leg sie über die Umgebungsvariable CALRS_BASE_URL oder im Admin-Bereich unter Systemeinstellungen fest, bevor du OAuth2-Abläufe nutzt.
google-error-authorization = Google-Autorisierung fehlgeschlagen: { $error }
google-error-token-exchange = Token-Austausch fehlgeschlagen: { $error }
google-error-account-email = E-Mail-Adresse des Google-Kontos konnte nicht abgerufen werden: { $error }

# Admin panel notices (src/web/admin.rs)
admin-resource-feed-invalid = Ungültige Feed-URL: { $error }
admin-resource-feed-check-failed = Feed-Prüfung fehlgeschlagen: { $error }
admin-resource-caldav-invalid = Ungültige CalDAV-URL: { $error }
admin-resource-credentials-failed = Zugangsdaten konnten nicht gespeichert werden: { $error }
admin-resource-added = Ressource „{ $name }“ hinzugefügt. Termine im Cache: { $count }.
admin-resource-updated = Ressource aktualisiert. Termine im Cache: { $count }.
admin-resource-deleted = Ressource gelöscht.
admin-resource-not-found = Ressource nicht gefunden.
admin-resource-synced = Feed synchronisiert. Termine im Cache: { $count }.
admin-resource-sync-error = Synchronisierung fehlgeschlagen: { $error }
admin-resource-no-caldav-url = Keine CalDAV-URL konfiguriert, und aus der Feed-URL ließ sich keine ableiten.
admin-resource-no-write-credentials = Keine Schreib-Zugangsdaten verfügbar (kein Dienstkonto, und kein zustimmendes Mitglied hat eine passende Kalenderquelle).
admin-resource-write-passed = Schreibtest mit { $user } erfolgreich (angelegt: { $created }, aufgeräumt: { $cleaned }).
admin-resource-write-verified = bestätigt
admin-resource-write-unverified = nicht bestätigt
admin-resource-write-check-calendar = prüf den Ressourcenkalender
admin-resource-write-failed = Schreibtest fehlgeschlagen. Letzter Fehler: { $error }
admin-base-url-invalid = Die Basis-URL muss eine absolute http(s)-URL sein, z. B. https://cal.example.com
admin-company-link-invalid = Der Firmenlink muss mit http:// oder https:// beginnen
admin-smtp-host-required = Der SMTP-Host ist erforderlich.
admin-smtp-from-invalid = Gib eine gültige Absender-E-Mail-Adresse ein.
admin-smtp-port-invalid = Der SMTP-Port muss eine Zahl zwischen 1 und 65535 sein.
admin-smtp-tls-invalid = Der TLS-Modus muss „starttls“, „tls“ oder „none“ sein.
admin-smtp-password-required = Bei der ersten SMTP-Einrichtung ist ein Passwort erforderlich.
admin-sms-switch-needs-secret = Beim Wechsel des SMS-Gateways musst du die Zugangsdaten dieses Gateways eingeben.
admin-sms-settings-invalid = SMS-Einstellungen: { $error }.
admin-sms-sender-format = Ein numerischer SMS-Absender muss im internationalen Format angegeben werden, z. B. +4915112345678.
admin-sms-not-configured = SMS ist nicht eingerichtet.
admin-sms-load-failed = Die SMS-Konfiguration konnte nicht geladen werden.
admin-sms-check-accepted = Zugangsdaten vom Gateway akzeptiert.
admin-sms-bad-number = Das ist keine verwendbare Telefonnummer.
admin-sms-test-body = calrs-Testnachricht. Dein SMS-Gateway funktioniert.
admin-sms-test-accepted = Testnachricht vom Gateway angenommen.

## Local multi-factor authentication

mfa-title = Zwei-Faktor-Authentifizierung
mfa-invalid = Die Überprüfung ist fehlgeschlagen. Prüfe dein Passwort oder deinen Code. Jeder Code kann nur einmal verwendet werden; warte bei Bedarf auf einen neuen. Warte nach zu vielen Versuchen 15 Minuten. Starte neu, wenn die Einrichtung abgelaufen ist.
mfa-restart = Zurück zur Anmeldung
mfa-manage = Zwei-Faktor-Authentifizierung verwalten
mfa-recovery-help = Bewahre diese Wiederherstellungscodes sicher auf. Sie werden nur einmal angezeigt. Jeder Code ersetzt einmalig einen Authenticator-Code. Neue Wiederherstellungscodes ersetzen alle bisherigen Codes.
mfa-continue = Weiter zum Dashboard
mfa-enabled = Die Zwei-Faktor-Authentifizierung ist aktiviert.
mfa-disabled = Die Zwei-Faktor-Authentifizierung ist deaktiviert.
mfa-required-help = Dein Administrator verlangt Zwei-Faktor-Authentifizierung für lokale Konten. Du kannst sie nicht deaktivieren.
mfa-code = Authenticator-Code oder Wiederherstellungscode
mfa-regenerate = Neue Wiederherstellungscodes erstellen
mfa-disable = Zwei-Faktor-Authentifizierung deaktivieren
mfa-enable = Zwei-Faktor-Authentifizierung einrichten
mfa-changes-help = Änderungen erfordern dein Passwort. Zum Deaktivieren oder Ersetzen der Wiederherstellungscodes brauchst du auch einen Code; deine anderen Sitzungen werden dabei abgemeldet.
mfa-back = Zurück zu den Einstellungen
mfa-setup-help = Scanne diesen QR-Code mit deiner Authenticator-App oder gib den Schlüssel unten manuell ein. Gib einen sechsstelligen Code ein, um die Einrichtung abzuschließen. Die Einrichtung läuft nach 10 Minuten ab.
mfa-qr-alt = QR-Code zur Einrichtung der Authenticator-App
mfa-login-help = Gib einen Code aus deiner Authenticator-App oder einen gespeicherten Wiederherstellungscode ein. Diese Anmeldung läuft nach 10 Minuten ab.
mfa-totp-code = Sechsstelliger Authenticator-Code
mfa-verify = Überprüfen
mfa-policy-help = Diese Richtlinie gilt für lokale Konten. SSO verwendet die MFA des Identitätsanbieters. Lokale Administratoren müssen vor einer Änderung MFA einrichten. Die MFA-Pflicht meldet lokale Sitzungen ohne MFA-Prüfung ab und verlangt die Einrichtung bei der nächsten Anmeldung.
mfa-policy = MFA-Richtlinie für lokale Konten
mfa-optional = Optional
mfa-required = Erforderlich
mfa-save-policy = MFA-Richtlinie speichern

# First day of the week (#330)
week-start-label = Erster Tag der Woche
week-start-inherit-site = Standard der Website verwenden
week-start-inherit-owner = Standard des Eigentümers verwenden

## Booking directory and personal favorites

directory-title = Terminverzeichnis
directory-help = Finde den Terminlink eines Kollegen oder Teams. Markiere häufig geteilte Termine als Favoriten, um sie auf deinem Dashboard zu sehen.
directory-search = Nach Person, Team oder Termintitel suchen
directory-find = Suchen
directory-favorites-only = Nur Favoriten
directory-favorites = Favorisierte Termine
directory-browse = Verzeichnis durchsuchen
directory-generate = Einladungslink erstellen
directory-invite-help = Einmalige Einladung, 7 Tage gültig.
directory-add = { $title } zu Favoriten hinzufügen
directory-remove = { $title } aus Favoriten entfernen
directory-added = Zu Favoriten hinzugefügt.
directory-removed = Aus Favoriten entfernt.
directory-error = Diese Aktion konnte nicht abgeschlossen werden. Lade die Seite neu und versuche es erneut.
directory-manual-copy = Kopiere diesen Link manuell:
directory-empty = Keine Termine gefunden. Versuche eine andere Suche.
directory-favorites-empty = Keine Favoriten gefunden. Markiere Termine im Verzeichnis als Favoriten, um sie hier zu sehen.
directory-with-hosts = Mit { $names }

## ilia fork: TON payments (docs/ton.md)

# Price on the slot and booking pages
ton-slots-price = Preis: { $usd } $
ton-slots-price-approx = (≈ { $ton } TON)
ton-slots-price-note = Zahlung in TON nach der Terminwahl

# Event type form
ton-etf-section = Zahlung in TON
ton-etf-off-note = TON-Zahlungen sind auf diesem Server nicht eingerichtet. Ein kostenpflichtiger Termintyp kann erst gebucht werden, wenn sie eingerichtet sind.
ton-etf-enabled = Kostenpflichtige Buchung
ton-etf-enabled-help = Gäste zahlen in TON direkt nach der Terminwahl. Während der Zahlung bleibt der Termin für sie reserviert; eine unbezahlte Buchung wird automatisch freigegeben.
ton-etf-price = Preis
ton-etf-price-help = In USD: von 1 $ bis 10000 $; der Betrag in TON wird beim Buchen zum aktuellen Kurs berechnet.
ton-etf-window = Zahlungsfrist (Minuten)
ton-etf-window-help = Von 10 bis 240. Leer lassen für den Standardwert des Servers ({ $minutes } Min.).
ton-form-error-price = Der Preis muss zwischen 1 und 10000 US-Dollar liegen, mit höchstens zwei Nachkommastellen.
ton-form-error-window = Die Zahlungsfrist muss zwischen 10 und 240 Minuten liegen.
ton-form-error-team = Team-Termintypen können nicht kostenpflichtig sein.

# Booking error pages
ton-bae-title-unavailable = Dieser Termin kann nicht gebucht werden
ton-bae-body-unavailable = Die Zahlung ist vorübergehend nicht verfügbar, daher kann dieser Termin gerade nicht gebucht werden. Versuch es in ein paar Minuten noch einmal.
ton-bae-body-too-late = Bis zu diesem Termin bleibt zu wenig Zeit für die Zahlung. Wähle bitte einen späteren Termin.
ton-bae-body-already-open = Du hast mit dieser E-Mail-Adresse bereits eine unbezahlte Buchung. Wir können dir ihre Links schicken: Dort kannst du sie bezahlen oder stornieren, um den Termin freizugeben, und dann neu buchen.
ton-bae-body-personal-only = Dieser Termin kann nur auf der eigenen Seite des Gastgebers gebucht werden.
ton-bae-title-unpaid = Buchung nicht bezahlt
ton-bae-body-unpaid = Diese Buchung ist nicht bezahlt und kann daher nicht bestätigt werden. Um sie kostenlos zu bestätigen, nutze „Ohne Zahlung bestätigen“ auf der Buchungsseite deines Dashboards.
ton-bae-body-reschedule-unpaid = Diese Buchung ist nicht bezahlt und kann daher nicht verschoben werden. Bezahle sie oder storniere sie und buche einen anderen Termin.
ton-bae-body-pay-link-invalid = Dieser Zahlungslink ist ungültig.

# Payment page
ton-pay-page-title = Zahlung: { $title }
ton-pay-held-until = Der Termin ist bis { $time } ({ $zone }) für dich reserviert.
ton-pay-left = noch %1
ton-pay-amount-label = Zu zahlender Betrag
ton-pay-rate = ≈ { $usd } $ · 1 TON = { $rate } $ ({ $sources }, { $time } UTC)
ton-pay-open-wallet = Wallet öffnen
ton-pay-open-tonkeeper = In Tonkeeper öffnen
ton-pay-qr-alt = QR-Code mit den Zahlungsdaten
ton-pay-scan = Oder scanne diesen Code mit deiner Wallet-App.
ton-pay-address = Adresse
ton-pay-comment = Kommentar
ton-pay-copy = Kopieren
ton-pay-copied = Kopiert
ton-pay-warn-comment = Der Kommentar ist Pflicht: Daran erkennen wir deine Zahlung.
ton-pay-warn-encrypt = Verschlüssle den Kommentar nicht.
ton-pay-warn-token = Zahle in TON, nicht mit Tokens.
ton-pay-warn-exchange = Du zahlst von einer Börse? Trag den Code oben ins Memo-Feld (Kommentar) ein.
ton-pay-warn-deadline = Sende die Zahlung vor { $time }. Eine später gesendete Zahlung wird womöglich nicht berücksichtigt.
ton-pay-check = Ich habe bezahlt, jetzt prüfen
ton-pay-email = Link per E-Mail senden
ton-pay-emailed = Wir haben diesen Link an die E-Mail-Adresse deiner Buchung gesendet.
ton-pay-email-unavailable = Der Link konnte nicht gesendet werden. Lass diese Seite offen oder speichere ihre Adresse.
ton-pay-checking-now = Wir prüfen deine Zahlung. Diese Seite aktualisiert sich von selbst.
ton-pay-state-waiting = Wir warten auf deine Zahlung.
ton-pay-state-seen = Deine Zahlung ist im Netzwerk sichtbar. Wir warten, bis sie endgültig ist, was meist ein paar Sekunden dauert.
ton-pay-state-partial = Wir haben { $paid } TON erhalten. Bitte zahle die restlichen { $rest } TON mit demselben Kommentar.
ton-pay-state-checking = Die Zahlungsfrist ist abgelaufen. Falls du die Zahlung schon gesendet hast, prüfen wir sie. Diese Seite aktualisiert sich von selbst.
ton-pay-state-paid = Zahlung erhalten. Deine Buchung wird bestätigt …
ton-pay-state-confirmed = Bezahlt. Deine Buchung ist bestätigt, die Bestätigungs-E-Mail ist unterwegs.
ton-pay-state-waived = Für diese Buchung ist keine Zahlung nötig. Der Gastgeber bestätigt sie.
ton-pay-state-busy = Zahlung erhalten, aber dieser Termin ist nicht mehr frei. Wir melden uns bei dir.
ton-pay-state-expired = Die Zahlungsfrist ist abgelaufen und der Termin wurde freigegeben. Falls du trotzdem gezahlt hast, schreib uns von der E-Mail-Adresse deiner Buchung, dann erstatten wir den Betrag.
ton-pay-state-late = Deine Zahlung kam nach Ablauf der Frist, daher wurde die Buchung nicht bestätigt. Wir melden uns wegen der Erstattung. Du kannst uns auch von der E-Mail-Adresse deiner Buchung schreiben.
ton-pay-state-closed = Diese Buchung ist nicht mehr aktiv.
ton-pay-state-revoked = Diese Zahlungsadresse wird nicht mehr verwendet. Sende nichts an sie und buche bitte erneut.
ton-pay-rebook = Anderen Termin buchen

# Leaving the embedded widget
ton-exit-title = Termin reserviert
ton-exit-body = Der Termin ist bis { $time } ({ $zone }) für dich reserviert. Schließ die Zahlung ab, um deine Buchung zu bestätigen.
ton-exit-go = Zur Zahlung

# Dashboard
ton-dash-badge-awaiting = wartet auf Zahlung · { $amount } TON · bis { $time }
ton-dash-badge-partial = teilweise bezahlt · { $paid } von { $amount } TON · bis { $time }
ton-dash-badge-paid = bezahlt · { $amount } TON
ton-dash-badge-overpaid = bezahlt · { $excess } TON zu viel
ton-dash-badge-busy = bezahlt, Termin nicht mehr frei
ton-dash-badge-confirm-failed = bezahlt, nicht bestätigt · von Hand bestätigen
ton-dash-badge-refund = { $amount } TON bezahlt · Erstattung fällig
ton-dash-badge-waived = ohne Zahlung bestätigt
ton-dash-badge-unpaid = nicht bezahlt
ton-dash-waive = Ohne Zahlung bestätigen
ton-dash-error-unpaid = Diese Buchung ist nicht bezahlt und kann daher weder bestätigt noch verschoben werden. Nutze „Ohne Zahlung bestätigen“, um sie kostenlos zu bestätigen, oder lehne sie ab.
ton-dash-error-waive-failed = Die Buchung konnte nicht ohne Zahlung bestätigt werden: Sie wartet nicht mehr auf eine Zahlung.
ton-waive-page-title = Ohne Zahlung bestätigen
ton-waive-heading = Diese Buchung ohne Zahlung bestätigen?
ton-waive-body = Diese Buchung ist NICHT bezahlt ({ $amount } TON, fällig bis { $time }). Wenn du sie bestätigst, bekommt der Gast den Termin kostenlos, und eine später eingehende Zahlung muss von Hand erstattet werden.
ton-waive-submit = Kostenlos bestätigen
ton-waive-back = Zurück zu den Buchungen

# Guest emails
ton-email-link-subject = Schließ deine Buchung ab: { $event }
ton-email-link-message = Du hast begonnen, { $event } mit { $host } zu buchen. Schließ deine Buchung auf der Seite unten ab, bevor die Zeit abläuft.
ton-email-link-action = Buchung abschließen
ton-email-link-line = Schließ deine Buchung ab: { $url }
ton-email-expired-subject = Nicht gebucht: { $event }, { $date }
ton-email-expired-message = Die Zahlung für { $event } ist nicht rechtzeitig eingegangen, daher wurde der Termin freigegeben. Du kannst gern einen anderen Termin wählen.
ton-email-expired-partial = Wir haben einen Teil der Zahlung erhalten und melden uns wegen der Erstattung.
ton-email-rebook-line = Anderen Termin wählen: { $url }

# Owner emails
ton-email-row-amount = Betrag:
ton-email-row-paid = Erhalten:
ton-email-row-excess = Zu viel gezahlt:
ton-email-row-transfer = Überweisung:
ton-email-row-sender = Absender:
ton-email-row-tx = Transaktion:
ton-email-row-detail = Details:
ton-email-amount-value = { $amount } TON ≈ { $usd } $ (1 TON = { $rate } $)
ton-email-sender-wallet = { $address } (eine Wallet)
ton-email-sender-other = { $address } (keine Wallet: eine Börse oder ein Contract)
ton-email-dashboard-line = Buchungen: { $url }
ton-email-dashboard-action = Buchungen öffnen
ton-email-refund-rule = Erstattungen erfolgen von Hand aus deiner Wallet: an die Absenderadresse, wenn sie eine Wallet ist; sonst nur an eine Adresse, die der Gast von der E-Mail-Adresse seiner Buchung anfragt, in einer Nachricht mit seinem Zahlungs- oder Stornolink. Buchungscode, Betrag und Transaktion sind öffentlich und beweisen nichts.
ton-email-received-subject = Zahlung erhalten: { $event }, { $guest } ({ $date })
ton-email-received-heading = Zahlung erhalten
ton-email-received-message = { $guest } hat { $amount } TON bezahlt. Die Buchung ist bestätigt.
ton-email-alert-subject = TON-Zahlung braucht Aufmerksamkeit: { $what }
ton-email-alert-heading = Eine Zahlung braucht deine Aufmerksamkeit
ton-alert-partial = Teilzahlung
ton-alert-partial-body = Ein Teil des fälligen Betrags ist eingegangen. Die Buchung bleibt unbestätigt, bis der Rest kommt; sonst wird sie zur Frist freigegeben und der erhaltene Betrag muss erstattet werden.
ton-alert-overpaid = Überzahlung
ton-alert-overpaid-body = Eine Zahlung ist für eine bereits bezahlte Buchung eingegangen. Erstatte sie, wenn es sich lohnt.
ton-alert-late = verspätete Zahlung
ton-alert-late-body = Eine Zahlung ist nach ihrer Frist eingegangen. Die Buchung war bereits freigegeben und kann nicht mehr bestätigt werden. Erstatte die Zahlung oder bitte den Gast, neu zu buchen, und bestätige die neue Buchung ohne Zahlung.
ton-alert-unmatched = unbekannte Zahlung
ton-alert-unmatched-body = Eine Zahlung ohne bekannten Buchungscode ist eingegangen. Finde die zugehörige Buchung oder erstatte sie.
ton-alert-failed = fehlgeschlagene Überweisung
ton-alert-failed-body = Eine Überweisung an die Zahlungsadresse ist im Netzwerk fehlgeschlagen. Das Geld liegt womöglich noch in deiner Wallet: Prüfe die Transaktion im Explorer.
ton-alert-not-text = Überweisung ohne lesbaren Kommentar
ton-alert-not-text-body = Eine Überweisung mit verschlüsseltem Kommentar, einem Token oder anderem Inhalt ist eingegangen. Sie wird nie automatisch angerechnet: Prüfe sie von Hand.
ton-alert-other = ungewöhnliche Überweisung
ton-alert-other-body = Eine Überweisung konnte nicht automatisch verarbeitet werden. Prüfe sie in den Zahlungsdaten.
ton-email-problem-subject = Problem mit TON-Zahlung: { $event } ({ $date })
ton-email-problem-urgent-subject = Dringend: Eine bezahlte Buchung braucht eine Entscheidung: { $event } ({ $date })
ton-email-problem-heading = Eine bezahlte Buchung braucht deine Aufmerksamkeit
ton-problem-host-busy = Der Gast hat bezahlt, aber der Termin ist nicht mehr frei (ein Kalendereintrag oder eine andere Buchung). Der Termin bleibt bis zum Beginn reserviert. Bitte den Gast um einen neuen Termin, bestätige die Buchung von Hand oder lehne sie ab und erstatte die Zahlung.
ton-problem-partial = Nur ein Teil des Betrags wurde bezahlt, und die Buchung ist danach abgelaufen, wurde storniert oder abgelehnt oder ohne Zahlung bestätigt. Erstatte den erhaltenen Betrag, sofern du mit dem Gast nichts anderes vereinbart hast.
ton-problem-overpaid = Die Buchung ist bezahlt und bestätigt, aber mit mehr als dem fälligen Betrag. Erstatte den Überschuss, wenn es sich lohnt.
ton-problem-refund-due = Die Buchung wurde bezahlt und danach storniert oder abgelehnt. Erstatte die Zahlung.
ton-problem-confirmed-unpaid = Eine unbezahlte Buchung ist bestätigt. Das darf nie passieren: Prüfe, wie sie bestätigt wurde.
ton-problem-address-revoked = Die Zahlungsadresse dieser Buchung wurde widerrufen, die Buchung wird freigegeben.
ton-email-health-subject = TON-Zahlungen brauchen Aufmerksamkeit ({ $kind })
ton-email-health-heading = TON-Zahlungen brauchen Aufmerksamkeit
ton-health-indexer-down = Die Zahlungsüberwachung erreicht den Indexer nicht. Zahlungen werden nicht geprüft, und neue kostenpflichtige Buchungen werden abgelehnt, bis er wieder erreichbar ist.
ton-health-auth-failed = Der Indexer hat den API-Schlüssel abgelehnt (falsch oder für das andere Netzwerk). Prüfe CALRS_TON_API_KEY.
ton-health-wrong-network = Der Indexer antwortet für ein anderes Netzwerk als das eingestellte. Bis das behoben ist, wird nichts angerechnet und nichts läuft ab.
ton-health-wallet-inactive = Die Empfangs-Wallet ist nicht aktiv. Kostenpflichtige Buchungen werden abgelehnt, bis sie mindestens eine ausgehende Überweisung gemacht hat.
ton-health-lagging = Die Zahlungsüberwachung hinkt den Überweisungen an die Adresse hinterher (womöglich Spam). Unbezahlte Buchungen laufen erst ab, wenn sie aufgeholt hat.
ton-health-stuck-finality = Eine Überweisung ist seit Langem nicht endgültig. Die Überwachung wartet darauf, bevor sie weiterliest.
ton-health-watcher-crashed = Die Zahlungsüberwachung ist mehrmals abgestürzt und startet immer wieder neu. Prüfe das Serverprotokoll.
ton-health-disabled-with-open = TON-Zahlungen sind ausgeschaltet, während unbezahlte Buchungen offen sind. Ihre Termine werden zur Frist freigegeben.
ton-health-other-network-open = Es gibt offene Buchungen in einem anderen TON-Netzwerk als dem eingestellten. Sie werden zur Frist freigegeben.
ton-health-rate-unavailable = Es gibt keinen brauchbaren TON-Kurs (Quellen fehlen, weichen ab oder liegen außerhalb deines Korridors). Kostenpflichtige Buchungen werden abgelehnt, bis es einen gibt.
ton-health-capacity = Die Grenze offener oder täglicher unbezahlter Buchungen ist erreicht. Neue kostenpflichtige Buchungen werden vorerst abgelehnt.
ton-health-other = Die Zahlungsüberwachung hat ein Problem gemeldet. Prüfe das Serverprotokoll.

# TON follow-ups: releases, refunds, contact, resend
ton-resend-submit = Links per E-Mail schicken
ton-resend-title = Sieh in dein Postfach
ton-resend-done = Falls es für diese E-Mail-Adresse eine unbezahlte Buchung gibt, haben wir ihre Zahlungs- und Stornolinks dorthin geschickt.
ton-pay-state-taken = Dieser Termin wurde vergeben, bevor die Buchung bezahlt war, deshalb wurde sie freigegeben. Es ist nichts zu zahlen. Bitte wähle einen anderen Termin.
ton-pay-state-refund = Diese Buchung wurde ohne Bestätigung beendet. Die bezahlten { $paid } TON werden erstattet; der Gastgeber meldet sich bei dir.
ton-pay-contact = Fragen? Schreib an { $host }: { $email }.
ton-pay-copy-address = Adresse kopieren
ton-pay-copy-comment = Kommentar kopieren
ton-pay-copy-manual = Kopieren hat nicht geklappt. Der Text ist markiert: Kopiere ihn von Hand.
ton-email-row-pay-by = Zahlen bis:
ton-email-taken-message = Der Termin, den du für { $event } gewählt hast, wurde vergeben, bevor die Buchung bezahlt war, deshalb wurde sie freigegeben. Es ist nichts zu zahlen. Du kannst gern einen anderen Termin wählen.
ton-email-refund-message = Wir haben deine Zahlung für { $event } erhalten, aber der Termin konnte nicht bestätigt werden, deshalb wurde die Buchung freigegeben. { $host } erstattet die Zahlung und meldet sich bei dir.
ton-problem-host-busy-unpaid = Der Termin dieser unbezahlten Buchung wurde vergeben (eine Anfrage wurde darüber bestätigt, oder ein Kalendertermin kam hinzu), deshalb wurde die Buchung vor der Zahlung freigegeben. Falls schon etwas eingegangen ist, erstatte es.
ton-problem-confirm-failed = Der Gast hat bezahlt, aber die Buchung konnte nach mehreren Versuchen nicht automatisch bestätigt werden (ein Fehler, kein belegter Gastgeber: siehe Serverprotokoll). Der Termin bleibt bis zum Beginn reserviert. Bestätige die Buchung von Hand oder lehne sie ab und erstatte die Zahlung.
ton-health-capacity-near = Eine neue bezahlte Buchung hat den letzten offenen Platz oder das letzte Kontingent des Tages belegt. Weitere werden abgelehnt, bis Buchungen bezahlt oder freigegeben sind. Wenn das keine echte Nachfrage ist, blockiert womöglich jemand Termine, ohne zu zahlen.
ton-health-payment-alerts-held = Weitere Überweisungen sind für Buchungen eingegangen, zu denen du in der letzten Stunde schon eine E-Mail bekommen hast. Sie sind ohne eigene E-Mail erfasst: Prüfe die Zahlungsdaten.
ton-waive-body-partial = Diese Buchung ist nur teilweise bezahlt ({ $paid } von { $amount } TON erhalten, fällig bis { $time }). Wenn du sie bestätigst, bekommt der Gast den Termin, ohne die restlichen { $rest } TON zu zahlen. Erstatte den erhaltenen Betrag, sofern nichts anderes vereinbart ist.

## Telegram (ilia fork)

tg-when = { $date }, { $time } ({ $tz })
tg-in-minutes =
    { $count ->
        [one] in { $count } Minute
       *[other] in { $count } Minuten
    }
tg-in-hours =
    { $count ->
        [one] in { $count } Stunde
       *[other] in { $count } Stunden
    }
tg-in-days =
    { $count ->
        [one] in { $count } Tag
       *[other] in { $count } Tagen
    }
tg-reason = Grund: { $reason }
tg-link-line = { $label }: { $url }
tg-guest-confirmed = Buchung bestätigt: { $event }, { $when }. Gastgeber: { $host }.
tg-guest-pending = Anfrage gesendet: { $event }, { $when }. Die Antwort des Gastgebers kommt hier an.
tg-guest-approved = Anfrage bestätigt: { $event }, { $when }.
tg-guest-reschedule-approved = Neuer Termin bestätigt: { $event }, { $when }.
tg-guest-declined = Anfrage abgelehnt: { $event }, { $when }.
tg-guest-cancelled-by-host = Vom Gastgeber abgesagt: { $event }, { $when }.
tg-guest-cancelled-by-guest = Du hast deine Buchung storniert: { $event }, { $when }.
tg-guest-rescheduled = Buchung verschoben: { $event } ist jetzt am { $when }.
tg-guest-reschedule-pending = Neuer Termin angefragt: { $event }, { $when }. Die Antwort des Gastgebers kommt hier an.
tg-guest-reschedule-requested = Der Gastgeber bittet um einen neuen Termin für { $event }. Bisher: { $when }.
tg-guest-reminder = Erinnerung: { $event } beginnt { $in }, { $when }.
tg-guest-expired = Anfrage abgelaufen: { $event }, { $when }. Der Gastgeber hat nicht rechtzeitig geantwortet; bitte buche einen anderen Termin.
tg-button-cancel = Buchung stornieren
tg-button-reschedule = Verschieben
tg-button-calendar = Zum Kalender hinzufügen
tg-button-pick-time = Neuen Termin wählen
tg-button-rebook = Anderen Termin buchen
tg-button-bookings = Buchungen öffnen
tg-host-new-booking = Neue Buchung: { $event }, { $when }. Gast: { $guest }, { $email }.
tg-host-approval-request = Zu bestätigen: { $event }, { $when }. Gast: { $guest }, { $email }.
tg-host-reschedule-approval = Neuer Termin zu bestätigen: { $event }, { $when }. Gast: { $guest }, { $email }.
tg-host-rescheduled = { $guest } hat { $event } verschoben auf { $when }.
tg-host-cancelled = { $guest } hat abgesagt: { $event }, { $when }.
tg-host-guest-linked = { $guest } erhält Telegram-Benachrichtigungen zu { $event }, { $when }: { $username }.
tg-host-guest-linked-differs = Im Formular angegeben: { $entered }.
tg-host-no-username = kein Benutzername
tg-bot-help = Dieser Bot sendet Benachrichtigungen zu Buchungen auf { $site }. Zum Einschalten den Link auf der Bestätigungsseite oder in der E-Mail öffnen; bei einer kostenpflichtigen Buchung steht der Link auf ihrer Zahlungsseite. /stop schaltet sie aus. Ist eine Buchung mit diesem Chat verbunden, lässt sich hier auch dem Gastgeber schreiben.
tg-bot-linked-guest = Fertig: Benachrichtigungen zu dieser Buchung kommen hier an.
tg-bot-linked-host = Fertig: Gastgeber-Benachrichtigungen für { $name } kommen hier an.
tg-bot-link-invalid = Dieser Link funktioniert nicht mehr: Er gilt nur in einem Telegram-Chat und nur bis zum Ende des Termins.
tg-bot-link-replaced = Dieser Link wurde ersetzt. Die Zahlungsseite der Buchung erneut öffnen und den Telegram-Link von dort verwenden.
tg-bot-rate-limited = Zu viele Versuche. Bitte später erneut versuchen.
tg-bot-stopped = Benachrichtigungen sind aus. Zum erneuten Einschalten für eine Buchung ihren Link aus der Bestätigungs-E-Mail noch einmal öffnen; Gastgeber erstellen einen neuen Link unter Profil & Einstellungen → Telegram.
tg-bot-not-linked = Dieser Chat erhält keine Benachrichtigungen.
tg-cmd-help = So funktioniert dieser Bot
tg-cmd-stop = Benachrichtigungen ausschalten
tg-book-label = Telegram
tg-book-placeholder = @benutzername
tg-book-help-optional = Nach der Buchung kannst du Benachrichtigungen in Telegram einschalten.
tg-book-help-required = Nach der Buchung den Telegram-Link öffnen, um Benachrichtigungen zu erhalten.
tg-book-invalid-title = Telegram-Benutzername prüfen
tg-book-invalid = Einen Telegram-Benutzernamen wie @name angeben (Buchstaben, Ziffern und _, 4 bis 32 Zeichen).
tg-book-required = Telegram-Benutzernamen angeben.
tg-confirmed-title = Benachrichtigungen in Telegram
tg-confirmed-body = Den Bot öffnen und unten im Chat auf „Starten“ tippen: Benachrichtigungen zu dieser Buchung kommen dort an.
tg-confirmed-button = Telegram öffnen
tg-email-button = Benachrichtigungen in Telegram
tg-email-plain = Benachrichtigungen in Telegram: { $url }
tg-settings-nav = Telegram-Benachrichtigungen
tg-settings-title = Telegram
tg-settings-intro = Gastgeber-Benachrichtigungen im eigenen Telegram-Chat: neue Buchungen, zu bestätigende Anfragen, Absagen und Verschiebungen durch Gäste.
tg-settings-off = Telegram ist auf diesem Server nicht eingerichtet.
tg-settings-no-bot-name = Der Bot ist noch nicht bereit. Bitte in einer Minute erneut versuchen.
tg-settings-linked = Gastgeber-Benachrichtigungen gehen an { $username }. Verbunden: { $date }.
tg-settings-not-linked = Dein Telegram ist nicht verbunden.
tg-settings-link = Mein Telegram verbinden
tg-settings-link-open = Telegram öffnen
tg-settings-link-expires = Im Chat unten auf „Starten“ tippen. Der Link gilt einmal innerhalb von 15 Minuten.
tg-settings-link-limited = Zu viele Links in dieser Stunde. Bitte später erneut versuchen.
tg-settings-unlink = Trennen
tg-settings-modes = Telegram des Gastes pro Termintyp
tg-settings-modes-help = Optional oder erforderlich: Das Buchungsformular fragt nach dem Telegram des Gastes, und die Bestätigung bietet den Link zum Bot an.
tg-settings-modes-none = Keine Termintypen einzustellen.
tg-settings-mode-off = Aus
tg-settings-mode-optional = Optional
tg-settings-mode-required = Erforderlich
tg-settings-save = Speichern
tg-settings-saved = Gespeichert.
tg-settings-team = Team { $team }
tg-dashboard-label = Telegram { $username }
tg-dashboard-entered = (angegeben: { $entered })
tg-dashboard-not-linked = (noch nicht verbunden)
tg-bot-already-linked = Schon eingeschaltet: Benachrichtigungen zu dieser Buchung kommen in diesen Chat.
tg-bot-link-invalid-host = Dieser Link ist abgelaufen: Links aus den Einstellungen gelten einmal innerhalb von 15 Minuten. Einen neuen unter Profil & Einstellungen → Telegram erstellen.
tg-settings-linked-no-username = Gastgeber-Benachrichtigungen gehen an den verbundenen Telegram-Chat. Verbunden: { $date }.
tg-bot-description = Dieser Bot sendet Benachrichtigungen zu Buchungen auf { $site }: Bestätigungen, Änderungen und Erinnerungen. Zum Einschalten für eine Buchung unten auf die Schaltfläche tippen. /stop schaltet sie aus.
tg-bot-short-description = Buchungsbenachrichtigungen von { $site }.
tg-button-join = Teilnehmen

## Telegram: TON payments (ilia fork; docs/ton.md)

tg-guest-payment-due = Deine Buchung wartet auf die Zahlung: { $event }, { $when }. Betrag: { $amount } TON (≈ { $usd } $). Bezahle bis { $deadline }, sonst wird der Termin freigegeben.
tg-guest-payment-partial = Ein Teil der Zahlung für { $event }, { $when } ist eingegangen: { $paid } von { $amount } TON. Zahle die restlichen { $rest } TON bis { $deadline }, sonst wird der Termin freigegeben.
tg-guest-payment-received = Zahlung erhalten, deine Buchung ist bestätigt: { $event }, { $when }. Gastgeber: { $host }.
tg-guest-payment-late = Eine Zahlung von { $value } TON für { $event }, { $when } ist nach der Frist eingegangen und hat die Buchung deshalb nicht bestätigt. { $host } meldet sich wegen der Erstattung bei dir.
tg-guest-payment-expired = Die Zeit zum Bezahlen von { $event }, { $when } ist abgelaufen, deshalb wurde der Termin freigegeben. Du kannst einen anderen Termin buchen.
tg-guest-payment-taken = Der Termin, den du für { $event }, { $when } gewählt hast, wurde vergeben, bevor die Buchung bezahlt war, deshalb wurde sie freigegeben. Du musst nichts bezahlen. Du kannst einen anderen Termin buchen.
tg-guest-payment-refund = Wir haben deine Zahlung für { $event }, { $when } erhalten, aber der Termin konnte nicht bestätigt werden, deshalb wurde die Buchung freigegeben. { $host } erstattet die Zahlung und meldet sich bei dir.
tg-guest-payment-refund-part = Ein Teil der Zahlung war schon eingegangen: { $host } meldet sich wegen der Erstattung bei dir.
tg-guest-payment-held = Wir haben deine Zahlung für { $event }, { $when } erhalten, aber die Buchung konnte nicht automatisch bestätigt werden. { $host } meldet sich bei dir.
tg-button-pay = Bezahlen
tg-host-payment-received = Bezahlt: { $event }, { $when }. Gast: { $guest }, { $email }. { $paid } TON erhalten (≈ { $usd } $), die Buchung ist bestätigt.
tg-host-payment-problem = Zahlung braucht Aufmerksamkeit: { $event }, { $when }. Gast: { $guest }, { $email }.
tg-host-payment-transfer = Überweisung: { $value } TON.
tg-host-payment-amounts = Erhalten: { $paid } von { $amount } TON.
tg-host-payment-guest-telegram = Telegram des Gastes: { $username }.

## ilia fork: booking flow v2 (docs/ton.md, docs/operations.md)
troubleshoot-label-busy = Belegt
ton-etf-currency = Währung
ton-etf-currency-ton = TON
ton-etf-currency-usd = USD (in TON umgerechnet)
ton-etf-price-help-ton = In TON: von 1 bis 100000, der Gast zahlt genau diesen Betrag (100 TON, wenn das Feld leer bleibt).
ton-etf-usd-no-rates = Auf diesem Server sind keine Wechselkurse eingerichtet (CALRS_TON_RATE_MIN_USD und CALRS_TON_RATE_MAX_USD), daher kann ein Preis in USD nicht gebucht werden. Gib den Preis in TON an.
ton-form-error-price-ton = Der Preis muss zwischen 1 und 100000 TON liegen, mit höchstens zwei Nachkommastellen.
ton-form-error-currency = Die Währung des Preises muss TON oder USD sein.
ton-form-error-telegram = Für das Telegram des Gastes ist aus, optional oder erforderlich möglich.
ton-etf-telegram = Telegram des Gastes
ton-etf-telegram-help = Bei einer bezahlten Buchung standardmäßig erforderlich: Der Gast bekommt dann Zahlungserinnerungen und ist über Telegram erreichbar. Du kannst es optional machen oder ausschalten.
ton-slots-price-ton = Preis: { $ton } TON
ton-email-amount-value-ton = { $amount } TON
tg-guest-payment-due-ton = Deine Buchung wartet auf die Zahlung: { $event }, { $when }. Betrag: { $amount } TON. Bezahle bis { $deadline }, sonst wird der Termin freigegeben.
tg-host-payment-received-ton = Bezahlt: { $event }, { $when }. Gast: { $guest }, { $email }. { $paid } TON erhalten, die Buchung ist bestätigt.
ton-pay-renewed = Die Frist wurde verlängert ({ $n } von { $max }).
ton-dash-badge-renewed = Frist verlängert { $n } von { $max }
ton-email-renewed-subject = Dein Termin ist noch reserviert: { $event }
ton-email-renewed-message = Die Zahlung für { $event } mit { $host } ist noch nicht bei uns eingegangen, daher bleibt dein Termin bis { $deadline } reserviert. Wenn du schon bezahlt hast, ist nichts weiter zu tun: Die Zahlung wird angerechnet. Andernfalls bezahle bis dahin auf der Seite unten oder storniere die Buchung, wenn sich deine Pläne geändert haben.
tg-guest-payment-renewed = Die Zahlung für { $event }, { $when } ist noch nicht eingegangen, daher bleibt der Termin bis { $deadline } für dich reserviert. Bezahle die noch offenen { $rest } TON bis dahin, sonst wird der Termin freigegeben. Wenn du schon bezahlt hast, ist nichts weiter zu tun: Die Zahlung wird angerechnet.

## ilia fork: messages with guests (docs/operations.md)
msg-page-title = Nachrichten
msg-back = Buchungen
msg-guest = Gast: { $guest }
msg-sent = Nachricht gesendet.
msg-empty = Noch keine Nachrichten.
msg-label = Nachricht an den Gast
msg-help-email = Der Gast erhält sie per E-Mail von der Systemadresse; seine Antworten erscheinen hier und im Telegram-Chat.
msg-help-no-email = Auf diesem Server ist keine E-Mail eingerichtet.
msg-help-telegram = Der Gast hat Telegram verbunden und erhält sie auch dort.
msg-help-no-telegram = Der Gast hat Telegram nicht verbunden.
msg-help-limits = Höchstens 20 Nachrichten pro Buchung und Tag, davon 5 per E-Mail.
msg-send = Senden
msg-link = Nachrichten
msg-link-count = Nachrichten ({ $count })
msg-from-host = Gastgeber
msg-from-guest = Gast
msg-from-attendee = Teilnehmer: { $address }
msg-from-other = Andere Adresse, nicht die des Gastes: { $address }
msg-via-dashboard = Dashboard
msg-via-mcp = Agent
msg-via-telegram = Telegram
msg-status-email-queued = E-Mail in der Warteschlange
msg-status-email-sent = E-Mail gesendet
msg-status-email-failed = E-Mail nicht zugestellt
msg-status-telegram-queued = Telegram in der Warteschlange
msg-status-telegram-sent = an Telegram gesendet
msg-status-telegram-failed = nicht an Telegram zugestellt
msg-error-empty = Die Nachricht ist leer.
msg-error-too-long = Eine Nachricht darf höchstens 2000 Zeichen haben.
msg-error-limit = Das Tageslimit für Nachrichten zu dieser Buchung ist erreicht. Bitte morgen erneut versuchen.
msg-error-no-channel = Der Gast ist nicht erreichbar: Auf diesem Server ist keine E-Mail eingerichtet, und der Gast hat Telegram nicht verbunden.
msg-error-closed = Nachrichten sind nur zu ausstehenden oder bestätigten Buchungen möglich, bis eine Woche nach dem Termin.
msg-email-subject = Nachricht von { $host }: { $event }, { $date }
msg-email-intro = Nachricht von { $host } zu { $event }:
msg-email-footer = Zum Antworten an { $host } einfach auf diese E-Mail antworten.
tg-guest-host-message = Nachricht von { $host } zu { $event }, { $when }:
tg-guest-reply-hint = Zum Antworten einfach hier schreiben.
tg-host-guest-message = Nachricht von { $guest } ({ $email }) zu { $event }, { $when }:
tg-host-reply-hint = Zum Antworten an den Gast auf diese Nachricht antworten.
tg-bot-message-sent = Die Nachricht wurde an den Gastgeber weitergegeben.
tg-bot-message-pick-booking = Mit diesem Chat sind mehrere Buchungen verbunden. Bitte auf eine Nachricht zur gemeinten Buchung antworten.
tg-bot-message-closed = Diese Buchung nimmt keine Nachrichten mehr an.
tg-bot-message-after-payment = Nachrichten an den Gastgeber sind möglich, sobald die Buchung bezahlt ist.
tg-bot-message-limit = Zu viele Nachrichten. Bitte später erneut versuchen.
tg-bot-reply-sent = An den Gast gesendet.
tg-bot-reply-not-linked = Dieser Gast hat Telegram nicht verbunden, daher wurde die Antwort nicht gesendet. Bitte über die Buchung im Dashboard schreiben.
tg-bot-reply-not-allowed = Die Antwort wurde nicht gesendet: Die Buchung gehört nicht zu diesem Konto oder nimmt keine Nachrichten mehr an.
msg-error-internal = Die Nachricht konnte nicht gespeichert werden, daher wurde nichts gesendet. Bitte erneut versuchen.
msg-link-new = Nachrichten ({ $count } neu)
msg-recent-title = Letzte Termine
msg-recent-help = Termine, die begonnen haben oder in der letzten Woche geendet haben. Mit ihren Gästen können noch Nachrichten ausgetauscht werden.
tg-bot-message-sent-shortened = Die Nachricht wurde an den Gastgeber weitergegeben, gekürzt auf { $max } Zeichen.
tg-bot-reply-too-long = Die Antwort wurde nicht gesendet: Eine Nachricht darf höchstens { $max } Zeichen haben.
tg-bot-reply-empty = Die Antwort wurde nicht gesendet: Sie enthält keinen sichtbaren Text.
tg-bot-reply-failed = Die Antwort konnte nicht gespeichert werden und wurde daher nicht gesendet. Bitte erneut versuchen.
troubleshoot-not-synced = Kalender werden nur vom { $from } bis { $until } gelesen; Termine außerhalb dieser Tage werden nicht geladen und hier nicht angezeigt.
event-type-form-horizon-capped =
    { $days ->
        [one] Dieser Server bietet Termine höchstens { $days } Tag im Voraus an, unabhängig vom Buchungshorizont.
       *[other] Dieser Server bietet Termine höchstens { $days } Tage im Voraus an, unabhängig vom Buchungshorizont.
    }

## ilia fork: the system mailbox (src/inbox)
msg-help-email-no-inbox = Der Gast erhält sie per E-Mail von der Systemadresse. Antworten per E-Mail werden auf diesem Server nicht gelesen.
msg-via-email = E-Mail
msg-email-footer-no-reply = Diese Nachricht hat das Buchungssystem im Namen von { $host } gesendet; Antworten darauf werden nicht gelesen.
tg-host-guest-email = E-Mail von { $guest } ({ $email }) zu { $event }, { $when }:
tg-host-attendee-email = E-Mail von { $sender }, weiterer Teilnehmer der Buchung von { $guest } ({ $email }), zu { $event }, { $when }:
tg-host-other-email = E-Mail von einer anderen Adresse, nicht der des Gastes: { $sender }, im Verlauf der Buchung von { $guest } ({ $email }) zu { $event }, { $when }:
booking-blocked-body-system = Der Gastgeber verlangt mindestens { $minutes } Minuten Vorlaufzeit. Falls du nicht teilnehmen kannst, schreib bitte an <a href="{ $href }">{ $address }</a>.
booking-blocked-body-no-reply = Der Gastgeber verlangt mindestens { $minutes } Minuten Vorlaufzeit. Falls du nicht teilnehmen kannst, gib dem Gastgeber bitte auf anderem Weg Bescheid.

## ilia fork: passkeys and security keys (docs/passkeys-plan.md)

ilia-passkey-sign-in = Mit Passkey anmelden
ilia-mfa-login-help-passkey = Bestätige diese Anmeldung mit deinem Passkey oder Sicherheitsschlüssel. Die Anmeldung läuft nach 10 Minuten ab.
ilia-mfa-use-totp = Authenticator-Code verwenden
ilia-mfa-use-recovery = Wiederherstellungscode verwenden
ilia-mfa-use-passkey = Passkey verwenden
ilia-mfa-use-passkey-instead = Stattdessen einen Passkey oder Sicherheitsschlüssel verwenden
ilia-mfa-totp-login-help = Gib den sechsstelligen Code aus deiner Authenticator-App ein. Die Anmeldung läuft nach 10 Minuten ab.
ilia-mfa-recovery-code = Wiederherstellungscode
ilia-mfa-recovery-login-help = Gib einen deiner gespeicherten Wiederherstellungscodes ein. Jeder Code funktioniert nur einmal. Die Anmeldung läuft nach 10 Minuten ab.
ilia-passkey-unsupported = Dieser Browser unterstützt keine Passkeys oder Sicherheitsschlüssel. Wähle unten eine andere Methode oder öffne diese Seite in einem aktuellen Browser.
ilia-passkey-noscript = Passkeys brauchen JavaScript. Aktiviere es für diese Website oder wähle unten eine andere Methode.
ilia-passkey-cancelled = Die Anfrage wurde abgebrochen oder ist abgelaufen. Versuche es erneut oder wähle eine andere Methode.
ilia-passkey-invalid = Der Passkey oder Sicherheitsschlüssel konnte nicht überprüft werden, oder die Anfrage ist abgelaufen. Versuche es erneut oder wähle eine andere Methode.
ilia-passkey-key-type = Dieser Browser kann diesen Schlüsseltyp nicht lesen, daher lässt sich der Schlüssel hier nicht hinzufügen. Versuche es mit einem anderen Browser, etwa einem aktuellen Firefox oder Chrome, oder wähle eine andere Methode.
ilia-passkey-wrong-site = Passkeys funktionieren nur unter { $origin }. Öffne diese Seite dort und versuche es erneut, oder wähle eine andere Methode.
ilia-mfa-enroll-help = Wähle, womit du jede Anmeldung bestätigst. Am sichersten ist ein Passkey oder Sicherheitsschlüssel: Er funktioniert nur auf dieser Website, eine gefälschte Kopie der Seite kann ihn nicht nutzen.
ilia-passkey-add-security-key = Sicherheitsschlüssel hinzufügen
ilia-passkey-add-device = Passkey auf diesem Gerät oder Telefon hinzufügen
ilia-passkey-kinds-help = Ein Sicherheitsschlüssel ist ein USB- oder NFC-Schlüssel wie ein YubiKey. Ein Passkey auf diesem Gerät oder Telefon nutzt Touch ID, Face ID, Windows Hello oder die Bildschirmsperre des Telefons und kann mit anderen Geräten synchronisiert werden.
ilia-mfa-use-app-instead = Stattdessen eine Authenticator-App verwenden
ilia-passkey-name = Name des Schlüssels
ilia-passkey-name-help = Optional, zum Beispiel „YubiKey am Schlüsselbund“ oder „MacBook“. Bis zu 64 Zeichen.
ilia-passkey-default-security-key = Sicherheitsschlüssel { $n }
ilia-passkey-default-device = Passkey { $n }
ilia-passkey-register-help = Klicke auf eine Schaltfläche und folge deinem Browser: Stecke den Sicherheitsschlüssel ein und berühre ihn, oder bestätige mit Touch ID, Face ID, Windows Hello oder deinem Telefon.
ilia-passkey-exists = Dieser Schlüssel ist bereits registriert.
ilia-mfa-recovery-help = Bewahre diese Wiederherstellungscodes sicher auf; sie werden nur einmal angezeigt. Jeder Code ersetzt bei einer Anmeldung einen Passkey oder einen Authenticator-Code. Neue Codes ersetzen alle bisherigen.
ilia-mfa-status-on = Die Zwei-Faktor-Authentifizierung ist aktiviert.
ilia-mfa-status-off = Die Zwei-Faktor-Authentifizierung ist deaktiviert.
ilia-mfa-setup-intro = Füge jeder Anmeldung einen zweiten Schritt hinzu: einen Passkey oder Sicherheitsschlüssel (YubiKey, Touch ID, Face ID, Windows Hello, dein Telefon) oder eine Authenticator-App.
ilia-passkey-list = Passkeys und Sicherheitsschlüssel
ilia-passkey-none = Noch keine Passkeys oder Sicherheitsschlüssel.
ilia-passkey-kind-hardware = Hardware-Schlüssel
ilia-passkey-kind-synced = Synchronisierter Passkey
ilia-passkey-added = Hinzugefügt am { $date }
ilia-passkey-last-used = Zuletzt verwendet am { $date }
ilia-passkey-never-used = Noch nicht verwendet
ilia-passkey-other-site = Registriert für { $rp_id }; unter dieser Adresse funktioniert er nicht.
ilia-passkey-rename = Umbenennen
ilia-passkey-delete = Entfernen
ilia-passkey-add = Passkey oder Sicherheitsschlüssel hinzufügen
ilia-passkey-new-name = Neuer Name
ilia-passkey-limit = Du hast die Höchstzahl an Schlüsseln erreicht ({ $max }). Entferne einen, um einen neuen hinzuzufügen.
ilia-passkey-backup-advice = Füge einen zweiten Schlüssel hinzu und bewahre ihn sicher auf, damit du dich bei Verlust eines Schlüssels nicht aussperrst.
ilia-passkey-unavailable = Passkeys sind deaktiviert: Die öffentliche Adresse des Servers (CALRS_BASE_URL) ist keine https://-Adresse. Wende dich an den Administrator. Bereits hinzugefügte Schlüssel werden aufgelistet und können entfernt werden.
ilia-mfa-totp-title = Authenticator-App
ilia-mfa-totp-on = Eine Authenticator-App ist eingerichtet.
ilia-mfa-totp-off = Keine Authenticator-App eingerichtet.
ilia-mfa-totp-add = Authenticator-App einrichten
ilia-mfa-totp-remove = Authenticator-App entfernen
ilia-mfa-recovery-title = Wiederherstellungscodes
ilia-mfa-recovery-left =
    { $count ->
        [one] { $count } Wiederherstellungscode übrig
       *[other] { $count } Wiederherstellungscodes übrig
    }
ilia-mfa-last-factor = Die Zwei-Faktor-Authentifizierung ist vorgeschrieben, daher kann deine letzte Anmeldemethode nicht entfernt werden. Füge zuerst eine weitere hinzu.
ilia-mfa-last-factor-off = Das ist deine letzte Methode: Wenn du sie entfernst, wird die Zwei-Faktor-Authentifizierung deaktiviert und deine Wiederherstellungscodes werden gelöscht.
ilia-mfa-changes-help = Jede Änderung hier erfordert dein Passwort und einen Passkey oder Code und meldet deine anderen Sitzungen ab.
ilia-mfa-confirm-title = Bestätige, dass du es bist
ilia-mfa-confirm-help = Gib dein Passwort ein und bestätige dann mit einem Passkey oder einem Code.
ilia-passkey-confirm = Mit Passkey bestätigen
ilia-mfa-confirm-code = Mit Code bestätigen
ilia-mfa-action-passkey-add = Du fügst einen Passkey oder Sicherheitsschlüssel hinzu.
ilia-mfa-action-passkey-rename = Du benennst den Schlüssel „{ $name }“ um.
ilia-mfa-action-passkey-delete = Du entfernst den Schlüssel „{ $name }“.
ilia-mfa-action-totp-add = Du richtest eine Authenticator-App ein.
ilia-mfa-action-totp-remove = Du entfernst die Authenticator-App.
ilia-mfa-action-regenerate = Du ersetzt alle Wiederherstellungscodes.
ilia-mfa-action-policy = Du setzt die Zwei-Faktor-Richtlinie für lokale Konten auf:
ilia-mfa-policy-continue = Weiter

## ilia fork: the instance name (CALRS_BRAND_NAME) and the error page (src/web/error_page.rs)
ilia-login-subtitle = Melde dich bei deinem { $brand }-Konto an
ilia-register-subtitle = Registriere ein neues { $brand }-Konto
ilia-error-not-found = Nicht gefunden
ilia-error-not-found-body = Unter dieser Adresse gibt es nichts. Prüfe den Link.
ilia-error-gone = Dieser Link ist nicht mehr gültig
ilia-error-bad-request = Ungültige Anfrage
ilia-error-forbidden = Zugriff eingeschränkt

# ilia fork: TON price by demand (the ladder)
ton-slots-ladder-now = Jetzt: { $ton } TON
ton-slots-ladder-now-free = Jetzt: kostenlos, der Gastgeber bestätigt die Anfrage
ton-slots-ladder-locked = Der Preis wird bei der Buchung festgelegt.
ton-slots-ladder-summary-free = Kostenlos, solange der Kalender leer ist, dann { $from } bis { $to } TON, je voller er wird.
ton-slots-ladder-summary = { $from } bis { $to } TON, je nachdem, wie voll der Kalender ist.
ton-slots-ladder-all-free = Kostenlos; der Gastgeber bestätigt jede Anfrage.
ton-slots-ladder-fee = Die Netzwerkgebühr zahlt deine Wallet.
ton-quote-title = Der Preis hat sich geändert
ton-quote-was = Dir wurden { $ton } TON angezeigt.
ton-quote-was-free = Dir wurde eine kostenlose Buchung angezeigt.
ton-quote-now = Der Preis jetzt: { $ton } TON.
ton-quote-now-free = Jetzt ist es kostenlos: Der Gastgeber bestätigt die Anfrage.
ton-quote-why = Seit dem Laden der Seite haben sich der Kalender oder die Preise geändert. Es wurde noch nichts gebucht.
ton-quote-why-free-used = Du hattest vor Kurzem schon eine kostenlose Buchung, daher ist diese Buchung kostenpflichtig. Es wurde noch nichts gebucht.
ton-quote-confirm-help = Bitte bestätige den Preis. Es wurde noch nichts gebucht.
ton-quote-confirm = Für { $ton } TON buchen
ton-quote-confirm-free = Kostenlos anfragen
ton-bae-body-free-used = Du hattest vor Kurzem schon eine kostenlose Buchung, daher kann diese nicht kostenlos sein. Versuche es bitte später noch einmal.
ton-form-error-mode = Wähle einen festen Preis oder den Preis nach Nachfrage.
ton-etf-mode = Preisgestaltung
ton-etf-mode-fixed = Fester Preis
ton-etf-mode-ladder = Nach Nachfrage (die Preisstaffel der Seite, in TON)
ton-etf-ladder-now = Aktuelle Staffel:
ton-etf-ladder-help = Der Preis ist die Stufe für die Zahl deiner anstehenden Buchungen in den nächsten Tagen (bestätigte oder bezahlte, kostenlose Anfragen, die auf deine Bestätigung warten, und Team-Buchungen, die du leitest; unbezahlte zählen nicht): zuerst kostenlos (jede kostenlose Buchung bestätigst du), dann höher, je voller dein Kalender wird. Gäste sehen den aktuellen Preis und zahlen den angezeigten. Die Staffel legt der Administrator fest.

# ilia fork: the TON settings page (/dashboard/admin/ton)
nav-admin-ton = TON-Zahlungen
ton-etf-ladder-edit = Staffel bearbeiten
ton-admin-page-title = TON-Zahlungen
ton-admin-heading = TON-Zahlungen
ton-admin-intro = Diese Einstellungen gelten für TON-Zahlungen auf diesem Server. Ein Wert aus der Umgebung des Servers hat Vorrang, dann ein hier oder mit config apply gespeicherter, dann der Standardwert.
ton-admin-saved-live = Gespeichert. Die neuen Einstellungen gelten ab sofort; offene Rechnungen behalten ihre Beträge und Fristen.
ton-admin-saved-off = Gespeichert. Zahlungen sind auf diesem Server aus (CALRS_TON_NETWORK ist nicht gesetzt); die Einstellungen gelten, sobald sie an sind.
ton-admin-saved-restart = Gespeichert. Die Einstellungen gelten nach einem Neustart von calrs.
ton-admin-saved-invalid = Gespeichert, aber die Einstellungen des Servers sind insgesamt ungültig, daher bleiben Zahlungen aus. Siehe den Status unten.
ton-admin-saved-none = Nichts geändert.
ton-admin-errors = Nichts gespeichert: Korrigiere die unten markierten Felder.
ton-admin-status = Status
ton-admin-status-on = Zahlungen sind an
ton-admin-status-off = Zahlungen sind aus
ton-admin-network = Netzwerk
ton-admin-address = Wallet-Adresse
ton-admin-address-raw = Rohform
ton-admin-indexer = Indexer
ton-admin-watcher = Watcher
ton-admin-watcher-ready = bereit
ton-admin-watcher-not-ready = nicht bereit: { $reason }
ton-admin-last-step = letzter Schritt
ton-admin-open-invoices = Offene Rechnungen
ton-admin-api-key = API-Schlüssel des Indexers
ton-admin-coingecko-key = CoinGecko-Schlüssel
ton-admin-secret-credentials = gesetzt (systemd-Credentials)
ton-admin-secret-env = gesetzt (Umgebung)
ton-admin-not-set = nicht gesetzt
ton-admin-env-only = Netzwerk, Wallet-Adresse, Indexer und API-Schlüssel werden nur in der Umgebung des Servers und in systemd-Credentials gesetzt, nie auf dieser Seite.
ton-admin-src-env = durch die Umgebung gesetzt ({ $var })
ton-admin-src-apply = durch config apply gesetzt
ton-admin-src-panel = hier gesetzt
ton-admin-src-default = Standard
ton-admin-locked-env = Um es hier zu ändern, entferne { $var } aus der Umgebung des Servers und starte calrs neu.
ton-admin-locked-apply = Ändere es im Dokument von config apply oder lass es in dessen ton-Abschnitt weg, um es an diese Seite zu übergeben.
ton-admin-default-hint = Leer lassen für den Standardwert: { $value }.
ton-admin-blank-default = Leer lassen für keinen Wert.
ton-admin-group-pricing = Preis nach Nachfrage
ton-admin-group-timing = Fristen der Rechnungen
ton-admin-group-rate = Wechselkurs (Preise in USD)
ton-admin-group-alerts = Warnungen
ton-admin-f-ladder = Staffel (TON nach Anzahl der Buchungen)
ton-admin-f-free_per_guest_days = Eine kostenlose Buchung pro Gast innerhalb von (Tagen, 0 = ohne Grenze)
ton-admin-f-window_min = Zahlungsfenster (Minuten)
ton-admin-f-grace_min = Nachfrist nach dem Fenster (Minuten)
ton-admin-f-fuse_hours = Sicherheitsfrist nach der Nachfrist (Stunden)
ton-admin-f-renew_min_lead_hours = Ein unbezahltes Fenster verlängern, wenn der Termin mindestens so weit entfernt ist (Stunden)
ton-admin-f-renew_window_min = Verlängertes Fenster (Minuten)
ton-admin-f-max_renewals = Verlängerungen pro Rechnung
ton-admin-f-underpay_tolerance_ton = Akzeptierte Unterzahlung (TON)
ton-admin-f-rate_min_usd = Niedrigster akzeptierter Kurs (USD pro TON)
ton-admin-f-rate_max_usd = Höchster akzeptierter Kurs (USD pro TON)
ton-admin-f-rate_max_spread_pct = Größte Abweichung zwischen den Quellen (%)
ton-admin-f-rate_max_age_min = Ältester akzeptierter Kurs (Minuten)
ton-admin-f-rate_max_jump_pct = Größter Sprung gegenüber dem letzten Kurs (%)
ton-admin-f-alert_email = E-Mail für Warnungen (leer: alle Admins)
ton-admin-ladder-help = Stufen durch Kommas getrennt: der Preis bei 0, 1, 2… bestätigten Buchungen in den kommenden Tagen; die letzte Stufe ist die Obergrenze. „free“ nur am Anfang: Eine kostenlose Buchung wartet auf die Bestätigung des Inhabers.
ton-admin-ladder-row = Ab %1 Buchungen
ton-admin-ladder-add = Neue Stufe
ton-admin-ladder-remove = Entfernen
ton-admin-free = kostenlos
ton-admin-preview = Vorschau
ton-admin-preview-button = Vorschau der Staffel
ton-admin-previewing = Eine Vorschau der oben eingegebenen Staffel. Noch ist nichts gespeichert.
ton-admin-ladder-invalid = Die geltende Staffel ist ungültig: { $error }
ton-admin-preview-n = Anstehende Buchungen
ton-admin-preview-price = Preis
ton-admin-preview-or-more = { $n } oder mehr
ton-admin-my-demand = Deine anstehenden Buchungen jetzt: { $n }. Dein Staffelpreis jetzt: { $price }.
ton-admin-types = Terminarten mit Preis
ton-admin-types-none = Keine Terminart hat einen Preis.
ton-admin-type = Terminart
ton-admin-type-owner = Inhaber
ton-admin-type-price = Preis
ton-admin-type-now = Jetzt
ton-admin-mode-ladder = nach Nachfrage
ton-admin-type-off = Preis ausgeschaltet
ton-admin-type-below-ceiling = Auf oder unter der Obergrenze der Staffel: Ein Preis für „jederzeit“ liegt normalerweise darüber.
ton-admin-save = Speichern
ton-admin-audit = Letzte Änderungen
ton-admin-audit-none = Noch keine Änderungen.
ton-admin-audit-at = Wann
ton-admin-audit-who = Wer
ton-admin-audit-key = Einstellung
ton-admin-audit-change = Änderung
ton-admin-audit-default = (Standard)
ton-admin-via-apply = config apply

## ilia fork: the debug booking page (CALRS_DEBUG_BOOKING, src/web/debug_booking.rs)

nav-admin-debug-booking = Testbuchung
dbg-title = Testbuchung
dbg-intro = Einen Testtermin so buchen, wie ein Gast es täte, ohne jedes Mal das Gastformular auszufüllen. Die Buchung läuft über denselben Weg wie die öffentliche Buchungsseite: die Prüfungen, der Preis und die TON-Rechnung, das Freihalten der Zeit und die E-Mails. Die Zahlung wird nie übersprungen und nie als erhalten markiert: Bezahle die Rechnung aus deiner Wallet oder storniere die Buchung.
dbg-switch-note = Schalte diese Seite nach dem Debuggen wieder aus, mit dem Schalter oben auf der Seite TON-Zahlungen:
dbg-cancelled = Stornierte Testbuchungen: { $count }.
dbg-no-types = Es gibt keinen aktiven persönlichen Termintyp.
dbg-event-type = Termintyp
dbg-priced = kostenpflichtig
dbg-private-short = privat
dbg-choose = Anzeigen
dbg-price = Preis
dbg-price-now-free = jetzt kostenlos (die Anfrage wird von dir bestätigt)
dbg-price-now-ton = jetzt { $ton } TON
dbg-price-ladder = nach Nachfrage, bei der Buchung festgelegt
dbg-price-free-used = dieser Gast hatte vor Kurzem schon eine kostenlose Buchung, daher gilt der erste Preis
dbg-free = kostenlos
dbg-private = Dieser Termintyp ist nicht öffentlich: Für jede Testbuchung wird eine einmalige Einladung an die eigene Adresse erstellt.
dbg-ton-off = TON-Zahlungen sind auf diesem Server ausgeschaltet, daher kann ein kostenpflichtiger Termintyp jetzt nicht gebucht werden.
dbg-captcha-on = Für Buchungsformulare ist Captcha eingeschaltet. Diese Seite kann es nicht lösen, daher lehnt der Buchungsweg Testbuchungen ab, solange Captcha eingeschaltet ist.
dbg-err-username = Das Konto hat keinen Benutzernamen und damit keine Buchungsseiten.
dbg-slot = Zeit
dbg-tz = Zeiten in { $tz }, der Zeitzone des Profils.
dbg-no-slots = In den nächsten 31 Tagen ist keine Zeit frei.
dbg-name = Name
dbg-email = E-Mail
dbg-email-help = Nur die eigenen Adressen: die Anmelde-E-Mail und die Buchungs-E-Mail.
dbg-telegram-username = Telegram-Benutzername
dbg-telegram-required = für diesen Termintyp erforderlich
dbg-notes = Notizen
dbg-notes-default = Testbuchung von der Debug-Seite.
dbg-own-chat = Telegram: Nachrichten an den Gast in meinen eigenen verknüpften Telegram-Chat zustellen
dbg-own-chat-off = Telegram ist auf diesem Server ausgeschaltet (kein Bot-Token).
dbg-own-chat-linked = Verknüpfter Chat: { $who }.
dbg-own-chat-help = Die Gastseite der Buchung (was zu zahlen ist, die Bestätigung, die Erinnerung, eine Stornierung) geht in den Chat, in dem der Bot schon schreibt, als wäre der Link des Gastes dort geöffnet worden. Ohne Häkchen zeigt die Zahlungsseite den Link des Gastes wie üblich.
dbg-own-chat-not-linked = Der Telegram-Chat ist nicht verknüpft. Zuerst verknüpfen, dann hierher zurückkehren:
dbg-own-chat-no-username = kein Benutzername
dbg-test-note = Das ist eine echte Buchung, als Test markiert. Solange sie besteht, hält sie die Zeit frei, zählt für den Preis nach Nachfrage und verschickt die üblichen E-Mails und Nachrichten an die eigene Adresse. Nach dem Test stornieren.
dbg-submit = Als Gast buchen
dbg-list = Meine Testbuchungen
dbg-list-none = Noch keine Testbuchungen.
dbg-col-when = Wann
dbg-col-type = Termintyp
dbg-col-status = Status
dbg-col-payment = Zahlung
dbg-col-telegram = Telegram
dbg-open-pay = Zahlungsseite
dbg-own-chat-yes = mein Chat
dbg-cancel-all = Alle meine Testbuchungen stornieren ({ $count })
dbg-cancel-all-help = Storniert jede eigene Testbuchung, die noch besteht (eine ausstehende wird abgelehnt), so wie die Buchungsübersicht es tut: Offene Rechnungen werden geschlossen und die üblichen Benachrichtigungen verschickt. Andere Buchungen bleiben unberührt.
dbg-cancel-reason = Testbuchung von der Debug-Seite storniert.
dbg-impersonating = Die Testbuchung ist nur ohne Identitätswechsel verfügbar. Beende zuerst den Identitätswechsel.
dbg-err-type = Wähle einen der eigenen Termintypen.
dbg-err-email = Nur eine der eigenen Adressen ist möglich.
dbg-err-slot = Wähle eine Zeit aus der Liste.
dbg-err-own-chat = Zuerst den Telegram-Chat verknüpfen oder das Telegram-Häkchen entfernen.
dbg-own-chat-failed = Die Testbuchung wurde angelegt, aber die Nachrichten an den Gast konnten nicht in deinen Telegram-Chat geleitet werden: Der Chat wurde getrennt, der Bot gestoppt oder der Link ist inzwischen abgelaufen. Die Buchung besteht und hält die Zeit frei: Öffne ihre Zahlungsseite in der Liste unten oder storniere sie.
dbg-st-pending = ausstehend
dbg-st-confirmed = bestätigt
dbg-st-cancelled = storniert
dbg-st-declined = abgelehnt
dbg-inv-open = wartet auf Zahlung
dbg-inv-paid = bezahlt
dbg-inv-confirmed = bezahlt und bestätigt
dbg-inv-expired = abgelaufen
dbg-inv-closed = geschlossen
dbg-inv-waived = ohne Zahlung bestätigt
dbg-banner = DEBUG · Testbuchung
dbg-badge-test = TEST
ton-admin-debug-title = Debug
ton-admin-debug-intro = Testbuchung: deine eigenen Termintypen so buchen, wie ein Gast es tut (der echte Preis, die Rechnung, die Zahlungsseite und Telegram), ohne das Gastformular auszufüllen. Nur für Admins. Solange sie aus ist, antwortet ihre Seite mit 404 und nichts verlinkt sie.
ton-admin-debug-state-on = Testbuchung: an
ton-admin-debug-state-off = Testbuchung: aus
ton-admin-debug-turn-on = Einschalten
ton-admin-debug-turn-off = Ausschalten
ton-admin-debug-open = Testbuchung öffnen
ton-admin-debug-locked-env = Von der Umgebung des Servers festgelegt ({ $var }={ $value }). Um es hier zu schalten, entferne { $var } aus der Umgebung und starte calrs neu.
ton-admin-debug-locked-apply = Von config apply festgelegt (site.ton.debug_booking). Ändere es im Dokument von config apply oder lass den Schlüssel weg, um es an diese Seite zu übergeben.
ton-admin-debug-saved-on = Die Testbuchung ist an.
ton-admin-debug-saved-off = Die Testbuchung ist aus.
ton-admin-debug-saved-locked = Nichts geändert: Der Schalter ist gesperrt.
ton-admin-f-debug_booking = Testbuchung
