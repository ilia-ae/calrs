# Deutsch, formelle Anrede (Sie) — overlay on i18n/de/main.ftl.
# Only the messages that address the reader ("du" in de) are listed here; every
# other key is read from i18n/de (fallback de-formal -> de -> en, `parent_lang`).
# When a German string addresses the reader, add its "Sie" form here.
# `german_formal_addresses_the_reader_formally` (src/i18n.rs) fails when the
# effective formal text (override, else de) still says du/dich/dir/dein or uses an
# informal imperative, and `german_formal_overrides_match_german` fails on an
# override whose key is gone from de, whose placeables differ, or which is
# identical to de.

# Booking confirmation page (templates/confirmed.html)

confirmed-subtitle-reschedule-requested = Ihre Anfrage zur Terminverschiebung wurde an { $host } gesendet. Sie erhalten eine E-Mail an { $email }, sobald sie genehmigt ist.
confirmed-subtitle-rescheduled = Ihre Buchung wurde verschoben. Eine Bestätigungs-E-Mail wurde an { $email } gesendet.
confirmed-subtitle-pending = Ihre Buchungsanfrage wurde an { $host } gesendet. Sie erhalten eine E-Mail an { $email }, sobald sie bestätigt ist.

# Slot picker (templates/slots.html)

slots-tz-label = Ihre Zeitzone

slots-click-highlighted = Klicken Sie auf ein hervorgehobenes Datum, um verfügbare Zeiten zu sehen

# Booking form (templates/book.html)

book-name-label = Ihr Name
book-email-invalid = Bitte geben Sie eine vollständige E-Mail-Adresse samt Domain an (z. B. jane@example.com).
book-notes-placeholder = Möchten Sie etwas Bestimmtes besprechen?
book-phone-optional-consequence = Lassen Sie das Feld leer, wenn Sie keine SMS zu dieser Buchung erhalten möchten.
book-phone-invalid = Bitte geben Sie eine per SMS erreichbare Nummer an oder lassen Sie das Feld leer.
captcha-initial-state = Bestätigen Sie, dass Sie ein Mensch sind
captcha-solved = Sie sind ein Mensch
captcha-verify-aria = Klicken Sie, um zu bestätigen, dass Sie ein Mensch sind
captcha-required = Bitte bestätigen Sie, dass Sie ein Mensch sind
captcha-error-aria = Ein Fehler ist aufgetreten, bitte versuchen Sie es erneut

# Shared labels used across the cancel / decline / approve / reschedule / claim flows

common-close-page = Sie können diese Seite schließen.

# Cancel flow (booking_cancel_form.html, booking_cancelled_guest.html)

cancel-subtitle = Sie sind dabei, Ihre Buchung zu stornieren.
cancel-reason-placeholder-host = Teilen Sie dem Gastgeber den Grund mit...
cancelled-subtitle = Ihre Buchung wurde storniert und der Gastgeber wurde benachrichtigt.

# Decline flow (booking_decline_form.html, booking_declined.html)

decline-subtitle = Sie sind dabei, diese Buchungsanfrage abzulehnen.
decline-reason-placeholder-guest = Teilen Sie dem Gast den Grund mit...

# Approve flow (booking_approve_form.html, booking_approved.html)

approve-subtitle = Sie sind dabei, diese Buchungsanfrage zu genehmigen.

# Claim flow (booking_claim_form.html, booking_claimed.html, booking_already_claimed.html)

claim-subtitle = Sie sind dabei, diese Buchung zu übernehmen. Sie werden als Teilnehmer hinzugefügt.
claimed-subtitle = Sie haben diese Buchung übernommen. Eine Kalendereinladung wurde an Ihre E-Mail-Adresse gesendet.

# Guest reschedule confirmation (booking_reschedule_confirm.html)

resched-confirm-subtitle = Sie sind dabei, Ihre Buchung auf einen neuen Termin zu verschieben.

# Base layout chrome (templates/base.html)

base-loader-please-wait = Bitte warten Sie, die neuesten Kalenderdaten werden geladen...
base-language-choose = Wählen Sie Ihre Sprache

# Profile (templates/profile.html)

profile-pick-event-type-invite = Wählen Sie eine Terminart, um einen Termin zu buchen.

# email Subject header, not the ICS SUMMARY (#157).
email-confirm-greeting = Guten Tag { $name },
email-confirm-headline = Ihre Buchung wurde bestätigt!

# Email: guest reminder

email-reminder-headline = Ihr Termin steht bevor.

# Email: guest cancellation

email-cancel-headline-by-host = Ihre Buchung wurde von { $host } storniert.
email-cancel-headline-by-guest = Ihre Buchung wurde storniert.

# Google Meet (templates/event_type_form.html, src/google_meet.rs)
event-type-form-location-google-meet-hint = Bei der Bestätigung wird ein eigener Google-Meet-Link erstellt, im Namen des zugewiesenen Gastgebers. Jeder Gastgeber (Sie oder jedes berechtigte Teammitglied) muss Google Kalender verbunden und einen Schreibkalender ausgewählt haben.
google-meet-prereq-missing = Google Meet setzt voraus, dass jeder Gastgeber Google Kalender verbunden und einen Schreibkalender ausgewählt hat. Es fehlen noch: { $names }. Verbinden Sie sie unter Dashboard → Kalenderquellen.

# Booking action blocked page (templates/booking_action_blocked.html)

booking-blocked-body = Der Gastgeber verlangt mindestens { $minutes } Minuten Vorlaufzeit. Falls Sie nicht teilnehmen können, schreiben Sie bitte direkt an <a href="mailto:{ $host_email }">{ $host_email }</a>.

# Timezone mismatch banner (templates/dashboard_base.html)

tz-banner-text = Die Zeitzone Ihres Browsers ist { $detected }, Ihre Buchungszeitzone ist jedoch auf { $current } eingestellt.

# Dashboard overview (templates/dashboard_overview.html)

overview-avail-banner-body = Ihre Standardarbeitszeiten wurden auf Mo–Fr, 9:00–17:00 Uhr gesetzt. Sie gelten, wenn andere Sie in dynamische Gruppentermine einbeziehen.
overview-getting-started-help = Folgen Sie diesen Schritten, um Buchungen anzunehmen.
overview-action-public-desc = Teilen Sie einen Link — jeder kann ein Zeitfenster wählen und Zeit mit Ihnen buchen.
overview-action-team-desc = Verteilen Sie Buchungen auf Teammitglieder oder finden Sie einen Termin, an dem alle frei sind.
overview-action-team-desc-empty = Legen Sie zuerst ein Team an und richten Sie dann gemeinsame Terminarten ein.
overview-action-private-desc = Erzeugen Sie Einmal-Links für bestimmte Kontakte. Niemand sonst kann buchen.
overview-action-reason-calendar = Verbinden Sie zuerst einen Kalender
overview-action-reason-team-admin = Erfordert ein Team — legen Sie zuerst eines an
overview-action-reason-team-member = Erfordert ein Team — fragen Sie die Administration

# Dashboard bookings (templates/dashboard_bookings.html)

bookings-badge-writeback-failed = nicht in Ihrem Kalender
bookings-empty = Noch keine anstehenden Buchungen.<br>Teilen Sie Ihre { $link }, damit andere Zeit mit Ihnen buchen können.

# Dashboard teams listing (templates/dashboard_teams.html)

teams-empty-admin = { $link }, um mit Ihrem Team zusammenzuarbeiten.
teams-empty-admin-link-label = Legen Sie eines an
teams-empty-member = Teams werden von der Administration angelegt. Bitten Sie sie, eines anzulegen und Sie als Mitglied hinzuzufügen.

# Dashboard invite links (templates/dashboard_internal.html)

invite-links-help = Erzeugen Sie Einmal-Buchungslinks für interne Terminarten. Alle angemeldeten Kolleginnen und Kollegen können hier Links erstellen und teilen.
invite-links-empty-link-label = Legen Sie eine Terminart an

# Dashboard calendar sources (templates/dashboard_sources.html)

sources-no-write-calendar = Kein Schreibkalender ausgewählt. Bestätigte Buchungen bleiben in calrs und werden nicht in diesen Kalender übertragen. Wählen Sie unten einen aus, um das Rückschreiben zu aktivieren.
sources-empty-link-label = Fügen Sie eine hinzu
source-calendars-help = Wählen Sie, welche Kalender von { $name } calrs synchronisiert. Nicht ausgewählte Kalender werden nie abgerufen und blockieren Ihre Verfügbarkeit nicht.
source-calendars-saved = Gespeichert. Neu ausgewählte Kalender werden bei der nächsten Synchronisierung abgerufen; mit „Synchronisieren“ auf der Seite der Quellen holen Sie sie sofort.
source-calendars-empty = Für diese Quelle wurden noch keine Kalender gefunden. Starten Sie eine Synchronisierung, um sie zu finden.

# Dashboard event types listing (templates/dashboard_event_types.html)

event-types-empty-link-label = Legen Sie eine an

# Profile and settings (templates/settings.html)

settings-public-page-label = Ihre öffentliche Buchungsseite
settings-display-name-placeholder = Ihr Name
settings-username-hint = (wird in Ihrer Buchungs-URL verwendet)
settings-username-help = Ihre öffentliche Buchungsseite:
settings-title-help = Wird auf Ihrem öffentlichen Profil und in der Seitenleiste angezeigt.
settings-bio-placeholder = Erzählen Sie ein wenig über sich...
settings-bio-help = Wird auf Ihrer öffentlichen Buchungsseite angezeigt. Unterstützt **fett**, *kursiv*, ~~durchgestrichen~~, `Code` und [Links](url).
settings-booking-email-help = Diese Adresse erscheint auf Ihren öffentlichen Buchungsseiten und in E-Mail-Benachrichtigungen. Leer lassen, um Ihre Anmeldeadresse zu verwenden.
settings-booking-email-warning = Stellen Sie sicher, dass diese Adresse bei Ihrem Mailanbieter existiert. Andernfalls werden Benachrichtigungen nicht zugestellt.
settings-timezone-help = Ihre Verfügbarkeitsregeln und Buchungszeiten werden in dieser Zeitzone berechnet.
settings-language-help = Wählen Sie eine Oberflächensprache oder belassen Sie es bei „Automatisch“, um der Browsereinstellung zu folgen.
settings-public-page-language-help = Sprache Ihres öffentlichen Profils, Ihrer Buchungsseiten und der E-Mails an Ihre Gäste. „Immer meine Sprache“ verwendet die oben gewählte Sprache.
settings-error-public-page-language = Wählen Sie eine gültige Sprache für die öffentlichen Buchungsseiten.
settings-dynamic-group-help = Wenn aktiviert, können andere Benutzer spontane Gruppentermin-URLs erstellen, die Sie einschließen (z. B. { $example }).
settings-lend-resource-help = Wenn eine Buchung eine geteilte Ressource (Demolabor, Besprechungsraum) reservieren muss, in die Ihr Kalenderkonto schreiben darf, erlauben Sie calrs, Ihre gespeicherten Kalenderzugangsdaten dafür zu verwenden.
settings-default-availability-help = Ihre Standardarbeitszeiten. Werden für dynamische Gruppenlinks verwendet, wenn andere Sie zu einem Termin hinzufügen.

# Sign in (templates/auth/login.html)

login-subtitle = Melden Sie sich bei Ihrem calrs-Konto an

# Registration (templates/auth/register.html)

register-subtitle = Registrieren Sie ein neues calrs-Konto
register-name-placeholder = Ihr Name
register-have-account = Sie haben bereits ein Konto? { $link }

# Authentication errors (src/auth.rs)

auth-error-rate-limited = Zu viele Anmeldeversuche. Bitte versuchen Sie es später erneut.
auth-error-email-invalid = Bitte geben Sie eine gültige E-Mail-Adresse ein

# Calendar source test and write-back setup (templates/source_test.html, templates/source_write_setup.html)

source-write-help = Wenn jemand einen Termin mit Ihnen bucht, kann calrs den Eintrag automatisch in Ihrem Kalender anlegen. Wählen Sie, in welchen Kalender Buchungen für { $name } geschrieben werden.

# Date overrides (templates/overrides.html)

overrides-empty = Noch keine Datumsausnahmen.<br>Nutzen Sie das Formular oben, um bestimmte Tage zu sperren (Feiertage, freie Tage) oder abweichende Zeiten festzulegen.

# Public team page (templates/team_profile.html)

team-profile-subtitle = Wählen Sie eine Terminart, um einen Termin zu buchen.

# Availability troubleshoot (templates/troubleshoot.html, src/web/mod.rs)

troubleshoot-empty-link-label = Legen Sie eine an
troubleshoot-subtitle = Sehen Sie nach, warum Zeitfenster für { $title } verfügbar oder blockiert sind
troubleshoot-none-custom-hours = Ausnahme mit abweichenden Zeiten aktiv, aber keine passenden Zeitfenster. Prüfen Sie die Einstellungen der Ausnahme.

# Invite management (templates/invite_form.html)

invites-capped = <strong>Die Eingabe wurde auf { $max } Empfänger pro Absendung begrenzt.</strong> Senden Sie den Rest in einem weiteren Durchgang.
invites-failed-hint = — Einzelheiten finden Sie in den Serverprotokollen.
invites-quick-link-help = Erzeugen Sie einen Einmal-Link und kopieren Sie ihn in die Zwischenablage.
invites-message-placeholder = Ich freue mich darauf, Ihnen eine Demo zu zeigen...
invites-empty = Noch keine Einladungen gesendet. Nutzen Sie das Formular oben, um jemandem einen Buchungslink zu schicken.

# Calendar source form (templates/source_form.html)

source-form-subtitle-edit = Aktualisieren Sie die Verbindung. Lassen Sie das Passwort leer, um das bestehende beizubehalten. Führen Sie nach einer Änderung von URL oder Benutzername eine Synchronisierung durch, um die Kalenderliste zu aktualisieren.
source-form-subtitle-add = Verbinden Sie einen CalDAV-Server oder Microsoft Exchange (EWS), damit calrs bei Buchungen die Verfügbarkeit prüfen kann.
source-form-google-unavailable = Google Kalender ist nicht verfügbar. Wenden Sie sich an Ihre Administration.
source-form-skip-test-help = Nutzen Sie dies, wenn der Test hängt (kommt bei manchen BlueMind-/Zimbra-Installationen vor). Sie können die Verbindung später testen.
source-form-help-google-configured = Klicken Sie auf die Schaltfläche unten, um calrs den Zugriff auf Ihren Google Kalender zu erlauben.
source-form-help-google-unconfigured = Die Google-Kalender-Integration ist noch nicht eingerichtet. Bitten Sie Ihre Administration, im Administrationsbereich OAuth2-Zugangsdaten für Google zu hinterlegen.

# Calendar source form: provider help (templates/source_form.html)

source-form-help-bluemind = <strong>BlueMind</strong> — Verwenden Sie den DAV-Endpunkt Ihres BlueMind-Servers.<br> Üblicherweise: <code>https://mail.yourcompany.com/dav/</code><br> Der Benutzername ist Ihre <strong>E-Mail-Adresse</strong> (z. B. <code>alice@yourcompany.com</code>), nicht nur der Anmeldename.<br> Wenn der Verbindungstest hängt, aktivieren Sie „Verbindungstest überspringen“ und synchronisieren Sie direkt.
source-form-help-nextcloud = <strong>Nextcloud</strong> — Verwenden Sie die WebDAV-Wurzel, nicht die URL eines einzelnen Kalenders.<br> Üblicherweise: <code>https://cloud.example.com/remote.php/dav</code>
source-form-help-fastmail = <strong>Fastmail</strong> — Verwenden Sie Ihre vollständige Adresse im URL-Pfad.<br> Beispiel: <code>https://caldav.fastmail.com/dav/calendars/user/you@fastmail.com/</code><br> Verwenden Sie ein App-Passwort (Settings &rarr; Privacy &amp; Security &rarr; Integrations).
source-form-help-icloud = <strong>iCloud</strong> — Verwenden Sie <code>https://caldav.icloud.com/</code><br> Sie benötigen ein App-Passwort von <a href="https://appleid.apple.com" target="_blank" style="color: var(--accent);">appleid.apple.com</a> (Sicherheit &rarr; App-spezifische Passwörter).
source-form-help-zimbra = <strong>Zimbra</strong> — Verwenden Sie den DAV-Endpunkt Ihres Zimbra-Servers.<br> Üblicherweise: <code>https://mail.example.com/dav/</code>
source-form-help-sogo = <strong>SOGo</strong> — Verwenden Sie den DAV-Endpunkt von SOGo.<br> Üblicherweise: <code>https://mail.example.com/SOGo/dav/</code>
source-form-help-radicale = <strong>Radicale</strong> — Verwenden Sie die Wurzel-URL des Servers.<br> Üblicherweise: <code>https://cal.example.com/</code>
source-form-help-baikal = <strong>Baikal</strong>: Verwenden Sie den DAV-Endpunkt Ihres Baikal-Servers.<br> Üblicherweise: <code>https://cal.example.com/dav.php/</code>
source-form-help-exchange = <strong>Microsoft Exchange (EWS)</strong>. Verwenden Sie den SOAP-Endpunkt:<br> <code>https://mail.example.com/EWS/Exchange.asmx</code><br> Der Benutzername ist die Postfachadresse; das Passwort muss HTTP Basic über TLS zulassen (bei deaktiviertem Basic im Tenant an einem Dienstpostfach aktivieren).<br> Wählen Sie außerdem oben im Backend-Menü <strong>Microsoft Exchange (EWS)</strong>.
source-form-help-other = Geben Sie die <strong>DAV-Wurzel-URL</strong> Ihres CalDAV-Servers an — nicht die eines einzelnen Kalenders oder einen öffentlichen Link.<br> calrs findet Ihre Kalender automatisch per PROPFIND (RFC 4791).

# Team creation (templates/team_form.html)

team-form-members-help = Sie werden automatisch als Teamadministrator hinzugefügt. Fügen Sie einzelne Benutzer hinzu oder verknüpfen Sie OIDC-Gruppen.
team-form-you = (Sie)

# Team settings (templates/team_settings.html)

team-settings-invite-link-help = Teilen Sie diesen Link, um Zugang zur Buchungsseite dieses privaten Teams zu geben.
team-settings-description-placeholder = Beschreiben Sie dieses Team...
team-settings-members-help = Verwalten Sie die Mitglieder dieses Teams. Fügen Sie einzelne Benutzer hinzu oder verknüpfen Sie OIDC-Gruppen für die automatische Synchronisierung.

# Event type form (templates/event_type_form.html)

etf-required-members-help = Alle angehakten Mitglieder müssen frei sein, damit ein Zeitfenster angeboten wird. Nehmen Sie den Haken bei Mitgliedern weg, die ausgeschlossen werden sollen (ihre Verfügbarkeit wird dann ignoriert).
etf-timezone-help = Die Zeiten unten werden in dieser Zeitzone ausgelegt. Wählen Sie bei Team-Terminarten die Arbeitszeitzone des Teams (nicht zwingend die der erstellenden Person).
etf-reset-default-title = Diese Zeiten durch die Standardverfügbarkeit aus Ihrem Profil ersetzen
etf-availability-prefilled = Vorbelegt aus Ihrer { $link }. Sie können sie hier für diese Terminart überschreiben.
etf-requires-confirmation-help = Buchungen bleiben ausstehend, bis Sie sie im Dashboard bestätigen.
etf-conflict-calendars-help = Wählen Sie, welche Kalender auf Konflikte geprüft werden. Ohne Auswahl werden alle Kalender verwendet.
etf-no-resources = Noch keine geteilten Ressourcen eingerichtet. Legen Sie im { $link } eine an (Demolabor, Besprechungsraum), um sie hier zu verlangen.
etf-visibility-public = Öffentlich — auf Ihrem Profil sichtbar
etf-reminder-help = Vor dem Termin eine Erinnerungs-E-Mail an Sie und Ihren Gast senden.
etf-dynamic-group-help = Erstellen Sie einen spontanen Terminlink, der die Verfügbarkeit von Ihnen und anderen Benutzern prüft.

# Literal braces are escaped as {"{"} because Fluent reads a bare { as a placeable.

etf-preset-public = Sie erstellen eine <strong>öffentliche</strong> Terminart &mdash; jeder mit dem Link kann buchen.
etf-preset-private = Sie erstellen eine <strong>private</strong> Terminart &mdash; nur eingeladene Personen können buchen.
etf-preset-internal = Sie erstellen eine <strong>interne</strong> Terminart &mdash; alle Kolleginnen und Kollegen können den Buchungslink teilen.
etf-preset-team = Sie erstellen eine <strong>Team-Terminart</strong> &mdash; Buchungen werden auf die Teammitglieder verteilt.
etf-pattern-random-warning = Dieses Muster enthält keinen <code>{"{"}random{"}"}</code>-Platzhalter. Zwei Buchungen dieser Terminart am selben Tag teilen sich denselben Raum, und der zweite Gast kann in das Gespräch des ersten geraten. Verwenden Sie feste Räume nur, wenn Sie genau das möchten.

# Admin panel (templates/admin.html)

admin-error-last-admin = Das ist der letzte aktive Admin. Stufen Sie zuerst einen anderen Benutzer hoch oder aktivieren Sie einen anderen Admin.
admin-theme-help = Wählen Sie ein Farbdesign für alle Seiten. Die Umschaltung zwischen Hell und Dunkel ist davon unabhängig — die Designs passen sich beiden Modi an.
admin-theme-custom-desc = Ihre Farben
admin-no-groups = Noch keine Gruppen synchronisiert. Gruppen werden automatisch von Ihrem OIDC-Anbieter übernommen.
admin-private-hosts-help = Kommagetrennte Hostnamen, die für CalDAV-/EWS-Quellen auf private oder reservierte IP-Adressen zeigen dürfen (Ausnahme vom SSRF-Schutz). Tragen Sie nur Hosts ein, die Sie kontrollieren (etwa einen Kalenderserver im selben Docker-Netz). Leer lassen, um den Schutz für alle Hosts aktiv zu halten.
admin-unset-env = Entfernen Sie die Umgebungsvariable, um dies hier zu bearbeiten.
admin-wasm-url-help = Von dort lädt das Widget seinen Löser. Tragen Sie die WASM-Datei Ihres Cap-Servers ein, damit Buchungsseiten cdn.jsdelivr.net nicht mehr kontaktieren. Leer lassen für den Standardwert.
admin-captcha-disable-help = Lassen Sie Instanz-URL, Site-Key und Secret leer, um das Captcha auf Buchungsseiten zu deaktivieren.
admin-jitsi-help = Wenn der Ort einer Terminart auf „Jitsi (automatisch erzeugter Raum)“ gesetzt ist, baut calrs für jede Buchung eine frische Raum-URL, indem es das Muster unten an Ihre Jitsi-Basis-URL anhängt. Ein externer API-Aufruf ist nicht nötig.
admin-jitsi-disable-help = Lassen Sie die Basis-URL leer, um die automatische Jitsi-Erzeugung zu deaktivieren.
admin-webhook-disable-help = Lassen Sie die URL leer, um den Termin-Webhook zu deaktivieren.
admin-smtp-test-failed = Die Test-E-Mail konnte nicht gesendet werden. Prüfen Sie die Serverprotokolle und Ihre SMTP-Einstellungen.
admin-send-test-email-hint = (standardmäßig Ihre Kontoadresse)
admin-sms-captcha-warning = Das Buchungsformular ist öffentlich und die Empfängernummer kommt vom Gast. SMS ohne Captcha sind daher ein offenes Relay, das jemand anderes auf Ihre Kosten nutzen kann. Richten Sie oben das Captcha ein und beschränken Sie die Zielländer in den Einstellungen Ihres Gateways.

# Admin panel: strings carrying markup or literal braces (templates/admin.html)

admin-google-help = Um die Google-Kalender-Integration zu aktivieren, erstellen Sie OAuth2-Zugangsdaten in der <a href="https://console.cloud.google.com/apis/credentials" target="_blank" style="color: var(--accent);">Google Cloud Console</a>. Aktivieren Sie die <strong>CalDAV API</strong> (Lesen belegter Zeiten und Zurückschreiben von Buchungen) und die <strong>Google Calendar API</strong> (Google-Meet-Links) und tragen Sie dann { $redirect_uri } als zugelassene Weiterleitungs-URI ein.
admin-room-pattern-warning = Ohne <code>{"{"}random{"}"}</code> ist der Raumname vorhersehbar: Zwei Gäste, die dieselbe Terminart am selben Tag buchen, landen im selben Raum und sehen das Gespräch des jeweils anderen. Feste Räume sind zulässig (etwa ein persönlicher Raum je Gastgeber), aktivieren Sie das aber nur, wenn Sie den Kompromiss verstehen.
admin-tls-none-warning = Wählen Sie <strong>Keines</strong> nur für ein Relay auf diesem Rechner, das kein STARTTLS anbietet oder dessen Zertifikat selbstsigniert ist. E-Mails und etwaige Zugangsdaten gehen dann unverschlüsselt über die Leitung.
admin-smtp-env-error-help = Korrigieren Sie die Umgebungsvariablen <code>CALRS_SMTP_*</code> oder entfernen Sie sie, um SMTP hier über die Datenbank zu verwalten.
admin-smtp-env-managed = Verwaltet über <strong>Umgebungsvariablen</strong> (haben Vorrang vor der Datenbank). Ändern Sie die Variablen <code>CALRS_SMTP_*</code> oder entfernen Sie sie, um SMTP hier zu verwalten.
admin-smtp-env-help = Alternativ über Umgebungsvariablen konfigurieren (sie haben Vorrang): <code>CALRS_SMTP_HOST</code>, <code>CALRS_SMTP_PORT</code>, <code>CALRS_SMTP_TLS_MODE</code> (<code>starttls</code>, <code>tls</code> oder <code>none</code>), <code>CALRS_SMTP_USERNAME</code>, <code>CALRS_SMTP_PASSWORD</code>, <code>CALRS_SMTP_FROM_EMAIL</code>, <code>CALRS_SMTP_FROM_NAME</code>. Nur <code>CALRS_SMTP_HOST</code> und <code>CALRS_SMTP_FROM_EMAIL</code> sind erforderlich; lassen Sie Benutzername und Passwort weg, um ohne Authentifizierung über einen lokalen MTA zu relayen.
admin-sms-env-error-help = Korrigieren Sie die Umgebungsvariablen <code>CALRS_SMS_*</code> oder entfernen Sie sie, um SMS hier über die Datenbank zu verwalten.
admin-sms-env-managed = Verwaltet über <strong>Umgebungsvariablen</strong> (haben Vorrang vor der Datenbank). Ändern Sie die Variablen <code>CALRS_SMS_*</code> oder entfernen Sie sie, um SMS hier zu verwalten.
admin-sms-trial-warning = <strong>Der Twilio-Testmodus ist aktiv</strong> (<code>CALRS_SMS_TWILIO_TRIAL</code>). Gäste erhalten Twilios vordefinierte Vorlage <code>sms_appointment_reminders</code> statt der echten Nachricht, und nur in Ihrer Twilio-Konsole verifizierte Nummern werden erreicht. Das ist eine Testhilfe für Testkonten. Entfernen Sie die Variable, bevor Sie Buchungen annehmen.

# Calendar source form: backend picker (templates/source_form.html)

source-form-backend-help = Wählen Sie das Protokoll, das Ihr Server spricht. EWS richtet sich an lokal betriebene Exchange-Server 2019/2016/2013.

admin-sms-going-live = <strong>Vor dem Produktivbetrieb:</strong> Beschränken Sie die Zielländer in Ihrem Gateway (bei Twilio heißt das Geo Permissions), halten Sie das Konto auf Guthabenbasis ohne automatische Aufladung, und lassen Sie das Captcha aktiv. Diese drei Maßnahmen begrenzen zusammen, was ein SMS-Pumping-Versuch kosten kann.

# Host-side form validation errors (src/web/mod.rs)

form-error-slug-reserved = Dieses Kürzel ist reserviert. Bitte wählen Sie ein anderes.
form-error-event-type-horizon = Der Buchungshorizont muss eine ganze Zahl von Tagen ab 0 sein. Lassen Sie das Feld leer für keine Begrenzung.
form-error-not-team-admin = Sie sind kein Teamadministrator dieses Teams.
form-error-no-account = Kein Planungsprofil gefunden. Bitte wenden Sie sich an die Administration.
form-error-connection-failed = Verbindung fehlgeschlagen: { $error }. Prüfen Sie URL und Zugangsdaten, oder aktivieren Sie „Verbindungstest überspringen“, um trotzdem zu speichern.

# Profile settings validation and flash messages (src/web/mod.rs)

settings-error-booking-email = Bitte geben Sie eine gültige Buchungs-E-Mail-Adresse ein.

# Host-facing error responses (src/web/mod.rs)

error-team-not-found-or-not-admin = Team nicht gefunden, oder Sie sind kein Teamadministrator.
error-oauth-invalid-state = Ungültiger State-Parameter. Bitte versuchen Sie es erneut.

# Guest booking-flow errors (src/web/mod.rs)

error-too-many-bookings = Zu viele Buchungsversuche. Bitte versuchen Sie es in ein paar Minuten erneut.
error-too-many-requests = Zu viele Anfragen. Bitte versuchen Sie es später erneut.

# Booking action error page: bodies

bae-body-go-back = Bitte gehen Sie zurück und versuchen Sie es erneut.
bae-body-unavailable = Der Gastgeber nimmt für dieses Datum keine weiteren Buchungen an. Bitte wählen Sie ein anderes Datum oder schauen Sie später noch einmal vorbei.

# Booking form validation (src/web/mod.rs)

validate-email-invalid = Bitte geben Sie eine gültige E-Mail-Adresse ein.

# Additional guests and dynamic group links (src/web/mod.rs)

guests-too-many =
    { $max ->
        [one] Sie können höchstens einen zusätzlichen Gast hinzufügen.
       *[other] Sie können höchstens { $max } zusätzliche Gäste hinzufügen.
    }

# Pending requests: slot hold, expiry and approval re-check (#234)

sms-expired = Anfrage abgelaufen: { $event }, { $date } um { $time } ({ $tz }). Keine Antwort erhalten, bitte wählen Sie einen anderen Termin.
email-pending-expired-headline = Ihre Buchungsanfrage ist abgelaufen: Der Gastgeber hat nicht rechtzeitig geantwortet. Bitte wählen Sie einen anderen Termin.
etf-pending-blocks-slot-help = Solange eine Anfrage auf Ihre Antwort wartet, kann niemand sonst diese Zeit buchen, weder bei dieser noch bei Ihren anderen Terminarten.
etf-pending-expiry-help = Eine Anfrage, die Sie bis dahin nicht beantwortet haben, wird automatisch abgelehnt und der Gast wird gebeten, einen anderen Termin zu wählen. Sie endet früher, wenn der Termin zu nah ist, als dass jemand anderes ihn noch buchen könnte. Zur Halbzeit bekommen Sie eine Erinnerung.

# Host, watcher and remaining guest emails (src/email.rs, #209)

email-host-approval-dashboard = Melden Sie sich in Ihrem Dashboard an, um diese Buchung zu bestätigen oder abzulehnen.
email-host-new-message = { $guest } hat einen Termin bei Ihnen gebucht.
email-host-confirmed-message = Sie haben die Buchung mit { $guest } genehmigt.
email-host-confirmed-footer = Der Termin wurde Ihrem Kalender hinzugefügt.
email-host-sync-failure-heading = Ihr Kalender wurde nicht aktualisiert
email-host-sync-failure-intro = Diese Buchung wurde verschoben, aber calrs konnte den Termin in Ihrem Google Kalender nicht aktualisieren. Der Gast hat die neue Uhrzeit, Ihr Kalender zeigt noch die alte.
email-host-sync-failure-action = Öffnen Sie den Termin in Google Kalender und verschieben Sie ihn auf die oben genannte Uhrzeit. Verschieben Sie den vorhandenen Termin, statt ihn neu anzulegen, sonst funktioniert der Google-Meet-Link nicht mehr, den der Gast bereits hat.
email-writeback-failure-subject = Handlungsbedarf: { $event } ({ $date }) fehlt in Ihrem Kalender
email-writeback-failure-heading = Eine Buchung fehlt in Ihrem Kalender
email-writeback-failure-intro = calrs konnte diese Buchung auch nach mehreren Versuchen nicht in Ihren Kalender eintragen. Die Buchung ist bestätigt und der Gast hat die Einladung.
email-writeback-failure-action = Tragen Sie den Termin selbst in Ihren Kalender ein, damit Sie ihn nicht verpassen, und prüfen Sie die Kalenderquelle in calrs. calrs versucht es noch einige Stunden weiter; Sie bekommen diese E-Mail für diese Buchung nicht noch einmal.
email-source-failure-intro = calrs kann diese Kalenderquelle seit über einer Stunde nicht synchronisieren. Ihre Buchungsseiten bieten weiter Termine an, die aber auf der letzten erfolgreichen Synchronisierung beruhen: Seitdem hinzugefügte oder verschobene Termine werden nicht berücksichtigt.
email-source-failure-action = Prüfen Sie die Quelle auf der Seite Kalenderquellen: Meist ist ein abgelaufenes Passwort oder ein entzogener Zugriff die Ursache. Von dieser Quelle hören Sie erst wieder, wenn sie sich erholt und danach erneut ausfällt.
email-host-reminder-message = Erinnerung: Sie haben bald eine Buchung mit { $guest }.
email-host-cancel-message-by-host = Sie haben diese Buchung storniert.
email-host-approval-message = { $guest } möchte einen Termin bei Ihnen buchen.
email-host-approval-reminder-subject = Erinnerung: { $guest } wartet auf Ihre Antwort ({ $event }, { $date })
email-host-approval-reminder-message = { $guest } wartet noch auf Ihre Antwort. Wenn Sie nicht antworten, wird die Anfrage am { $deadline } automatisch abgelehnt.
email-host-approval-footer = Sie können dies auch in Ihrem Dashboard verwalten.
email-host-expired-footer = Wie lange Anfragen warten, können Sie in den Einstellungen des Termintyps ändern.
email-watcher-claim-message = Eine neue Buchung kann übernommen werden. Klicken Sie unten, um als Teilnehmer dazuzukommen.
email-watcher-claim-footer = Sie können sie auch in Ihrem Dashboard übernehmen.
email-claim-confirmed-footer = Sie werden als Teilnehmer zu diesem Termin hinzugefügt.
email-attendee-greeting = Guten Tag,
email-attendee-message = Sie wurden als Teilnehmer zu einer Buchung hinzugefügt.
email-attendee-rescheduled-message = Eine Buchung, an der Sie teilnehmen, wurde auf einen neuen Termin verschoben.
email-attendee-cancelled-message = Eine Buchung, an der Sie teilnehmen, wurde storniert.
email-original-booking-cancelled = Ihre ursprüngliche Buchung ist ebenfalls storniert.
email-pick-new-time-subject = Verschiebung: { $event }, bitte wählen Sie einen neuen Termin
email-pick-new-time-message = { $host } muss Ihre Buchung verschieben. Bitte wählen Sie einen neuen Termin.
email-rescheduled-message = Ihre Buchung wurde von { $host } verschoben.
email-reschedule-approved-message = { $host } hat den neuen Termin Ihrer Buchung bestätigt.
email-invite-subject = { $host } hat Sie eingeladen, einen Termin zu buchen: { $event }
email-invite-message = { $host } hat Sie eingeladen, einen Termin zu buchen: { $event }
email-invite-link = Klicken Sie auf den Link unten, um einen Termin zu wählen:

# Impersonation banner (templates/base.html), embed snippet generator
# (templates/embed.html), admin notices, Google and team errors (src/web/*.rs)

base-impersonating = Sie handeln als { $name }
embed-subtitle = Erzeugen Sie ein Code-Snippet, um diese Terminart auf Ihrer eigenen Website einzubetten.
embed-warn-internal = Achtung: Diese Terminart ist <strong>intern</strong>. Besucher der eingebetteten Seite brauchen einen gültigen Einladungslink, um zu buchen — das erzeugte Snippet enthält keinen. Stellen Sie die Sichtbarkeit auf öffentlich, wenn Sie direkt einbetten möchten.
embed-warn-private = Achtung: Diese Terminart ist <strong>privat</strong>. Besucher der eingebetteten Seite brauchen einen gültigen Einladungslink, um zu buchen — das erzeugte Snippet enthält keinen. Stellen Sie die Sichtbarkeit auf öffentlich, wenn Sie direkt einbetten möchten.
embed-warn-private-team = Achtung: Dieses Team ist <strong>privat</strong>. Besucher der eingebetteten Seite können ohne den Einladungslink des Teams nicht buchen — das erzeugte Snippet enthält keinen. Stellen Sie das Team auf öffentlich, wenn Sie direkt einbetten möchten.
embed-need-username = Sie müssen einen <strong>Benutzernamen</strong> festlegen, bevor Sie diese Terminart einbetten können — der Einbettungslink lautet { $path }. { $link } und kommen Sie dann zurück.
embed-need-username-link = Legen Sie in den Einstellungen einen Benutzernamen fest
embed-tab-element = Element anklicken
embed-element-help = Fügen Sie das Snippet einmal in Ihre Website ein und ergänzen Sie dann die angezeigten data-Attribute an jedem klickbaren Element (Button, Link, div), das das Buchungsfenster öffnen soll.
embed-preview-floating-hint = Klicken Sie auf den Button, um das Buchungsfenster zu simulieren
embed-preview-element-hint = Demo-Auslöser — fügen Sie auf Ihrer Website die data-Attribute aus dem Snippet einem beliebigen Element hinzu. Klicken Sie auf einen davon, um das Fenster auszuprobieren:
embed-preview-text-link = oder klicken Sie auf diesen Textlink
embed-snippet-attr-comment = Fügen Sie diese Attribute einem beliebigen klickbaren Element Ihrer Seite hinzu:
admin-logo-refused = Das Logo wurde nicht gespeichert. Laden Sie ein PNG-, JPEG-, GIF- oder WebP-Bild mit höchstens 2 MB hoch.
admin-favicon-refused = Das Favicon wurde nicht gespeichert. Laden Sie ein PNG-, JPEG-, GIF- oder WebP-Bild mit höchstens 512 KB hoch.
admin-smtp-from-invalid = Geben Sie eine gültige Absender-E-Mail-Adresse ein.
admin-sms-switch-needs-secret = Beim Wechsel des SMS-Gateways müssen Sie die Zugangsdaten dieses Gateways eingeben.
admin-sms-test-body = calrs-Testnachricht. Ihr SMS-Gateway funktioniert.
google-error-no-base-url = Die öffentliche Basis-URL ist nicht konfiguriert. Legen Sie sie über die Umgebungsvariable CALRS_BASE_URL oder im Admin-Bereich unter Systemeinstellungen fest, bevor Sie OAuth2-Abläufe nutzen.
google-error-not-configured = Die Google-Kalender-Integration ist nicht eingerichtet. Bitten Sie Ihre Administratorin oder Ihren Administrator, die Google-OAuth2-Zugangsdaten im Admin-Bereich einzutragen.
team-event-type-not-team-admin = Sie sind in keinem Team Admin.

# User and team avatar uploads (#307)
avatar-upload-refused = Das Bild wurde nicht gespeichert. Laden Sie ein PNG-, JPEG-, GIF- oder WebP-Bild mit höchstens 2 MB hoch.
avatar-upload-save-failed = Das Bild konnte auf dem Server nicht gespeichert werden. Versuchen Sie es später erneut oder bitten Sie Ihren Administrator, das Server-Log zu prüfen.

## Local multi-factor authentication

mfa-invalid = Die Überprüfung ist fehlgeschlagen. Prüfen Sie Ihr Passwort oder Ihren Code. Jeder Code kann nur einmal verwendet werden; warten Sie bei Bedarf auf einen neuen. Warten Sie nach zu vielen Versuchen 15 Minuten. Starten Sie neu, wenn die Einrichtung abgelaufen ist.
mfa-recovery-help = Bewahren Sie diese Wiederherstellungscodes sicher auf. Sie werden nur einmal angezeigt. Jeder Code ersetzt einmalig einen Authenticator-Code. Neue Wiederherstellungscodes ersetzen alle bisherigen Codes.
mfa-required-help = Ihr Administrator verlangt Zwei-Faktor-Authentifizierung für lokale Konten. Sie können sie nicht deaktivieren.
mfa-changes-help = Änderungen erfordern Ihr Passwort. Zum Deaktivieren oder Ersetzen der Wiederherstellungscodes brauchen Sie auch einen Code; Ihre anderen Sitzungen werden dabei abgemeldet.
mfa-setup-help = Scannen Sie diesen QR-Code mit Ihrer Authenticator-App oder geben Sie den Schlüssel unten manuell ein. Geben Sie einen sechsstelligen Code ein, um die Einrichtung abzuschließen. Die Einrichtung läuft nach 10 Minuten ab.
mfa-login-help = Geben Sie einen Code aus Ihrer Authenticator-App oder einen gespeicherten Wiederherstellungscode ein. Diese Anmeldung läuft nach 10 Minuten ab.

# Bookable date range per event type (#238)
event-type-form-bookable-range-help = Nur an diesen Tagen kann gebucht werden. Leer lassen für kein Limit. Für andere Zeiten an einzelnen Tagen nutzen Sie Ausnahmen.
form-error-bookable-range-invalid = Geben Sie die Daten im Format JJJJ-MM-TT ein.
bookings-error-no-bookable-dates = Dieser Termintyp hat keine buchbaren Tage mehr, daher könnte der Gast keine neue Zeit wählen. Verlängern Sie zuerst seinen Buchungszeitraum.

## Booking directory and personal favorites

directory-help = Finden Sie den Terminlink eines Kollegen oder Teams. Markieren Sie häufig geteilte Termine als Favoriten, um sie auf Ihrem Dashboard zu sehen.
directory-error = Diese Aktion konnte nicht abgeschlossen werden. Laden Sie die Seite neu und versuchen Sie es erneut.
directory-manual-copy = Kopieren Sie diesen Link manuell:
directory-empty = Keine Termine gefunden. Versuchen Sie eine andere Suche.
directory-favorites-empty = Keine Favoriten gefunden. Markieren Sie Termine im Verzeichnis als Favoriten, um sie hier zu sehen.

## ilia fork: TON payments (docs/ton.md)

ton-bae-body-unavailable = Die Zahlung ist vorübergehend nicht verfügbar, daher kann dieser Termin gerade nicht gebucht werden. Versuchen Sie es in ein paar Minuten noch einmal.
ton-bae-body-too-late = Bis zu diesem Termin bleibt zu wenig Zeit für die Zahlung. Wählen Sie bitte einen späteren Termin.
ton-bae-body-already-open = Sie haben mit dieser E-Mail-Adresse bereits eine unbezahlte Buchung. Wir können Ihnen ihre Links schicken: Dort können Sie sie bezahlen oder stornieren, um den Termin freizugeben, und dann neu buchen.
ton-bae-body-unpaid = Diese Buchung ist nicht bezahlt und kann daher nicht bestätigt werden. Um sie kostenlos zu bestätigen, nutzen Sie „Ohne Zahlung bestätigen“ auf der Buchungsseite Ihres Dashboards.
ton-bae-body-reschedule-unpaid = Diese Buchung ist nicht bezahlt und kann daher nicht verschoben werden. Bezahlen Sie sie oder stornieren Sie sie und buchen Sie einen anderen Termin.
ton-pay-held-until = Der Termin ist bis { $time } ({ $zone }) für Sie reserviert.
ton-pay-scan = Oder scannen Sie diesen Code mit Ihrer Wallet-App.
ton-pay-warn-comment = Der Kommentar ist Pflicht: Daran erkennen wir Ihre Zahlung.
ton-pay-warn-encrypt = Verschlüsseln Sie den Kommentar nicht.
ton-pay-warn-token = Zahlen Sie in TON, nicht mit Tokens.
ton-pay-warn-exchange = Sie zahlen von einer Börse? Tragen Sie den Code oben ins Memo-Feld (Kommentar) ein.
ton-pay-warn-deadline = Senden Sie die Zahlung vor { $time }. Eine später gesendete Zahlung wird womöglich nicht berücksichtigt.
ton-pay-emailed = Wir haben diesen Link an die E-Mail-Adresse Ihrer Buchung gesendet.
ton-pay-email-unavailable = Der Link konnte nicht gesendet werden. Lassen Sie diese Seite offen oder speichern Sie ihre Adresse.
ton-pay-checking-now = Wir prüfen Ihre Zahlung. Diese Seite aktualisiert sich von selbst.
ton-pay-state-waiting = Wir warten auf Ihre Zahlung.
ton-pay-state-seen = Ihre Zahlung ist im Netzwerk sichtbar. Wir warten, bis sie endgültig ist, was meist ein paar Sekunden dauert.
ton-pay-state-partial = Wir haben { $paid } TON erhalten. Bitte zahlen Sie die restlichen { $rest } TON mit demselben Kommentar.
ton-pay-state-checking = Die Zahlungsfrist ist abgelaufen. Falls Sie die Zahlung schon gesendet haben, prüfen wir sie. Diese Seite aktualisiert sich von selbst.
ton-pay-state-paid = Zahlung erhalten. Ihre Buchung wird bestätigt …
ton-pay-state-confirmed = Bezahlt. Ihre Buchung ist bestätigt, die Bestätigungs-E-Mail ist unterwegs.
ton-pay-state-busy = Zahlung erhalten, aber dieser Termin ist nicht mehr frei. Wir melden uns bei Ihnen.
ton-pay-state-expired = Die Zahlungsfrist ist abgelaufen und der Termin wurde freigegeben. Falls Sie trotzdem gezahlt haben, schreiben Sie uns von der E-Mail-Adresse Ihrer Buchung, dann erstatten wir den Betrag.
ton-pay-state-late = Ihre Zahlung kam nach Ablauf der Frist, daher wurde die Buchung nicht bestätigt. Wir melden uns wegen der Erstattung. Sie können uns auch von der E-Mail-Adresse Ihrer Buchung schreiben.
ton-pay-state-revoked = Diese Zahlungsadresse wird nicht mehr verwendet. Senden Sie nichts an sie und buchen Sie bitte erneut.
ton-exit-body = Der Termin ist bis { $time } ({ $zone }) für Sie reserviert. Schließen Sie die Zahlung ab, um Ihre Buchung zu bestätigen.
ton-dash-error-unpaid = Diese Buchung ist nicht bezahlt und kann daher weder bestätigt noch verschoben werden. Nutzen Sie „Ohne Zahlung bestätigen“, um sie kostenlos zu bestätigen, oder lehnen Sie sie ab.
ton-waive-body = Diese Buchung ist NICHT bezahlt ({ $amount } TON, fällig bis { $time }). Wenn Sie sie bestätigen, bekommt der Gast den Termin kostenlos, und eine später eingehende Zahlung muss von Hand erstattet werden.
ton-email-link-subject = Schließen Sie Ihre Buchung ab: { $event }
ton-email-link-message = Sie haben begonnen, { $event } mit { $host } zu buchen. Schließen Sie Ihre Buchung auf der Seite unten ab, bevor die Zeit abläuft.
ton-email-link-line = Schließen Sie Ihre Buchung ab: { $url }
ton-email-expired-message = Die Zahlung für { $event } ist nicht rechtzeitig eingegangen, daher wurde der Termin freigegeben. Sie können gern einen anderen Termin wählen.
ton-email-refund-rule = Erstattungen erfolgen von Hand aus Ihrer Wallet: an die Absenderadresse, wenn sie eine Wallet ist; sonst nur an eine Adresse, die der Gast von der E-Mail-Adresse seiner Buchung anfragt, in einer Nachricht mit seinem Zahlungs- oder Stornolink. Buchungscode, Betrag und Transaktion sind öffentlich und beweisen nichts.
ton-email-alert-heading = Eine Zahlung braucht Ihre Aufmerksamkeit
ton-alert-overpaid-body = Eine Zahlung ist für eine bereits bezahlte Buchung eingegangen. Erstatten Sie sie, wenn es sich lohnt.
ton-alert-late-body = Eine Zahlung ist nach ihrer Frist eingegangen. Die Buchung war bereits freigegeben und kann nicht mehr bestätigt werden. Erstatten Sie die Zahlung oder bitten Sie den Gast, neu zu buchen, und bestätigen Sie die neue Buchung ohne Zahlung.
ton-alert-unmatched-body = Eine Zahlung ohne bekannten Buchungscode ist eingegangen. Finden Sie die zugehörige Buchung oder erstatten Sie sie.
ton-alert-failed-body = Eine Überweisung an die Zahlungsadresse ist im Netzwerk fehlgeschlagen. Das Geld liegt womöglich noch in Ihrer Wallet: Prüfen Sie die Transaktion im Explorer.
ton-alert-not-text-body = Eine Überweisung mit verschlüsseltem Kommentar, einem Token oder anderem Inhalt ist eingegangen. Sie wird nie automatisch angerechnet: Prüfen Sie sie von Hand.
ton-alert-other-body = Eine Überweisung konnte nicht automatisch verarbeitet werden. Prüfen Sie sie in den Zahlungsdaten.
ton-email-problem-heading = Eine bezahlte Buchung braucht Ihre Aufmerksamkeit
ton-problem-host-busy = Der Gast hat bezahlt, aber der Termin ist nicht mehr frei (ein Kalendereintrag oder eine andere Buchung). Der Termin bleibt bis zum Beginn reserviert. Bitten Sie den Gast um einen neuen Termin, bestätigen Sie die Buchung von Hand oder lehnen Sie sie ab und erstatten Sie die Zahlung.
ton-problem-partial = Nur ein Teil des Betrags wurde bezahlt, und die Buchung ist danach abgelaufen, wurde storniert oder abgelehnt oder ohne Zahlung bestätigt. Erstatten Sie den erhaltenen Betrag, sofern Sie mit dem Gast nichts anderes vereinbart haben.
ton-problem-overpaid = Die Buchung ist bezahlt und bestätigt, aber mit mehr als dem fälligen Betrag. Erstatten Sie den Überschuss, wenn es sich lohnt.
ton-problem-refund-due = Die Buchung wurde bezahlt und danach storniert oder abgelehnt. Erstatten Sie die Zahlung.
ton-problem-confirmed-unpaid = Eine unbezahlte Buchung ist bestätigt. Das darf nie passieren: Prüfen Sie, wie sie bestätigt wurde.
ton-health-auth-failed = Der Indexer hat den API-Schlüssel abgelehnt (falsch oder für das andere Netzwerk). Prüfen Sie CALRS_TON_API_KEY.
ton-health-watcher-crashed = Die Zahlungsüberwachung ist mehrmals abgestürzt und startet immer wieder neu. Prüfen Sie das Serverprotokoll.
ton-health-rate-unavailable = Es gibt keinen brauchbaren TON-Kurs (Quellen fehlen, weichen ab oder liegen außerhalb Ihres Korridors). Kostenpflichtige Buchungen werden abgelehnt, bis es einen gibt.
ton-health-other = Die Zahlungsüberwachung hat ein Problem gemeldet. Prüfen Sie das Serverprotokoll.

# TON follow-ups: releases, refunds, contact, resend
ton-resend-title = Sehen Sie in Ihr Postfach
ton-pay-state-taken = Dieser Termin wurde vergeben, bevor die Buchung bezahlt war, deshalb wurde sie freigegeben. Es ist nichts zu zahlen. Bitte wählen Sie einen anderen Termin.
ton-pay-state-refund = Diese Buchung wurde ohne Bestätigung beendet. Die bezahlten { $paid } TON werden erstattet; der Gastgeber meldet sich bei Ihnen.
ton-pay-contact = Fragen? Schreiben Sie an { $host }: { $email }.
ton-pay-copy-manual = Kopieren hat nicht geklappt. Der Text ist markiert: Kopieren Sie ihn von Hand.
ton-email-taken-message = Der Termin, den Sie für { $event } gewählt haben, wurde vergeben, bevor die Buchung bezahlt war, deshalb wurde sie freigegeben. Es ist nichts zu zahlen. Sie können gern einen anderen Termin wählen.
ton-email-refund-message = Wir haben Ihre Zahlung für { $event } erhalten, aber der Termin konnte nicht bestätigt werden, deshalb wurde die Buchung freigegeben. { $host } erstattet die Zahlung und meldet sich bei Ihnen.
ton-problem-host-busy-unpaid = Der Termin dieser unbezahlten Buchung wurde vergeben (eine Anfrage wurde darüber bestätigt, oder ein Kalendertermin kam hinzu), deshalb wurde die Buchung vor der Zahlung freigegeben. Falls schon etwas eingegangen ist, erstatten Sie es.
ton-problem-confirm-failed = Der Gast hat bezahlt, aber die Buchung konnte nach mehreren Versuchen nicht automatisch bestätigt werden (ein Fehler, kein belegter Gastgeber: siehe Serverprotokoll). Der Termin bleibt bis zum Beginn reserviert. Bestätigen Sie die Buchung von Hand oder lehnen Sie sie ab und erstatten Sie die Zahlung.
ton-health-payment-alerts-held = Weitere Überweisungen sind für Buchungen eingegangen, zu denen Sie in der letzten Stunde schon eine E-Mail bekommen haben. Sie sind ohne eigene E-Mail erfasst: Prüfen Sie die Zahlungsdaten.
ton-waive-body-partial = Diese Buchung ist nur teilweise bezahlt ({ $paid } von { $amount } TON erhalten, fällig bis { $time }). Wenn Sie sie bestätigen, bekommt der Gast den Termin, ohne die restlichen { $rest } TON zu zahlen. Erstatten Sie den erhaltenen Betrag, sofern nichts anderes vereinbart ist.

## Telegram (ilia fork)

tg-guest-cancelled-by-guest = Sie haben Ihre Buchung storniert: { $event }, { $when }.
tg-guest-expired = Anfrage abgelaufen: { $event }, { $when }. Der Gastgeber hat nicht rechtzeitig geantwortet; bitte buchen Sie einen anderen Termin.
tg-book-help-optional = Nach der Buchung können Sie Benachrichtigungen in Telegram einschalten.
tg-settings-not-linked = Ihr Telegram ist nicht verbunden.

## Telegram: TON payments (ilia fork; docs/ton.md)

tg-guest-payment-due = Ihre Buchung wartet auf die Zahlung: { $event }, { $when }. Betrag: { $amount } TON (≈ { $usd } $). Bezahlen Sie bis { $deadline }, sonst wird der Termin freigegeben.
tg-guest-payment-partial = Ein Teil der Zahlung für { $event }, { $when } ist eingegangen: { $paid } von { $amount } TON. Zahlen Sie die restlichen { $rest } TON bis { $deadline }, sonst wird der Termin freigegeben.
tg-guest-payment-received = Zahlung erhalten, Ihre Buchung ist bestätigt: { $event }, { $when }. Gastgeber: { $host }.
tg-guest-payment-late = Eine Zahlung von { $value } TON für { $event }, { $when } ist nach der Frist eingegangen und hat die Buchung deshalb nicht bestätigt. { $host } meldet sich wegen der Erstattung bei Ihnen.
tg-guest-payment-expired = Die Zeit zum Bezahlen von { $event }, { $when } ist abgelaufen, deshalb wurde der Termin freigegeben. Sie können einen anderen Termin buchen.
tg-guest-payment-taken = Der Termin, den Sie für { $event }, { $when } gewählt haben, wurde vergeben, bevor die Buchung bezahlt war, deshalb wurde sie freigegeben. Sie müssen nichts bezahlen. Sie können einen anderen Termin buchen.
tg-guest-payment-refund = Wir haben Ihre Zahlung für { $event }, { $when } erhalten, aber der Termin konnte nicht bestätigt werden, deshalb wurde die Buchung freigegeben. { $host } erstattet die Zahlung und meldet sich bei Ihnen.
tg-guest-payment-refund-part = Ein Teil der Zahlung war schon eingegangen: { $host } meldet sich wegen der Erstattung bei Ihnen.
tg-guest-payment-held = Wir haben Ihre Zahlung für { $event }, { $when } erhalten, aber die Buchung konnte nicht automatisch bestätigt werden. { $host } meldet sich bei Ihnen.

## ilia fork: booking flow v2 (docs/ton.md, docs/operations.md)
ton-etf-usd-no-rates = Auf diesem Server sind keine Wechselkurse eingerichtet (CALRS_TON_RATE_MIN_USD und CALRS_TON_RATE_MAX_USD), daher kann ein Preis in USD nicht gebucht werden. Geben Sie den Preis in TON an.
ton-etf-telegram-help = Bei einer bezahlten Buchung standardmäßig erforderlich: Der Gast bekommt dann Zahlungserinnerungen und ist über Telegram erreichbar. Sie können es optional machen oder ausschalten.
tg-guest-payment-due-ton = Ihre Buchung wartet auf die Zahlung: { $event }, { $when }. Betrag: { $amount } TON. Bezahlen Sie bis { $deadline }, sonst wird der Termin freigegeben.
ton-email-renewed-subject = Ihr Termin ist noch reserviert: { $event }
ton-email-renewed-message = Die Zahlung für { $event } mit { $host } ist noch nicht bei uns eingegangen, daher bleibt Ihr Termin bis { $deadline } reserviert. Wenn Sie schon bezahlt haben, ist nichts weiter zu tun: Die Zahlung wird angerechnet. Andernfalls bezahlen Sie bis dahin auf der Seite unten oder stornieren Sie die Buchung, wenn sich Ihre Pläne geändert haben.
tg-guest-payment-renewed = Die Zahlung für { $event }, { $when } ist noch nicht eingegangen, daher bleibt der Termin bis { $deadline } für Sie reserviert. Bezahlen Sie die noch offenen { $rest } TON bis dahin, sonst wird der Termin freigegeben. Wenn Sie schon bezahlt haben, ist nichts weiter zu tun: Die Zahlung wird angerechnet.

## ilia fork: the system mailbox (src/inbox)
booking-blocked-body-system = Der Gastgeber verlangt mindestens { $minutes } Minuten Vorlaufzeit. Falls Sie nicht teilnehmen können, schreiben Sie bitte an <a href="{ $href }">{ $address }</a>.
booking-blocked-body-no-reply = Der Gastgeber verlangt mindestens { $minutes } Minuten Vorlaufzeit. Falls Sie nicht teilnehmen können, geben Sie dem Gastgeber bitte auf anderem Weg Bescheid.

## ilia fork: passkeys and security keys (docs/passkeys-plan.md)

ilia-mfa-login-help-passkey = Bestätigen Sie diese Anmeldung mit Ihrem Passkey oder Sicherheitsschlüssel. Die Anmeldung läuft nach 10 Minuten ab.
ilia-mfa-totp-login-help = Geben Sie den sechsstelligen Code aus Ihrer Authenticator-App ein. Die Anmeldung läuft nach 10 Minuten ab.
ilia-mfa-recovery-login-help = Geben Sie einen Ihrer gespeicherten Wiederherstellungscodes ein. Jeder Code funktioniert nur einmal. Die Anmeldung läuft nach 10 Minuten ab.
ilia-passkey-unsupported = Dieser Browser unterstützt keine Passkeys oder Sicherheitsschlüssel. Wählen Sie unten eine andere Methode oder öffnen Sie diese Seite in einem aktuellen Browser.
ilia-passkey-noscript = Passkeys brauchen JavaScript. Aktivieren Sie es für diese Website oder wählen Sie unten eine andere Methode.
ilia-passkey-cancelled = Die Anfrage wurde abgebrochen oder ist abgelaufen. Versuchen Sie es erneut oder wählen Sie eine andere Methode.
ilia-passkey-invalid = Der Passkey oder Sicherheitsschlüssel konnte nicht überprüft werden, oder die Anfrage ist abgelaufen. Versuchen Sie es erneut oder wählen Sie eine andere Methode.
ilia-passkey-key-type = Dieser Browser kann diesen Schlüsseltyp nicht lesen, daher lässt sich der Schlüssel hier nicht hinzufügen. Versuchen Sie es mit einem anderen Browser, etwa einem aktuellen Firefox oder Chrome, oder wählen Sie eine andere Methode.
ilia-passkey-wrong-site = Passkeys funktionieren nur unter { $origin }. Öffnen Sie diese Seite dort und versuchen Sie es erneut, oder wählen Sie eine andere Methode.
ilia-mfa-enroll-help = Wählen Sie, womit Sie jede Anmeldung bestätigen. Am sichersten ist ein Passkey oder Sicherheitsschlüssel: Er funktioniert nur auf dieser Website, eine gefälschte Kopie der Seite kann ihn nicht nutzen.
ilia-passkey-register-help = Klicken Sie auf eine Schaltfläche und folgen Sie Ihrem Browser: Stecken Sie den Sicherheitsschlüssel ein und berühren Sie ihn, oder bestätigen Sie mit Touch ID, Face ID, Windows Hello oder Ihrem Telefon.
ilia-mfa-recovery-help = Bewahren Sie diese Wiederherstellungscodes sicher auf; sie werden nur einmal angezeigt. Jeder Code ersetzt bei einer Anmeldung einen Passkey oder einen Authenticator-Code. Neue Codes ersetzen alle bisherigen.
ilia-mfa-setup-intro = Fügen Sie jeder Anmeldung einen zweiten Schritt hinzu: einen Passkey oder Sicherheitsschlüssel (YubiKey, Touch ID, Face ID, Windows Hello, Ihr Telefon) oder eine Authenticator-App.
ilia-passkey-limit = Sie haben die Höchstzahl an Schlüsseln erreicht ({ $max }). Entfernen Sie einen, um einen neuen hinzuzufügen.
ilia-passkey-backup-advice = Fügen Sie einen zweiten Schlüssel hinzu und bewahren Sie ihn sicher auf, damit Sie sich bei Verlust eines Schlüssels nicht aussperren.
ilia-passkey-unavailable = Passkeys sind deaktiviert: Die öffentliche Adresse des Servers (CALRS_BASE_URL) ist keine https://-Adresse. Wenden Sie sich an den Administrator. Bereits hinzugefügte Schlüssel werden aufgelistet und können entfernt werden.
ilia-mfa-last-factor = Die Zwei-Faktor-Authentifizierung ist vorgeschrieben, daher kann Ihre letzte Anmeldemethode nicht entfernt werden. Fügen Sie zuerst eine weitere hinzu.
ilia-mfa-last-factor-off = Das ist Ihre letzte Methode: Wenn Sie sie entfernen, wird die Zwei-Faktor-Authentifizierung deaktiviert und Ihre Wiederherstellungscodes werden gelöscht.
ilia-mfa-changes-help = Jede Änderung hier erfordert Ihr Passwort und einen Passkey oder Code und meldet Ihre anderen Sitzungen ab.
ilia-mfa-confirm-title = Bestätigen Sie, dass Sie es sind
ilia-mfa-confirm-help = Geben Sie Ihr Passwort ein und bestätigen Sie dann mit einem Passkey oder einem Code.
ilia-mfa-action-passkey-add = Sie fügen einen Passkey oder Sicherheitsschlüssel hinzu.
ilia-mfa-action-passkey-rename = Sie benennen den Schlüssel „{ $name }“ um.
ilia-mfa-action-passkey-delete = Sie entfernen den Schlüssel „{ $name }“.
ilia-mfa-action-totp-add = Sie richten eine Authenticator-App ein.
ilia-mfa-action-totp-remove = Sie entfernen die Authenticator-App.
ilia-mfa-action-regenerate = Sie ersetzen alle Wiederherstellungscodes.
ilia-mfa-action-policy = Sie setzen die Zwei-Faktor-Richtlinie für lokale Konten auf:

# ilia fork: TON price by demand (the ladder)
ton-slots-ladder-fee = Die Netzwerkgebühr zahlt Ihre Wallet.
ton-quote-was = Ihnen wurden { $ton } TON angezeigt.
ton-quote-was-free = Ihnen wurde eine kostenlose Buchung angezeigt.
ton-quote-confirm-help = Bitte bestätigen Sie den Preis. Es wurde noch nichts gebucht.
ton-quote-why-free-used = Sie hatten vor Kurzem schon eine kostenlose Buchung, daher ist diese Buchung kostenpflichtig. Es wurde noch nichts gebucht.
ton-bae-body-free-used = Sie hatten vor Kurzem schon eine kostenlose Buchung, daher kann diese nicht kostenlos sein. Versuchen Sie es bitte später noch einmal.
ton-form-error-mode = Wählen Sie einen festen Preis oder den Preis nach Nachfrage.
ton-etf-ladder-help = Der Preis ist die Stufe für die Zahl Ihrer anstehenden Buchungen in den nächsten Tagen (bestätigte oder bezahlte, kostenlose Anfragen, die auf Ihre Bestätigung warten, und Team-Buchungen, die Sie leiten; unbezahlte zählen nicht): zuerst kostenlos (jede kostenlose Buchung bestätigen Sie), dann höher, je voller Ihr Kalender wird. Gäste sehen den aktuellen Preis und zahlen den angezeigten. Die Staffel legt der Administrator fest.

# ilia fork: the TON settings page (/dashboard/admin/ton)
ton-admin-errors = Nichts gespeichert: Korrigieren Sie die unten markierten Felder.
ton-admin-locked-env = Um es hier zu ändern, entfernen Sie { $var } aus der Umgebung des Servers und starten Sie calrs neu.
ton-admin-locked-apply = Ändern Sie es im Dokument von config apply oder lassen Sie es in dessen ton-Abschnitt weg, um es an diese Seite zu übergeben.
ton-admin-my-demand = Ihre anstehenden Buchungen jetzt: { $n }. Ihr Staffelpreis jetzt: { $price }.
