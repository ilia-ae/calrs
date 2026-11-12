# Booking confirmation page (templates/confirmed.html)

confirmed-page-title-pending = Réservation en attente
confirmed-page-title-booked = Réservation confirmée

confirmed-heading-reschedule-requested = Reprogrammation demandée
confirmed-heading-rescheduled = Reprogrammé !
confirmed-heading-pending = En attente de confirmation
confirmed-heading-booked = C'est réservé !

confirmed-subtitle-reschedule-requested = Votre demande de reprogrammation a été envoyée à { $host }. Vous recevrez un e-mail à l'adresse { $email } une fois qu'elle sera approuvée.
confirmed-subtitle-rescheduled = Votre réservation a été reprogrammée. Un e-mail de confirmation a été envoyé à { $email }.
confirmed-subtitle-pending = Votre demande de réservation a été envoyée à { $host }. Vous recevrez un e-mail à l'adresse { $email } une fois qu'elle sera confirmée.
confirmed-subtitle-booked = Un e-mail de confirmation a été envoyé à { $email }.

confirmed-detail-event = Événement :
confirmed-detail-date = Date :
confirmed-detail-time = Heure :
confirmed-detail-with = Avec :
confirmed-detail-location = Lieu :
confirmed-detail-notes = Notes :
confirmed-detail-additional-guests = Invités supplémentaires :

confirmed-book-another = Réserver un autre créneau

confirmed-add-to-calendar = Ajouter à l'agenda

# Slot picker (templates/slots.html)

slots-location-video = Visioconférence
slots-location-phone = Appel téléphonique
slots-location-google-meet = Google Meet

slots-tz-label = Votre fuseau horaire
slots-time-format-label = Format de l'heure
tz-group-common = Fuseaux horaires courants
tz-region-africa = Afrique
tz-region-america = Amériques
tz-region-antarctica = Antarctique
tz-region-arctic = Arctique
tz-region-asia = Asie
tz-region-atlantic = Océan Atlantique
tz-region-australia = Australie
tz-region-europe = Europe
tz-region-indian = Océan Indien
tz-region-pacific = Océan Pacifique
slots-offset-note-day = Les horaires du %1 sont en %2 (changement d'heure le %3).
slots-offset-note-range = Changement d'heure le %1 : les horaires sont ensuite en %2.

slots-view-month = Vue mensuelle
slots-view-week = Vue hebdomadaire
slots-view-column = Vue en liste

slots-weekday-mon = Lun
slots-weekday-tue = Mar
slots-weekday-wed = Mer
slots-weekday-thu = Jeu
slots-weekday-fri = Ven
slots-weekday-sat = Sam
slots-weekday-sun = Dim

slots-weekday-mon-short = L
slots-weekday-tue-short = M
slots-weekday-wed-short = M
slots-weekday-thu-short = J
slots-weekday-fri-short = V
slots-weekday-sat-short = S
slots-weekday-sun-short = D

slots-select-date = Choisissez une date
slots-loading-availability = Chargement des disponibilités...
slots-click-highlighted = Cliquez sur une date en surbrillance pour voir les créneaux disponibles
slots-bookable-range-both = Réservable du { $from } au { $until }
slots-bookable-range-from = Réservable à partir du { $from }
slots-bookable-range-until = Réservable jusqu'au { $until }
slots-no-longer-bookable = Cet événement n'est plus réservable.
slots-no-times-month = Aucun créneau disponible ce mois-ci
slots-no-times-day = Aucun créneau disponible ce jour
slots-no-availability-participants = Aucune disponibilité commune trouvée pour tous les participants ce mois-ci
slots-week-more = autres

# Booking form (templates/book.html)

book-page-title = Réserver { $title }
book-back-to-times = Retour aux créneaux
book-name-label = Votre nom
book-name-placeholder = Jeanne Dupont
book-email-label = Adresse e-mail
book-email-placeholder = jeanne@example.com
book-email-invalid = Veuillez saisir une adresse e-mail complète, avec le nom de domaine (par ex. jeanne@example.com).
book-notes-label = Notes
book-notes-optional = (facultatif)
book-notes-placeholder = Y a-t-il des points que vous aimeriez aborder ?
book-additional-guests-label = Invités supplémentaires
book-additional-guests-hint = (facultatif, jusqu'à { $max })
book-add-guest-btn = + Ajouter un invité
book-guest-email-placeholder = collegue@example.com
book-phone-label = Numéro de téléphone
book-phone-placeholder = 06 12 34 56 78
book-phone-help = Les numéros locaux fonctionnent ; sans le + initial, nous supposons { $country }.
book-phone-optional-consequence = Laissez vide si vous préférez ne pas recevoir de SMS au sujet de cette réservation.
book-phone-required = Un numéro de téléphone est obligatoire pour cette réservation.
book-phone-invalid-title = Numéro de téléphone invalide
book-phone-invalid = Veuillez saisir un numéro joignable par SMS, ou laisser le champ vide.
book-phone-country-search = Rechercher
book-phone-country-label = Choisir le pays
book-phone-country-none = Aucun pays sélectionné
book-phone-country-no-results = Aucun pays ne correspond à cette recherche
captcha-label = Vérification de sécurité
captcha-initial-state = Vérifiez que vous êtes humain
captcha-verifying = Vérification en cours...
captcha-solved = Vous êtes humain
captcha-error = Erreur
captcha-troubleshooting = Dépannage
captcha-wasm-disabled = Activez WASM pour une résolution plus rapide
captcha-verify-aria = Cliquez pour vérifier que vous êtes humain
captcha-verifying-aria = Vérification en cours, veuillez patienter
captcha-verified-aria = Vérifié
captcha-required = Veuillez vérifier que vous êtes humain
captcha-error-aria = Une erreur est survenue, veuillez réessayer
book-confirm-button = Confirmer la réservation

# SMS notifications (src/sms/message.rs).
#
# These are text messages, billed per 160-character segment (70 if the text
# contains any character outside the GSM-7 alphabet, which includes most
# accented letters). Keep them short and plain.

sms-confirmed = Réservation confirmée : { $event }, le { $date } à { $time } ({ $tz }).
sms-cancelled = Réservation annulée : { $event }, le { $date } à { $time } ({ $tz }).
sms-rescheduled = Réservation déplacée : { $event } aura lieu le { $date } à { $time } ({ $tz }).
sms-reminder = Rappel : { $event } commence le { $date } à { $time } ({ $tz }).

# Shared labels used across the cancel / decline / approve / reschedule / claim flows

common-detail-guest = Invité :
common-detail-reason = Motif :
common-reason-optional = (facultatif)
common-close-page = Vous pouvez fermer cette page.

# Cancel flow (booking_cancel_form.html, booking_cancelled_guest.html)

cancel-page-title = Annuler la réservation
cancel-heading = Annuler la réservation
cancel-subtitle = Vous êtes sur le point d'annuler votre réservation.
cancel-reason-label = Motif
cancel-reason-placeholder-host = Indiquez à l'organisateur la raison...
cancel-button = Annuler la réservation
cancelled-heading = Réservation annulée
cancelled-subtitle = Votre réservation a été annulée et l'organisateur a été informé.

# Decline flow (booking_decline_form.html, booking_declined.html)

decline-page-title = Refuser la réservation
decline-heading = Refuser la réservation
decline-subtitle = Vous êtes sur le point de refuser cette demande de réservation.
decline-reason-placeholder-guest = Indiquez à l'invité la raison...
decline-button = Refuser la réservation
declined-heading = Réservation refusée
declined-subtitle = La réservation a été refusée et l'invité a été informé.

# Approve flow (booking_approve_form.html, booking_approved.html)

approve-page-title = Approuver la réservation
approve-heading = Approuver la réservation
approve-subtitle = Vous êtes sur le point d'approuver cette demande de réservation.
approve-button = Approuver la réservation
approved-heading = Réservation approuvée
approved-subtitle = La réservation a été confirmée et un e-mail de confirmation a été envoyé à { $email }.

# Claim flow (booking_claim_form.html, booking_claimed.html, booking_already_claimed.html)

claim-page-title = Prendre la réservation
claim-heading = Prendre la réservation
claim-subtitle = Vous êtes sur le point de prendre en charge cette réservation. Vous serez ajouté comme participant.
claim-assigned-to = Attribuée à :
claim-button = Prendre cette réservation
claimed-page-title = Réservation prise en charge
claimed-heading = Réservation prise en charge
claimed-subtitle = Vous avez pris en charge cette réservation. Une invitation a été envoyée à votre adresse e-mail.
already-claimed-page-title = Déjà prise en charge
already-claimed-heading = Déjà prise en charge
already-claimed-subtitle = Cette réservation a déjà été prise en charge par { $name }.

# Generic error page (booking_action_error.html)

action-error-page-title = Erreur d'action sur la réservation

# Host-initiated reschedule (booking_host_reschedule.html)

host-resched-page-title = Reprogrammer la réservation — calrs
host-resched-heading = Reprogrammer la réservation
host-resched-subtitle = Cela enverra à { $guest } un e-mail lui demandant de choisir un nouveau créneau.
host-resched-currently = Actuellement :
host-resched-button = Envoyer la demande de reprogrammation
host-resched-cancel-link = Annuler

# Guest reschedule confirmation (booking_reschedule_confirm.html)

resched-confirm-page-title = Confirmer la reprogrammation
resched-confirm-heading = Confirmer la reprogrammation
resched-confirm-subtitle = Vous êtes sur le point de déplacer votre réservation à un nouveau créneau.
resched-was = Avant :
resched-new = Maintenant :
resched-button = Confirmer la reprogrammation
resched-back-to-picker = Retour au choix du créneau

# Base layout chrome (templates/base.html)

base-loader-checking = Vérification des disponibilités
base-loader-please-wait = Veuillez patienter, chargement des dernières données de calendrier...
base-stop-impersonating = Arrêter l'usurpation
base-theme-toggle = Changer de thème
base-language-choose = Choisissez votre langue
base-powered-by = Propulsé par

# Profile (templates/profile.html)

profile-pick-event-type-invite = Choisissez un type d'événement pour réserver un créneau.
profile-no-event-type = Aucun type d'événement disponible pour le moment.

# Month and weekday names + per-locale date format patterns.
# Used by server-side date formatters in src/i18n.rs.

common-month-1 = janvier
common-month-2 = février
common-month-3 = mars
common-month-4 = avril
common-month-5 = mai
common-month-6 = juin
common-month-7 = juillet
common-month-8 = août
common-month-9 = septembre
common-month-10 = octobre
common-month-11 = novembre
common-month-12 = décembre

common-weekday-long-mon = lundi
common-weekday-long-tue = mardi
common-weekday-long-wed = mercredi
common-weekday-long-thu = jeudi
common-weekday-long-fri = vendredi
common-weekday-long-sat = samedi
common-weekday-long-sun = dimanche

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
email-action-reschedule = Reprogrammer
email-action-cancel-booking = Annuler la réservation

# Email: guest booking confirmation

# Kept to "event — date": Exchange titles the guest appointment after the
# email Subject header, not the ICS SUMMARY (#157).
email-confirm-subject = { $event } — { $date }
email-confirm-greeting = Bonjour { $name },
email-confirm-headline = Votre réservation est confirmée !
email-confirm-ics-attached-plain = Une invitation est jointe à cet e-mail.
email-confirm-ics-attached-html = Une invitation est jointe à cet e-mail.
email-confirm-need-to-cancel = Besoin d'annuler ? { $url }

# Email: guest reminder

email-reminder-subject = Rappel : { $event } à { $time }
email-reminder-headline = Votre rendez-vous approche.

# Email: guest cancellation

email-cancel-subject = Annulation : { $event } — { $date }
email-cancel-headline-by-host = Votre réservation a été annulée par { $host }.
email-cancel-headline-by-guest = Votre réservation a été annulée.
email-cancel-ics-attached-plain = Une annulation de calendrier est jointe.
email-cancel-ics-attached-html = Une annulation de calendrier est jointe à cet e-mail.

# Confirmation email: notice-window policy lines (src/email.rs)

email-confirm-cancel-notice = Note : l'annulation exige un préavis d'au moins { $minutes } minutes.
email-confirm-reschedule-notice = Note : la reprogrammation exige un préavis d'au moins { $minutes } minutes.

# Event type form: cancel/reschedule minimum notice (templates/event_type_form.html)

# Google Meet (templates/event_type_form.html, src/google_meet.rs)
event-type-form-location-google-meet = Google Meet (lien généré automatiquement)
event-type-form-location-google-meet-hint = Un lien Google Meet unique est créé à la confirmation, au nom de l'organisateur assigné. Chaque organisateur (vous, ou chaque membre éligible de l'équipe) doit avoir Google Agenda connecté avec un agenda d'écriture sélectionné.
google-meet-prereq-no-host = Google Meet nécessite un organisateur avec Google Agenda connecté.
google-meet-prereq-no-eligible = Google Meet nécessite au moins un membre de l'équipe éligible avec Google Agenda connecté.
google-meet-prereq-missing = Google Meet exige que chaque organisateur ait Google Agenda connecté avec un agenda d'écriture sélectionné. Il manque encore : { $names }. Connectez-les depuis Tableau de bord → Sources d'agenda.
google-meet-unavailable-title = Google Meet n'est pas disponible
google-meet-dynamic-group-unavailable = L'organisateur doit avoir Google Agenda connecté avec un agenda d'écriture sélectionné.

event-type-form-cancel-notice-label = Préavis minimum pour annuler
event-type-form-reschedule-notice-label = Préavis minimum pour reprogrammer
event-type-form-notice-help = Laissez vide pour ne pas imposer de restriction.
event-type-form-resources-label = Ressources requises
event-type-form-resources-hint = Les créneaux ne sont proposés que si les ressources sélectionnées sont disponibles, selon le mode ci-dessous.
event-type-form-resources-mode-all = Toutes les ressources sélectionnées doivent être libres
event-type-form-resources-mode-round-robin = Une seule ressource libre suffit (elle est attribuée à la réservation)
event-type-form-notice-unit-minutes = minutes
event-type-form-notice-unit-hours = heures
event-type-form-notice-unit-days = jours
event-type-form-booking-horizon-label = Horizon de réservation
event-type-form-booking-horizon-help = Nombre de jours à l'avance pendant lesquels les invités peuvent réserver. Laissez vide pour aucune limite, 0 pour aujourd'hui seulement.
event-type-form-bookable-range-label = Période réservable
event-type-form-bookable-from = du
event-type-form-bookable-until = au
event-type-form-bookable-range-help = Seuls ces jours peuvent être réservés. Laissez vide pour aucune limite. Utilisez les exceptions pour d'autres horaires certains jours.
event-type-form-bookable-range-past-warning = Cette période est déjà terminée. Le type d'événement n'est plus affiché publiquement.
event-type-form-bookable-range-too-far-warning = Cette période commence dans plus d'un an. Les invités peuvent réserver au plus un an à l'avance, les réservations ouvrent donc le { $date }.
event-type-form-bookable-range-horizon-warning = L'horizon de réservation se termine avant le début de cette période. Les invités peuvent réserver à partir du { $date }.
event-type-form-bookable-range-bookings-outside =
    { $count ->
        [one] 1 réservation existante tombe en dehors de la période réservable.
       *[other] { $count } réservations existantes tombent en dehors de la période réservable.
    }
event-type-form-bookable-range-bookings-link = Voir les réservations
event-type-form-saved = Enregistré.

# Booking confirmation: cancel/reschedule policy notices (templates/confirmed.html)

confirmed-cancel-notice-info = L'annulation exige un préavis d'au moins { $minutes } minutes avant la réunion.
confirmed-reschedule-notice-info = La reprogrammation exige un préavis d'au moins { $minutes } minutes avant la réunion.

# Booking action blocked page (templates/booking_action_blocked.html)

booking-blocked-title-cancel = Cette réservation ne peut plus être annulée en ligne
booking-blocked-title-reschedule = Cette réservation ne peut plus être reprogrammée en ligne
booking-blocked-body = L'hôte exige un préavis d'au moins { $minutes } minutes. Si vous ne pouvez pas être présent, écrivez directement à <a href="mailto:{ $host_email }">{ $host_email }</a>.

# Dashboard event types listing (templates/dashboard_event_types.html)

dashboard-event-types-copy = Copier
dashboard-event-types-copied = Copié !
dashboard-event-types-copy-title = Copier le lien de réservation
dashboard-event-types-copy-failed = Échec de la copie

# Dashboard sidebar and shared chrome (templates/dashboard_base.html)

nav-section-scheduling = Planification
nav-overview = Vue d'ensemble
nav-event-types = Types d'événement
nav-bookings = Réservations
nav-teams = Équipes
nav-section-shared-links = Liens partagés
nav-invite-links = Liens d'invitation
nav-section-calendars = Agendas
nav-sources = Sources
nav-section-personal = Personnel
nav-settings = Profil et paramètres
nav-troubleshoot = Diagnostic
nav-section-admin = Administration
nav-admin-panel = Panneau d'administration
nav-sign-out = Se déconnecter
nav-release-notes = Voir les notes de version

# Timezone mismatch banner (templates/dashboard_base.html)

tz-banner-text = Le fuseau horaire de votre navigateur est { $detected } alors que votre fuseau de réservation est { $current }.
tz-banner-update = Mettre à jour
tz-banner-dismiss = Ignorer

# Markdown editor toolbar (templates/dashboard_base.html)

editor-link-prompt = Saisissez l'URL :
editor-link-default-label = texte du lien
editor-placeholder-text = texte
editor-nothing-to-preview = Rien à prévisualiser

# Dashboard overview (templates/dashboard_overview.html)

overview-page-title = Tableau de bord
overview-welcome = Bienvenue, { $name }
overview-public-page = Page publique :
overview-avail-banner-title = Disponibilité par défaut
overview-avail-banner-body = Vos horaires de travail par défaut ont été fixés du lundi au vendredi, de 9h00 à 17h00. Ils sont utilisés lorsque d'autres personnes vous incluent dans une réunion de groupe dynamique.
overview-avail-banner-cta = Vérifier vos disponibilités
overview-dismiss = Ignorer
overview-getting-started = Pour commencer
overview-getting-started-help = Suivez ces étapes pour commencer à accepter des réservations.
overview-step-connect-calendar = Connecter un agenda
overview-step-first-event-type = Créer votre premier type d'événement
overview-step-share-link = Partager votre lien de réservation
overview-pending-approval = En attente d'approbation
overview-booking-with = { $title } avec { $guest }
overview-badge-pending = en attente
overview-guest-booked = Réservé par l'invité :
overview-confirm = Confirmer
overview-decline = Refuser
overview-stat-event-types = Types d'événement
overview-stat-upcoming = Réservations à venir
overview-stat-pending = En attente d'approbation
overview-stat-sources = Sources d'agenda
overview-quick-actions = Créer un type d'événement
overview-action-public-title = Page de réservation publique
overview-action-public-desc = Partagez un lien — n'importe qui peut choisir un créneau et réserver avec vous.
overview-action-team-title = Planification d'équipe
overview-action-team-desc = Répartissez les réservations entre les membres de l'équipe ou trouvez un créneau où tout le monde est libre.
overview-action-team-desc-empty = Créez d'abord une équipe, puis configurez des types d'événement partagés.
overview-action-private-title = Privé, sur invitation
overview-action-private-desc = Générez des liens à usage unique pour des contacts précis. Personne d'autre ne peut réserver.
overview-action-shared-title = Liens d'invitation partagés
overview-action-shared-desc = Tout collègue de l'équipe peut générer des liens de réservation à partager à l'extérieur.
overview-action-reason-calendar = Connectez d'abord un agenda
overview-action-reason-ask-admin = Demandez à un administrateur de créer une équipe
overview-action-reason-team-admin = Nécessite une équipe — créez-en une d'abord
overview-action-reason-team-member = Nécessite une équipe — demandez à un administrateur

# Dashboard bookings (templates/dashboard_bookings.html)

bookings-page-title = Réservations
bookings-pending-approval = En attente d'approbation
bookings-available-to-claim = À prendre en charge
bookings-upcoming = Réservations à venir
bookings-with = { $title } avec { $guest }
bookings-guest-booked = Réservé par l'invité :
bookings-resource = Ressource :
bookings-confirm = Confirmer
bookings-reschedule = Reprogrammer
bookings-decline = Refuser
bookings-claim = Prendre en charge
bookings-badge-awaiting-reschedule = reprogrammation en attente
bookings-badge-writeback-failed = absent de votre agenda
bookings-writeback-failed-detail = Échec de l'écriture dans l'agenda : { $error }
bookings-cancel = Annuler
bookings-reason-placeholder = Motif (facultatif)
bookings-confirm-cancel = Confirmer l'annulation
bookings-back = Retour
bookings-empty = Aucune réservation à venir pour le moment.<br>Partagez vos { $link } pour que l'on puisse réserver avec vous.
bookings-empty-link-label = liens de types d'événement

# Dashboard teams listing (templates/dashboard_teams.html)

teams-page-title = Équipes
teams-heading = Équipes
teams-new = Nouvelle
teams-badge-public = publique
teams-badge-private = privée
teams-settings = Paramètres
teams-view = Voir
teams-empty = Aucune équipe pour le moment.
teams-empty-admin = { $link } pour collaborer avec votre équipe.
teams-empty-admin-link-label = Créez-en une
teams-empty-member = Les équipes sont créées par les administrateurs. Demandez au vôtre d'en créer une et de vous y ajouter.

# Dashboard invite links (templates/dashboard_internal.html)

invite-links-page-title = Liens d'invitation
invite-links-heading = Liens d'invitation
invite-links-new = Nouvel événement interne
invite-links-help = Générez des liens de réservation à usage unique pour les types d'événement internes. Tout collègue authentifié peut créer et partager des liens ici.
invite-links-duration = { $minutes } min
invite-links-hosted-by = Organisé par { $host }
invite-links-get-link = Obtenir un lien
invite-links-invites = Invitations
invite-links-empty = Aucun type d'événement interne pour le moment.<br>{ $link } avec la visibilité « Interne » pour permettre à tout collègue de générer des liens de réservation.
invite-links-empty-link-label = Créez un type d'événement
invite-links-js-generating = Génération...
invite-links-js-copied = Copié !
invite-links-js-error = Erreur

teams-member-count =
    { $count ->
        [one] { $count } membre
       *[other] { $count } membres
    }

# Dashboard calendar sources (templates/dashboard_sources.html)

sources-page-title = Sources d'agenda
sources-heading = Sources d'agenda
sources-add = Ajouter
sources-last-sync = Dernière synchronisation :
sources-sync-failing = Synchronisation en échec depuis le { $since } UTC : { $error }
sources-sync = Synchroniser
sources-full-resync = Resynchronisation complète
sources-full-resync-title = Vider le cache et récupérer tous les événements depuis le serveur
sources-test = Tester
sources-reconnect = Reconnecter
sources-reconnect-title = Relancer le processus de consentement Google
sources-edit = Modifier
sources-remove = Supprimer
sources-remove-confirm = Supprimer la source « { $name } » ? Tous les événements synchronisés depuis cette source seront effacés.
sources-no-write-calendar = Aucun agenda d'écriture sélectionné. Les réservations confirmées restent dans calrs et ne sont pas envoyées vers cet agenda. Choisissez-en un ci-dessous pour activer l'écriture.
sources-write-bookings-to = Écrire les réservations dans :
sources-write-none = Aucun (ne pas écrire)
sources-empty = Aucune source d'agenda connectée. { $link } pour vérifier les disponibilités.
sources-empty-link-label = Ajoutez-en une
sources-calendars = Agendas
sources-calendars-synced = Agendas synchronisés : { $synced } sur { $total }
source-calendars-page-title = Agendas à synchroniser
source-calendars-heading = Agendas à synchroniser
source-calendars-back = Retour aux sources d'agenda
source-calendars-help = Choisissez les agendas de { $name } que calrs synchronise. Les agendas décochés ne sont jamais récupérés et ne bloquent pas vos disponibilités.
source-calendars-select-all = Tout sélectionner
source-calendars-select-none = Tout désélectionner
source-calendars-write-badge = Réservations écrites ici
source-calendars-write-hint = Les réservations confirmées sont écrites dans cet agenda, qu'il soit synchronisé ou non.
source-calendars-save = Enregistrer
source-calendars-saved = Enregistré. Les agendas nouvellement cochés seront récupérés à la prochaine synchronisation ; utilisez Synchroniser sur la page des sources pour les récupérer dès maintenant.
source-calendars-empty = Aucun agenda trouvé pour cette source pour l'instant. Lancez une synchronisation pour les découvrir.

# Dashboard event types listing (templates/dashboard_event_types.html)

event-types-page-title = Types d'événement
event-types-heading = Types d'événement
event-types-new = Nouveau
event-types-badge-disabled = désactivé
event-types-badge-internal = interne
event-types-badge-private = privé
event-types-badge-resources = ressources
event-types-badge-active = actif
event-types-badge-expired = expiré
event-types-badge-range-from = à partir du { $date }
event-types-badge-range-until = jusqu'au { $date }
event-types-send-invites = Envoyer des invitations
event-types-duration = { $minutes } min
event-types-mode-collective = collectif
event-types-mode-round-robin = tour de rôle
event-types-edit = Modifier
event-types-disable = Désactiver
event-types-enable = Activer
event-types-embed = Intégrer
event-types-overrides = Exceptions
event-types-team-settings = Paramètres de l'équipe
event-types-invites = Invitations
event-types-view-public = Voir la page publique
event-types-view-page = Voir la page
event-types-delete = Supprimer
event-types-delete-confirm = Supprimer le type d'événement « { $title } » ? Cette action est irréversible.
event-types-duplicate = Dupliquer
event-types-duplicate-title = { $title } (copie)
event-types-empty = Aucun type d'événement pour le moment. { $link } pour commencer à accepter des réservations.
event-types-empty-link-label = Créez-en un

# Markdown editor toolbar (templates/settings.html, templates/team_form.html)

editor-bold = Gras (Ctrl+B)
editor-italic = Italique (Ctrl+I)
editor-strikethrough = Barré
editor-code = Code en ligne
editor-link = Insérer un lien (Ctrl+K)
editor-toggle-preview = Afficher ou masquer l'aperçu
editor-preview = Aperçu

# Profile and settings (templates/settings.html)

settings-page-title = Paramètres
settings-heading = Profil et paramètres
settings-public-page-label = Votre page de réservation publique
settings-copy = Copier
settings-copied = Copié !
settings-open = Ouvrir
settings-avatar = Avatar
settings-upload = Téléverser
settings-remove = Supprimer
settings-display-name = Nom affiché
settings-display-name-placeholder = Votre nom
settings-username = Nom d'utilisateur
settings-username-hint = (utilisé dans votre URL de réservation)
settings-username-pattern-title = Minuscules, chiffres et tirets uniquement
settings-username-help = Votre page de réservation publique :
settings-title = Fonction
settings-title-placeholder = ex. Ingénieure logiciel, Chef de produit
settings-title-help = Affichée sur votre profil public et dans la barre latérale.
settings-bio = Biographie
settings-bio-placeholder = Présentez-vous en quelques mots...
settings-bio-help = Affichée sur votre page de réservation publique. Prend en charge **gras**, *italique*, ~~barré~~, `code` et [liens](url).
settings-booking-email = E-mail de réservation
settings-booking-email-help = Cette adresse apparaîtra sur vos pages de réservation publiques et dans les notifications. Laissez vide pour utiliser votre adresse de connexion.
settings-booking-email-warning = Assurez-vous que cette adresse existe chez votre fournisseur de messagerie. Sinon, les notifications ne seront pas remises.
settings-timezone = Fuseau horaire
settings-timezone-help = Vos règles de disponibilité et vos horaires de réservation sont calculés dans ce fuseau horaire.
settings-language = Langue
settings-language-auto = Auto (langue du navigateur)
settings-language-help = Choisissez une langue d'interface, ou laissez sur Auto pour suivre le réglage de votre navigateur.
settings-public-page-language = Langue des pages de réservation publiques
settings-public-page-language-visitor = La langue de chaque visiteur
settings-public-page-language-host = Toujours ma langue
settings-public-page-language-help = Langue de votre profil public et de vos pages de réservation, et des e-mails reçus par vos invités. « Toujours ma langue » utilise la langue choisie ci-dessus.
settings-error-public-page-language = Choisissez une langue valide pour les pages de réservation publiques.
settings-dynamic-group = Autoriser les autres à m'inclure dans les liens de groupe dynamiques
settings-dynamic-group-help = Une fois activé, d'autres utilisateurs peuvent créer des URL de réunion collective ponctuelles qui vous incluent (ex. { $example }).
settings-lend-resource = Prêter mon accès agenda pour les réservations de ressources
settings-lend-resource-help = Lorsqu'une réservation doit réserver une ressource partagée (laboratoire de démo, salle de réunion) accessible en écriture par votre compte agenda, autoriser calrs à utiliser vos identifiants enregistrés pour cette écriture.
settings-default-availability = Disponibilité par défaut
settings-default-availability-help = Vos horaires de travail par défaut. Utilisés pour les liens de groupe dynamiques lorsque d'autres vous incluent dans une réunion.
settings-copy-to-all = Copier sur tous les jours
settings-copy-to-all-title = Copier les plages du premier jour activé vers tous les autres jours activés
settings-add-window = Ajouter une plage horaire
settings-remove-window = Supprimer la plage
settings-save = Enregistrer les paramètres
settings-appearance = Apparence
settings-theme-system = Système
settings-theme-light = Clair
settings-theme-dark = Sombre

# Sign in (templates/auth/login.html)

login-page-title = Connexion
login-heading = Connexion
login-subtitle = Connectez-vous à votre compte calrs
login-sso = Se connecter avec le SSO
login-or = ou
login-email = E-mail
login-password = Mot de passe
login-submit = Se connecter par e-mail
login-no-account = Vous n'avez pas de compte ? { $link }
login-register-link = Inscrivez-vous

# Registration (templates/auth/register.html)

register-page-title = Inscription
register-heading = Créer un compte
register-subtitle = Créez un nouveau compte calrs
register-domains-limited = L'inscription est réservée à : { $domains }
register-name = Nom
register-name-placeholder = Votre nom
register-email = E-mail
register-password = Mot de passe
register-password-hint = (12 caractères minimum)
register-submit = Créer un compte
register-have-account = Vous avez déjà un compte ? { $link }
register-signin-link = Connectez-vous

# Authentication errors (src/auth.rs)

auth-error-rate-limited = Trop de tentatives de connexion. Veuillez réessayer plus tard.
auth-error-invalid-credentials = Adresse e-mail ou mot de passe incorrect
auth-error-internal = Erreur interne
auth-error-registration-disabled = Les inscriptions sont désactivées.
auth-error-name-length = Le nom doit contenir entre 1 et 255 caractères
auth-error-email-length = L'adresse e-mail doit contenir entre 1 et 255 caractères
auth-error-email-invalid = Veuillez saisir une adresse e-mail valide
auth-error-email-domain = Domaine de messagerie non autorisé
auth-error-password-length = Le mot de passe doit contenir au moins 12 caractères
auth-error-email-taken = Cette adresse e-mail est déjà utilisée
auth-error-create-failed = Échec de la création du compte

# Calendar source test and write-back setup (templates/source_test.html, templates/source_write_setup.html)

source-test-page-title = Source d'agenda
source-test-sync-heading = Synchronisation : { $name }
source-test-heading = Test de connexion
source-write-page-title = Configurer l'écriture dans l'agenda
source-write-back = Retour au tableau de bord
source-write-heading = Où enregistrer les réservations ?
source-write-help = Lorsqu'une personne réserve une réunion avec vous, calrs peut créer automatiquement l'événement dans votre agenda. Choisissez l'agenda dans lequel écrire les réservations pour { $name }.
source-write-save = Enregistrer
source-write-skip = Passer pour l'instant
source-write-sync-results = Résultats de la synchronisation

source-write-event-count =
    { $count ->
        [one] { $count } événement
       *[other] { $count } événements
    }

# Date overrides (templates/overrides.html)

overrides-page-title = Exceptions de date
overrides-heading = Exceptions de date
overrides-back-teams = Retour aux équipes
overrides-back-event-types = Retour aux types d'événement
overrides-intro = Ajoutez des exceptions à des dates précises pour { $title }
overrides-add-heading = Ajouter une exception
overrides-date = Date
overrides-type = Type d'exception
overrides-type-blocked = Bloquer toute la journée
overrides-type-custom = Horaires personnalisés
overrides-start-time = Heure de début
overrides-end-time = Heure de fin
overrides-add-submit = Ajouter l'exception
overrides-existing = Exceptions existantes
overrides-badge-blocked = bloquée
overrides-badge-custom = horaires personnalisés
overrides-delete = Supprimer
overrides-delete-confirm = Supprimer cette exception ?
overrides-empty = Aucune exception de date pour le moment.<br>Utilisez le formulaire ci-dessus pour bloquer des dates précises (jours fériés, congés) ou définir des horaires personnalisés.

# Public team page (templates/team_profile.html)

team-profile-subtitle = Choisissez un type d'événement pour réserver un créneau.
team-profile-empty = Aucun type d'événement disponible pour le moment.

# Availability troubleshoot (templates/troubleshoot.html, src/web/mod.rs)

troubleshoot-page-title = Diagnostic
troubleshoot-empty = Aucun type d'événement trouvé. { $link } pour commencer à diagnostiquer vos disponibilités.
troubleshoot-empty-link-label = Créez-en un
troubleshoot-subtitle = Comprenez pourquoi les créneaux sont disponibles ou bloqués pour { $title }
troubleshoot-duration = { $minutes } min
troubleshoot-buffer-before = { $minutes } min de marge avant
troubleshoot-buffer-after = { $minutes } min de marge après
troubleshoot-min-notice = { $minutes } min de préavis
troubleshoot-blocked-override = Bloqué par une exception de date (jour de congé)
troubleshoot-custom-hours-active = Exception d'horaires personnalisés active (remplace les règles hebdomadaires)
troubleshoot-legend-available = Disponible
troubleshoot-legend-calendar-event = Événement d'agenda
troubleshoot-legend-booking = Réservation
troubleshoot-legend-resource = Ressource occupée
troubleshoot-legend-outside = Hors horaires
troubleshoot-legend-buffer = Marge / préavis minimum
troubleshoot-blocked-slots = Créneaux bloqués
troubleshoot-none-date-blocked = Cette date est bloquée par une exception de disponibilité (jour de congé). Aucun créneau disponible.
troubleshoot-none-custom-hours = Exception d'horaires personnalisés active, mais aucune plage correspondante. Vérifiez le réglage de l'exception.
troubleshoot-none-no-rules = Aucune règle de disponibilité pour ce jour de la semaine. Ce type d'événement n'est pas réservable le { $date }.
troubleshoot-none-all-bookable = Aucun créneau bloqué pendant les horaires de disponibilité. Tous les horaires sont réservables.
troubleshoot-label-outside = Hors disponibilité
troubleshoot-label-available = Disponible
troubleshoot-label-min-notice = Préavis minimum ({ $minutes } min)
troubleshoot-label-beyond-horizon = Au-delà de l'horizon de réservation ({ $days } jours)
troubleshoot-label-outside-date-range = En dehors de la période réservable
troubleshoot-label-buffer = Marge ({ $minutes } min)
troubleshoot-label-resource-busy = Ressource occupée : { $names }
troubleshoot-detail-around = Autour de : { $label }
troubleshoot-detail-around-booking = Autour de la réservation de { $guest }
troubleshoot-reason-calendar-event = Événement d'agenda : { $label }
troubleshoot-reason-booking = Réservation : { $label }

# Invite management (templates/invite_form.html)

invites-heading = Invitations
invites-back-teams = Retour aux équipes
invites-back-event-types = Retour aux types d'événement
invites-intro = Envoyez des liens d'invitation pour { $title }
invites-capped = <strong>La saisie a été limitée à { $max } destinataires par envoi.</strong> Envoyez le reste dans un autre lot.
invites-failed-hint = — consultez les journaux du serveur pour en savoir plus.
invites-quick-link = Lien rapide
invites-quick-link-help = Générez un lien à usage unique et copiez-le dans votre presse-papiers.
invites-get-link = Obtenir un lien
invites-or-email = Ou envoyer par e-mail
invites-recipients = Destinataires
invites-recipients-hint = (une adresse par ligne, { $max } au maximum)
invites-message = Message personnel
invites-message-hint = (facultatif, envoyé à chaque destinataire)
invites-message-placeholder = Au plaisir de vous présenter une démo...
invites-expires-in = Expire dans
invites-expires-days = { $days } jours
invites-expires-never = Jamais
invites-allow-multiple = Autoriser plusieurs réservations par destinataire
invites-send = Envoyer les invitations
invites-sent-heading = Invitations envoyées
invites-badge-expired = expirée
invites-badge-used = utilisée
invites-badge-active = active
invites-sent-by = Envoyée par { $name }
invites-uses = { $used }/{ $max } utilisations
invites-expires-at = Expire le { $date }
invites-copy-link = Copier le lien
invites-delete = Supprimer
invites-delete-confirm = Supprimer cette invitation ?
invites-empty = Aucune invitation envoyée pour le moment. Utilisez le formulaire ci-dessus pour envoyer un lien de réservation.
invites-js-generating = Génération...
invites-js-copied = Copié !
invites-js-error = Erreur

invites-sent-count =
    { $count ->
        [one] { $count } invitation envoyée.
       *[other] { $count } invitations envoyées.
    }

invites-skipped-invalid =
    { $count ->
        [one] { $count } ligne invalide ignorée :
       *[other] { $count } lignes invalides ignorées :
    }

invites-skipped-duplicate =
    { $count ->
        [one] { $count } ligne en double ignorée :
       *[other] { $count } lignes en double ignorées :
    }

invites-failed =
    { $count ->
        [one] { $count } invitation en échec (BDD ou SMTP) :
       *[other] { $count } invitations en échec (BDD ou SMTP) :
    }

# Calendar source form (templates/source_form.html)

source-form-title-edit = Modifier la source d'agenda
source-form-title-add = Ajouter un agenda
source-form-heading-edit = Modifier la source d'agenda
source-form-heading-add = Connecter un agenda
source-form-subtitle-edit = Mettez à jour la connexion. Laissez le mot de passe vide pour conserver l'actuel. Après avoir changé l'URL ou le nom d'utilisateur, lancez une synchronisation pour rafraîchir la liste des agendas détectés.
source-form-subtitle-add = Connectez un serveur CalDAV ou Microsoft Exchange (EWS) pour que calrs puisse vérifier vos disponibilités lors des réservations.
source-form-backend = Backend
source-form-preset = Préréglage
source-form-connect-google = Se connecter avec Google
source-form-google-unavailable = Google Agenda n'est pas disponible. Contactez votre administrateur.
source-form-name = Nom affiché
source-form-name-placeholder = Mon agenda
source-form-url-caldav = URL CalDAV
source-form-url-ews = URL du point de terminaison EWS
source-form-username = Nom d'utilisateur
source-form-password = Mot de passe
source-form-password-keep = Laissez vide pour conserver l'actuel
source-form-password-placeholder = Mot de passe d'application ou du compte
source-form-skip-test = Ignorer le test de connexion
source-form-skip-test-help = À utiliser si le test se bloque (fréquent sur certaines installations BlueMind/Zimbra). Vous pourrez tester la connexion plus tard.
source-form-save = Enregistrer les modifications
source-form-add = Ajouter la source d'agenda
source-form-help-google-configured = Cliquez sur le bouton ci-dessous pour autoriser calrs à accéder à votre Google Agenda.
source-form-help-google-unconfigured = L'intégration Google Agenda n'est pas encore configurée. Demandez à votre administrateur de renseigner les identifiants OAuth2 Google dans le panneau d'administration.

# Calendar source form: provider help (templates/source_form.html)

source-form-help-bluemind = <strong>BlueMind</strong> — Utilisez le point de terminaison DAV de votre serveur BlueMind.<br> En général : <code>https://mail.yourcompany.com/dav/</code><br> Le nom d'utilisateur est votre <strong>adresse e-mail</strong> (ex. <code>alice@yourcompany.com</code>), pas seulement l'identifiant.<br> Si le test de connexion se bloque, cochez « Ignorer le test de connexion » et lancez directement une synchronisation.
source-form-help-nextcloud = <strong>Nextcloud</strong> — Utilisez la racine WebDAV, pas l'URL d'un agenda précis.<br> En général : <code>https://cloud.example.com/remote.php/dav</code>
source-form-help-fastmail = <strong>Fastmail</strong> — Indiquez votre adresse complète dans le chemin de l'URL.<br> Exemple : <code>https://caldav.fastmail.com/dav/calendars/user/you@fastmail.com/</code><br> Utilisez un mot de passe d'application (Settings &rarr; Privacy &amp; Security &rarr; Integrations).
source-form-help-icloud = <strong>iCloud</strong> — Utilisez <code>https://caldav.icloud.com/</code><br> Un mot de passe d'application est nécessaire, à créer sur <a href="https://appleid.apple.com" target="_blank" style="color: var(--accent);">appleid.apple.com</a> (Sécurité &rarr; Mots de passe pour application).
source-form-help-zimbra = <strong>Zimbra</strong> — Utilisez le point de terminaison DAV de votre serveur Zimbra.<br> En général : <code>https://mail.example.com/dav/</code>
source-form-help-sogo = <strong>SOGo</strong> — Utilisez le point de terminaison DAV de SOGo.<br> En général : <code>https://mail.example.com/SOGo/dav/</code>
source-form-help-radicale = <strong>Radicale</strong> — Utilisez l'URL racine du serveur.<br> En général : <code>https://cal.example.com/</code>
source-form-help-baikal = <strong>Baikal</strong>: Utilisez le point de terminaison DAV de votre serveur Baikal.<br> En général : <code>https://cal.example.com/dav.php/</code>
source-form-help-exchange = <strong>Microsoft Exchange (EWS)</strong>. Utilisez le point de terminaison SOAP :<br> <code>https://mail.example.com/EWS/Exchange.asmx</code><br> Le nom d'utilisateur est l'adresse de la boîte aux lettres ; le mot de passe doit accepter l'authentification HTTP Basic sur TLS (à activer sur une boîte de service si votre tenant l'a désactivée).<br> Pensez également à choisir <strong>Microsoft Exchange (EWS)</strong> dans la liste Backend ci-dessus.
source-form-help-google = <strong>Google Agenda</strong> : connexion via OAuth2. Aucun mot de passe requis.<br>
source-form-help-other = Saisissez l'<strong>URL racine DAV</strong> de votre serveur CalDAV, pas celle d'un agenda précis ni un lien public.<br> calrs découvrira automatiquement vos agendas via PROPFIND (RFC 4791).

# Markdown editor toolbar, short labels (templates/team_form.html, templates/team_settings.html)

editor-bold-short = Gras
editor-italic-short = Italique
editor-link-short = Insérer un lien

# Team creation (templates/team_form.html)

team-form-heading = Nouvelle équipe
team-form-name = Nom de l'équipe
team-form-name-placeholder = Ingénierie
team-form-slug = Identifiant
team-form-slug-hint = (identifiant compatible URL)
team-form-slug-pattern-title = Minuscules, chiffres et tirets uniquement
team-form-description = Description
team-form-optional = (facultatif)
team-form-description-placeholder = En quelques mots, le rôle de cette équipe...
team-form-description-help = Affichée sur la page de l'équipe. Prend en charge **gras**, *italique* et [liens](url).
team-form-visibility = Visibilité
team-form-public = Publique
team-form-private = Privée
team-form-visibility-help = Les équipes privées reçoivent un jeton d'invitation à partager. Les équipes publiques apparaissent sur la page de profil d'équipe.
team-form-members = Membres
team-form-members-help = Vous serez ajouté comme administrateur de l'équipe automatiquement. Ajoutez des utilisateurs ou liez des groupes OIDC.
team-form-search-placeholder = Rechercher des utilisateurs ou des groupes...
team-form-search-users = Utilisateurs
team-form-search-groups = Groupes OIDC
team-form-you = (vous)
team-form-submit = Créer l'équipe

# Team settings (templates/team_settings.html)

team-settings-page-title = Paramètres
team-settings-subtitle = Paramètres de l'équipe — les administrateurs de l'équipe peuvent les modifier.
team-settings-public-url = URL publique
team-settings-public-url-help = N'importe qui peut réserver via ce lien.
team-settings-invite-link = Lien d'invitation
team-settings-invite-link-help = Partagez ce lien pour donner accès à la page de réservation de cette équipe privée.
team-settings-avatar = Avatar de l'équipe
team-settings-profile = Profil
team-settings-description-placeholder = Présentez cette équipe...
team-settings-description-help = Affichée sur la page de réservation publique de l'équipe. Prend en charge **gras**, *italique* et [liens](url).
team-settings-visibility-help = Les équipes publiques sont listées sur la page de profil d'équipe. Les équipes privées nécessitent un lien d'invitation.
team-settings-members-help = Gérez les membres de cette équipe. Ajoutez des utilisateurs ou liez des groupes OIDC pour une synchronisation automatique.
team-settings-role-member = Membre
team-settings-role-admin = Administrateur
team-settings-oidc-group = Groupe OIDC
team-settings-remove = Retirer
team-settings-save = Enregistrer les modifications
team-settings-danger-zone = Zone sensible
team-settings-danger-help = Supprimer définitivement cette équipe. Les types d'événement seront dissociés (pas supprimés). Cette action est irréversible.
team-settings-delete = Supprimer cette équipe
team-settings-delete-confirm = Supprimer l'équipe « { $name } » ? Cette action est irréversible.

# Event type form (templates/event_type_form.html)

etf-heading-edit = Modifier le type d'événement
etf-heading-new = Nouveau type d'événement
etf-disabled-banner = Ce type d'événement est désactivé : personne ne peut encore le réserver.
etf-team = Équipe
etf-team-hint = (facultatif — laissez vide pour un type d'événement personnel)
etf-team-personal = Personnel
etf-scheduling-mode = Mode de planification
etf-mode-round-robin = Tour de rôle — attribuer à un membre disponible
etf-mode-collective = Collectif — tous les membres doivent être disponibles
etf-scheduling-mode-help = Le tour de rôle attribue la réservation à un membre disponible (le moins chargé d'abord). Le mode collectif exige que tous les membres soient libres en même temps.
etf-title = Titre
etf-title-placeholder = Appel de découverte de 30 min
etf-slug = Identifiant
etf-slug-placeholder = généré à partir du titre
etf-description-placeholder = Un court appel de découverte pour parler de...
etf-description-help = Affichée sur la page de réservation. Prend en charge **gras**, *italique* et [liens](url).
etf-location = Lieu
etf-location-link = Visioconférence (URL fixe)
etf-location-jitsi = Jitsi (salon généré automatiquement)
etf-location-webhook = Webhook (fournisseur personnalisé)
etf-location-phone = Téléphone
etf-location-in-person = En personne
etf-location-custom = Personnalisé
etf-location-details = Détails
etf-location-details-placeholder = https://meet.example.com/ma-salle
etf-pattern-placeholder = Laissez vide pour utiliser le motif par défaut de l'organisation
etf-duration = Durée (minutes)
etf-slot-interval = Intervalle entre créneaux (minutes)
etf-slot-interval-placeholder = Identique à la durée
etf-slot-interval-help = Fréquence de démarrage des créneaux. Laissez vide pour suivre la durée.
etf-required-members = Membres requis
etf-required-members-help = Tous les membres cochés doivent être libres pour qu'un créneau soit proposé. Décochez les membres à exclure (leur disponibilité sera ignorée).
etf-member-priority = Priorité des membres
etf-member-priority-help = Les membres les plus prioritaires reçoivent les réservations en premier lorsqu'ils sont disponibles. À priorité égale, la répartition suit le nombre de réservations récentes.
etf-member-timezone-title = Fuseau horaire du membre. Ses horaires de travail personnels sont interprétés dans ce fuseau.
etf-priority-high = Haute
etf-priority-medium = Moyenne
etf-priority-low = Basse
etf-section-availability = Disponibilité
etf-timezone-help = Les horaires ci-dessous sont interprétés dans ce fuseau. Pour un type d'événement d'équipe, choisissez le fuseau de travail de l'équipe (pas forcément celui du créateur).
etf-reset-default = Rétablir mes horaires par défaut
etf-reset-default-title = Remplacer ces horaires par la disponibilité par défaut de votre profil
etf-availability-prefilled = Pré-rempli depuis votre { $link }. Vous pouvez le remplacer ici pour ce type d'événement.
etf-availability-prefilled-link = disponibilité par défaut
etf-section-buffers = Marges et préavis
etf-buffer-before = Marge avant (min)
etf-buffer-after = Marge après (min)
etf-min-notice = Préavis minimum
etf-min-notice-help = Délai minimum entre la réservation et la réunion.
etf-section-limits = Limites de réservation
etf-first-slot-only = Un seul créneau par jour
etf-first-slot-only-help = N'afficher que le premier horaire disponible de chaque journée.
etf-freq-limit = Limiter la fréquence des réservations
etf-freq-limit-help = Limiter le nombre de réservations de cet événement par période.
etf-add-limit = Ajouter une limite
etf-section-options = Options de réservation
etf-requires-confirmation = Confirmation requise
etf-requires-confirmation-help = Les réservations resteront en attente jusqu'à votre validation depuis le tableau de bord.
etf-sms = Notifications SMS
etf-sms-off = Désactivé, aucun numéro demandé
etf-sms-optional = Facultatif, les invités peuvent laisser un numéro
etf-sms-required = Obligatoire, les invités doivent laisser un numéro
etf-sms-help = Envoie un SMS à l'invité lorsque sa réservation est confirmée, déplacée, annulée ou sur le point de commencer, en plus de l'e-mail. Un invité qui laisse le champ vide ne reçoit tout simplement pas de SMS. Nécessite une passerelle SMS dans le { $link }.
etf-admin-panel-link = panneau d'administration
etf-additional-guests = Invités supplémentaires
etf-guests-none = Les invités ne peuvent pas en ajouter d'autres
etf-additional-guests-help = Permettre à la personne qui réserve d'inviter d'autres participants, qui recevront l'invitation d'agenda.
etf-default-view = Vue d'agenda par défaut
etf-view-month = Mois — grille d'agenda avec liste de créneaux
etf-view-week = Semaine — colonnes sur 7 jours avec créneaux
etf-view-column = Colonne — jours listés avec leurs créneaux
etf-view-week-short = semaine
etf-view-column-short = colonne
etf-default-view-help = La vue affichée par défaut aux invités. Ils peuvent en changer à tout moment.
etf-conflict-calendars = Agendas de conflits
etf-conflict-calendars-help = Choisissez les agendas à consulter pour détecter les conflits. Si aucun n'est sélectionné, tous sont utilisés.
etf-no-resources = Aucune ressource partagée configurée pour le moment. Ajoutez-en une (laboratoire de démo, salle de réunion) dans le { $link } pour l'exiger ici.
etf-section-access = Accès et notifications
etf-visibility-public = Public — visible sur votre profil
etf-visibility-internal = Interne — tout collègue peut générer des liens d'invitation
etf-visibility-private = Privé — uniquement sur lien d'invitation
etf-visibility-help = Détermine qui peut voir et réserver ce type d'événement.
etf-vis-internal = Interne
etf-reminder = Rappel de réservation
etf-reminder-none = Aucun rappel
etf-reminder-help = Envoyer un e-mail de rappel à vous et à votre invité avant la réunion.
etf-dynamic-group = Lien de groupe dynamique
etf-dynamic-group-help = Créez un lien de réunion ponctuel qui vérifie vos disponibilités et celles d'autres utilisateurs.
etf-dynamic-group-search = Rechercher un utilisateur à ajouter...
etf-dynamic-group-note = Seuls les utilisateurs qui autorisent les liens de groupe dynamiques sont affichés.
etf-dynamic-group-url = URL du lien de groupe
etf-watcher-teams = Équipes observatrices
etf-watcher-teams-help = Les équipes sélectionnées seront notifiées à chaque réservation. Leurs membres peuvent prendre en charge une réservation pour y participer.
etf-save = Enregistrer les modifications
etf-create = Créer le type d'événement
etf-js-loading = Chargement...
etf-js-no-default = Aucun défaut défini
etf-js-reset-done = Rétabli !
etf-js-error = Erreur
etf-js-remove-limit = Supprimer la limite
etf-period-day = Par jour
etf-period-week = Par semaine
etf-period-month = Par mois
etf-period-year = Par an

# Event type form: runtime summary hints (templates/event_type_form.html)

# %1 and %2 are substituted client-side; the values are only known once a field is edited.

etf-hint-no-days = Aucun jour défini
etf-hint-every-day = Tous les jours
etf-fmt-day-one = %1 jour
etf-fmt-day-other = %1 jours
etf-fmt-hours = %1 h
etf-fmt-minutes = %1 min
etf-hint-buffer-both = %1 min avant, %2 min après
etf-hint-buffer-before = %1 min de marge avant
etf-hint-buffer-after = %1 min de marge après
etf-hint-notice = %1 de préavis
etf-hint-no-buffers = Aucune marge, réservation à tout moment
etf-hint-max = Max %1
etf-hint-period-day = /jour
etf-hint-period-week = /semaine
etf-hint-period-month = /mois
etf-hint-period-year = /an
etf-hint-no-limits = Aucune limite
etf-hint-confirmation-required = Confirmation requise
etf-hint-auto-confirmed = Confirmation automatique
etf-hint-extra-guests-one = jusqu'à %1 invité supplémentaire
etf-hint-extra-guests-other = jusqu'à %1 invités supplémentaires
etf-hint-view = vue %1
etf-hint-reminder = rappel %1 avant
etf-hint-no-reminder = aucun rappel

etf-guests-up-to =
    { $count ->
        [one] Jusqu'à { $count } invité supplémentaire
       *[other] Jusqu'à { $count } invités supplémentaires
    }

etf-reminder-hours =
    { $count ->
        [one] { $count } heure avant
       *[other] { $count } heures avant
    }

etf-reminder-days =
    { $count ->
        [one] { $count } jour avant
       *[other] { $count } jours avant
    }

# Event type form: preset banners and meeting-pattern help (templates/event_type_form.html)
# Literal braces are escaped as {"{"} because Fluent reads a bare { as a placeable.

etf-preset-public = Création d'un type d'événement <strong>public</strong> &mdash; toute personne disposant du lien peut réserver.
etf-preset-private = Création d'un type d'événement <strong>privé</strong> &mdash; seules les personnes que vous invitez peuvent réserver.
etf-preset-internal = Création d'un type d'événement <strong>interne</strong> &mdash; tout collègue peut partager le lien de réservation.
etf-preset-team = Création d'un type d'événement <strong>d'équipe</strong> &mdash; les réservations sont réparties entre les membres.
etf-pattern-hint = Motif personnalisé facultatif. Jetons : <code>{"{"}username{"}"}</code>, <code>{"{"}event{"}"}</code>, <code>{"{"}date{"}"}</code>, <code>{"{"}random{"}"}</code>. Laissez vide pour utiliser le motif par défaut configuré par un administrateur.
etf-pattern-random-warning = Ce motif ne contient pas de jeton <code>{"{"}random{"}"}</code>. Deux réservations de ce type d'événement le même jour partageront le même salon, et le second invité pourra entrer dans la réunion du premier. N'utilisez des salons fixes que si c'est bien l'effet recherché.
etf-webhook-hint = L'URL de réunion propre à chaque réservation est récupérée depuis le webhook configuré par un administrateur sous Administration &rarr; Webhook de réunion. Aucune URL n'est nécessaire ici.

# Admin panel (templates/admin.html)

admin-page-title = Administration
admin-heading = Tableau de bord d'administration
admin-action-refused = Action refusée :
admin-error-last-admin = C'est le dernier administrateur actif. Promouvez ou réactivez d'abord un autre administrateur.
admin-logo = Logo de l'entreprise
admin-logo-help = Affiché sur les pages de réservation publiques. PNG, JPEG, GIF ou WebP, 2 Mo maximum.
admin-logo-refused = Le logo n'a pas été enregistré. Envoyez une image PNG, JPEG, GIF ou WebP de 2 Mo maximum.
admin-favicon = Favicon
admin-favicon-help = Affiché dans les onglets du navigateur et les favoris. Recommandé : un PNG carré d'au moins 32×32 pixels, 512 Ko maximum. Sans favicon, le crabe de calrs est affiché.
admin-favicon-refused = Le favicon n'a pas été enregistré. Envoyez une image PNG, JPEG, GIF ou WebP de 512 Ko maximum.
admin-upload-save-failed = Le fichier n'a pas pu être écrit dans le répertoire de données du serveur. Le journal du serveur donne les détails.

# User and team avatar uploads (#307)
avatar-upload-refused = L'image n'a pas été enregistrée. Envoyez une image PNG, JPEG, GIF ou WebP de 2 Mo au maximum.
avatar-upload-save-failed = L'image n'a pas pu être enregistrée sur le serveur. Réessayez plus tard ou demandez à votre administrateur de consulter le journal du serveur.
admin-company-link = Lien de l'entreprise
admin-company-link-help = Le logo pointe vers cette URL sur les pages de réservation publiques. Laissez vide pour ne pas créer de lien.
admin-theme = Thème
admin-theme-help = Choisissez un thème de couleurs pour toutes les pages. Le basculement clair/sombre est indépendant : les thèmes s'adaptent aux deux modes.
admin-theme-default = Par défaut
admin-theme-default-desc = Bleu épuré
admin-theme-nord-desc = Givre arctique
admin-theme-dracula-desc = Violet sombre
admin-theme-gruvbox-desc = Rétro chaleureux
admin-theme-solarized-desc = Le classique d'Ethan
admin-theme-tokyo-desc = Ville au néon
admin-theme-custom = Personnalisé
admin-theme-custom-desc = Vos couleurs
admin-custom-colors = Couleurs personnalisées
admin-color-accent = Accent
admin-color-accent-hover = Accent au survol
admin-color-bg = Arrière-plan
admin-color-surface = Surface
admin-color-text = Texte
admin-save-theme = Enregistrer le thème
admin-users = Utilisateurs ({ $count })
admin-user-filter = Filtrer par nom ou e-mail…
admin-badge-admin = admin
admin-badge-disabled = désactivé
admin-impersonate = Prendre l'identité
admin-demote = Rétrograder
admin-promote = Promouvoir
admin-disable = Désactiver
admin-enable = Activer
admin-delete = Supprimer
admin-no-users-match = Aucun utilisateur ne correspond au filtre.
admin-no-users = Aucun utilisateur pour le moment.
admin-groups = Groupes ({ $count })
admin-group-filter = Filtrer par nom de groupe…
admin-group-name = Nom du groupe
admin-weight = poids :
admin-no-groups-match = Aucun groupe ne correspond au filtre.
admin-no-groups = Aucun groupe synchronisé pour le moment. Les groupes sont synchronisés automatiquement depuis votre fournisseur OIDC.
admin-auth-settings = Paramètres d'authentification
admin-registration-enabled = Inscriptions activées
admin-allowed-domains = Domaines d'e-mail autorisés
admin-allowed-domains-hint = (séparés par des virgules, vide pour tous)
admin-save-auth = Enregistrer les paramètres d'authentification
admin-system-settings = Paramètres système
admin-base-url = URL de base
admin-base-url-help = URL publique de cette instance. Utilisée pour les redirections OIDC et les liens dans les e-mails (approbation/refus, annulation, rappels).
admin-private-hosts = Liste d'hôtes privés autorisés
admin-private-hosts-help = Noms d'hôtes, séparés par des virgules, autorisés à résoudre vers des IP privées ou réservées pour les sources CalDAV/EWS (dérogation à la protection SSRF). N'ajoutez que des hôtes que vous contrôlez (par exemple un serveur d'agenda sur le même réseau Docker). Laissez vide pour conserver la protection sur tous les hôtes.
admin-german-formal = Forme d'adresse en allemand
admin-german-formal-du = Informelle (du)
admin-german-formal-sie = Formelle (Sie)
admin-german-formal-help = Comment les pages et les e-mails s'adressent aux visiteurs dont le navigateur demande l'allemand. Les utilisateurs qui ont choisi une langue dans Profil et paramètres gardent la leur.
admin-unset-env = Supprimez la variable d'environnement pour modifier ce réglage ici.
admin-save-system = Enregistrer les paramètres système
admin-status = État :
admin-status-enabled = activé
admin-status-disabled = désactivé
admin-status-disabled-paren = (désactivé)
admin-status-configured = configuré
admin-status-not-configured = non configuré
admin-via-environment = (via l'environnement)
admin-issuer = Émetteur :
admin-client-id = ID client :
admin-instance = Instance :
admin-oidc-settings = Paramètres OIDC
admin-oidc-enabled = OIDC activé
admin-issuer-url = URL de l'émetteur
admin-client-id-label = ID client
admin-client-secret = Secret client
admin-keep-current-hint = (laissez vide pour conserver l'actuel)
admin-keep-current-set-hint = (laissez vide pour conserver l'actuel — actuellement défini)
admin-keep-unchanged = Laissez vide pour ne rien changer
admin-oidc-auto-register = Inscrire automatiquement les nouveaux utilisateurs OIDC
admin-save-oidc = Enregistrer les paramètres OIDC
admin-google = Google Agenda (OAuth2)
admin-save-google = Enregistrer les paramètres OAuth2 Google
admin-captcha = Captcha
admin-instance-url = URL de l'instance
admin-site-key = Clé de site
admin-secret = Secret
admin-widget-url = URL du script du widget
admin-widget-url-help = À remplacer si le CDN est bloqué. Les modifications prennent effet dès l'enregistrement.
admin-wasm-url = URL du WASM du widget
admin-wasm-url-help = Adresse d'où le widget charge son solveur. Indiquez le fichier WASM servi par votre serveur Cap pour que les pages de réservation ne contactent plus cdn.jsdelivr.net. Laissez vide pour la valeur par défaut.
admin-captcha-wasm-url-invalid = L'URL du WASM du widget doit être une URL http(s) absolue.
admin-captcha-disable-help = Laissez l'URL de l'instance, la clé de site et le secret vides pour désactiver le captcha sur les pages de réservation.
admin-save-captcha = Enregistrer les paramètres du captcha
admin-resources = Ressources
admin-resources-help = Ressources partagées réservables (laboratoire de démo, salles de réunion) alimentées par un flux d'agenda. Rattachée à des types d'événement, une ressource occupée bloque les réservations.
admin-resource-stats = Événements en cache : { $events } &middot; Rattachée à { $attached } type(s) d'événement
admin-never = jamais
admin-resource-sync-failed = (échec de la dernière tentative : { $error })
admin-writeback-enabled = Écriture : activée ({ $via })
admin-writeback-readonly = Écriture : lecture seule
admin-teams-allowed = Équipes autorisées :
admin-teams-allowed-none = aucune (administrateurs globaux uniquement)
admin-sync-now = Synchroniser maintenant
admin-test-write = Tester l'écriture
admin-delete-resource-confirm = Supprimer cette ressource ? Les types d'événement qui l'utilisent cesseront de la vérifier.
admin-name = Nom
admin-name-help = Laissez vide pour récupérer le nom depuis le flux.
admin-feed-url = URL du flux ICS (adresse de publication)
admin-feed-url-help = BlueMind : l'adresse d'agenda publique ou privée de l'agenda de la ressource.
admin-caldav-url = URL de la collection CalDAV (pour l'écriture)
admin-caldav-url-help = Facultatif. Pour BlueMind, elle est déduite automatiquement de l'URL du flux.
admin-caldav-username = Nom d'utilisateur CalDAV
admin-caldav-password = Mot de passe CalDAV
admin-resource-teams = Équipes autorisées à utiliser cette ressource
admin-resource-teams-help = Les administrateurs de ces équipes peuvent rattacher cette ressource à leurs types d'événement d'équipe. Vide : administrateurs globaux uniquement.
admin-no-teams = Aucune équipe pour le moment.
admin-save-resource = Enregistrer la ressource
admin-add-resource = Ajouter une ressource
admin-jitsi = Jitsi (liens de réunion générés automatiquement)
admin-jitsi-help = Lorsque le lieu d'un type d'événement est « Jitsi (salon généré automatiquement) », calrs construit une URL de salon pour chaque réservation en ajoutant le motif ci-dessous à votre URL de base Jitsi. Aucun appel d'API externe n'est nécessaire.
admin-display-name = Nom affiché
admin-jitsi-display-name-placeholder = ex. Meet DYB
admin-jitsi-display-name-help = Affiché aux invités sur le sélecteur de créneaux et le formulaire de réservation. « Visioconférence » par défaut si vide.
admin-room-pattern = Motif de nom de salon
admin-jitsi-disable-help = Laissez l'URL de base vide pour désactiver la génération automatique Jitsi.
admin-save-jitsi = Enregistrer les paramètres Jitsi
admin-meeting-webhook = Webhook de réunion (fournisseur de votre choix)
admin-webhook-url = URL du webhook
admin-webhook-display-name-placeholder = ex. Zoom, Whereby, Custom Meet
admin-webhook-display-name-help = Affiché aux invités à la place du badge générique « Visioconférence ».
admin-authentication = Authentification
admin-auth-none = Aucune
admin-auth-hmac = HMAC-SHA256 (en-tête X-Calrs-Signature)
admin-shared-secret = Secret partagé
admin-webhook-disable-help = Laissez l'URL vide pour désactiver le webhook de réunion.
admin-save-webhook = Enregistrer les paramètres du webhook
admin-smtp = Paramètres SMTP
admin-smtp-test-sent = E-mail de test envoyé.
admin-smtp-test-failed = L'e-mail de test n'a pas pu être envoyé. Vérifiez les journaux du serveur et vos paramètres SMTP.
admin-smtp-env-error = Erreur de configuration SMTP par l'environnement :
admin-smtp-host = Hôte :
admin-smtp-from = Expéditeur :
admin-smtp-enabled = SMTP activé
admin-host = Hôte
admin-port = Port
admin-tls-mode = Mode TLS
admin-tls-starttls = STARTTLS (port 587)
admin-tls-implicit = TLS implicite (port 465)
admin-tls-none = Aucun, non chiffré (MTA local uniquement)
admin-smtp-username-hint = (laissez vide pour un relais sans authentification)
admin-from-email = Adresse d'expédition
admin-from-name = Nom d'expéditeur
admin-save-smtp = Enregistrer les paramètres SMTP
admin-send-test-email = Envoyer un e-mail de test à
admin-send-test-email-hint = (par défaut, l'adresse de votre compte)
admin-send-test-email-btn = Envoyer l'e-mail de test
admin-smtp-clear-confirm = Supprimer la configuration SMTP enregistrée en base ?
admin-clear-db-config = Effacer la configuration en base
admin-sms = Paramètres SMS
admin-sms-help = Facultatif. Les SMS ne sont envoyés que pour les réservations sur les types d'événement où les « Notifications SMS » sont activées, et uniquement si l'invité a laissé un numéro.
admin-sms-test-sent = Message de test envoyé.
admin-sms-test-checked = Identifiants acceptés.
admin-sms-test-error = La passerelle SMS a refusé la requête.
admin-sms-captcha-warning = Le formulaire de réservation est public et le numéro du destinataire vient de l'invité : sans captcha, les SMS forment un relais ouvert que quelqu'un peut vous facturer. Configurez le captcha ci-dessus et restreignez les pays de destination dans les réglages de votre passerelle.
admin-sms-sent-today = Envoyés aujourd'hui :
admin-sms-of-cap = sur { $cap }
admin-sms-config-error = Erreur de configuration SMS :
admin-sms-gateway = Passerelle :
admin-sms-account = Compte :
admin-sms-sender = Expéditeur :
admin-sms-enabled = SMS activés
admin-sms-gateway-label = Passerelle
admin-required-on-switch = Obligatoire lors du changement de passerelle
admin-sms-docs = Documentation de l'API { $provider }
admin-sms-country = Indicatif pays par défaut
admin-sms-country-hint = (utilisé quand un invité saisit un numéro local)
admin-sms-daily-cap = Limite quotidienne
admin-sms-daily-cap-hint = (messages par jour pour toute l'instance, 0 pour aucune limite)
admin-sms-daily-cap-help = Au-delà de la limite, calrs cesse d'envoyer des SMS et continue les e-mails : une réservation n'échoue jamais parce que le budget SMS est épuisé.
admin-save-sms = Enregistrer les paramètres SMS
admin-send-test-sms = Envoyer un message de test à
admin-send-test-sms-hint-check = (laissez vide pour vérifier seulement les identifiants)
admin-send-test-sms-hint-e164 = (format E.164)
admin-test-gateway = Tester la passerelle
admin-sms-clear-confirm = Supprimer la configuration SMS enregistrée en base ?
admin-sms-allow-all = Autoriser tout utilisateur à activer les SMS sur ses types d'événement
admin-sms-allow-all-help = Désactivé par défaut : les SMS consomment le crédit du compte configuré ici, donc seuls les administrateurs peuvent basculer un type d'événement en mode SMS.
admin-save-policy = Enregistrer la politique
admin-page-of = Page %1 sur %2
admin-show-more-js = Afficher %1 de plus
admin-show-fewer = Afficher moins

# Admin panel: strings carrying markup or literal braces (templates/admin.html)

admin-delete-user-confirm = Supprimer définitivement l'utilisateur { $email } ?{"\u000A"}{"\u000A"}Cela efface son compte, son profil de planification, ses sources d'agenda, ses types d'événement et toutes les données dont il est le seul propriétaire. Les réservations passées seront supprimées avec leurs types d'événement.{"\u000A"}{"\u000A"}Pour les utilisateurs OIDC/SSO : si l'inscription automatique est activée, la personne sera recréée à sa prochaine connexion.{"\u000A"}{"\u000A"}Cette action est irréversible.
admin-system-settings-help = URL publique et paramètres de sécurité réseau. Ils peuvent aussi être définis par les variables d'environnement <code>CALRS_BASE_URL</code> et <code>CALRS_ALLOW_PRIVATE_HOSTS</code>. Lorsqu'une variable d'environnement est définie, elle <strong>prime</strong> sur la valeur ci-dessous.
admin-set-by-env = — défini par l'environnement ({ $var }), prioritaire sur la valeur enregistrée
admin-google-help = Pour activer l'intégration Google Agenda, créez des identifiants OAuth2 sur la <a href="https://console.cloud.google.com/apis/credentials" target="_blank" style="color: var(--accent);">Google Cloud Console</a>. Activez la <strong>CalDAV API</strong> (lecture des disponibilités et écriture des réservations) et la <strong>Google Calendar API</strong> (liens Google Meet), puis ajoutez { $redirect_uri } comme URI de redirection autorisée.
admin-room-pattern-help = Jetons disponibles : <code>{"{"}username{"}"}</code> (hôte), <code>{"{"}event{"}"}</code> (identifiant du type d'événement), <code>{"{"}date{"}"}</code> (AAAAMMJJ), <code>{"{"}random{"}"}</code> (8 caractères). Par défaut : { $default }.
admin-room-pattern-warning = Sans <code>{"{"}random{"}"}</code>, le nom du salon est prévisible : deux invités réservant le même type d'événement le même jour se retrouvent dans le même salon et peuvent voir la réunion de l'autre. Les salons fixes restent possibles (par exemple un salon personnel par hôte), mais n'activez cela qu'en connaissance de cause.
admin-meeting-webhook-help = Lorsque le lieu d'un type d'événement est « Webhook (fournisseur personnalisé) », calrs envoie la réservation en POST à cette URL à la confirmation et attend en retour un corps JSON <code>{"{"}"url": "https://..."{"}"}</code>.
admin-auth-hmac-help = Avec HMAC, calrs envoie <code>X-Calrs-Signature: sha256=&lt;hex&gt;</code> calculé sur le corps brut de la requête.
admin-tls-none-warning = Ne choisissez <strong>Aucun</strong> que pour un relais sur cette machine qui ne propose pas STARTTLS, ou dont le certificat est auto-signé. Le courrier, et les éventuels identifiants, circulent alors en clair.
admin-smtp-env-error-help = Corrigez les variables d'environnement <code>CALRS_SMTP_*</code>, ou supprimez-les pour gérer le SMTP depuis la base ici.
admin-smtp-env-managed = Géré par <strong>variables d'environnement</strong> (prioritaires sur la base). Modifiez les variables <code>CALRS_SMTP_*</code> pour changer ce réglage, ou supprimez-les pour gérer le SMTP ici.
admin-smtp-env-help = Vous pouvez aussi configurer par variables d'environnement (prioritaires sur ceci) : <code>CALRS_SMTP_HOST</code>, <code>CALRS_SMTP_PORT</code>, <code>CALRS_SMTP_TLS_MODE</code> (<code>starttls</code>, <code>tls</code> ou <code>none</code>), <code>CALRS_SMTP_USERNAME</code>, <code>CALRS_SMTP_PASSWORD</code>, <code>CALRS_SMTP_FROM_EMAIL</code>, <code>CALRS_SMTP_FROM_NAME</code>. Seules <code>CALRS_SMTP_HOST</code> et <code>CALRS_SMTP_FROM_EMAIL</code> sont obligatoires ; omettez le nom d'utilisateur et le mot de passe pour relayer via un MTA local sans authentification.
admin-sms-env-error-help = Corrigez les variables d'environnement <code>CALRS_SMS_*</code>, ou supprimez-les pour gérer les SMS depuis la base ici.
admin-sms-env-managed = Géré par <strong>variables d'environnement</strong> (prioritaires sur la base). Modifiez les variables <code>CALRS_SMS_*</code> pour changer ce réglage, ou supprimez-les pour gérer les SMS ici.
admin-sms-env-help = Vous pouvez aussi configurer par variables d'environnement (prioritaires sur ceci) : <code>CALRS_SMS_PROVIDER</code>, <code>CALRS_SMS_API_KEY</code>, <code>CALRS_SMS_API_SECRET</code>, <code>CALRS_SMS_SENDER</code>, <code>CALRS_SMS_BASE_URL</code>, <code>CALRS_SMS_DAILY_CAP</code>, <code>CALRS_SMS_DEFAULT_COUNTRY_CODE</code>.
admin-sms-trial-warning = <strong>Le mode d'essai Twilio est actif</strong> (<code>CALRS_SMS_TWILIO_TRIAL</code>). Les invités reçoivent le modèle prédéfini <code>sms_appointment_reminders</code> de Twilio, pas le vrai message, et seuls les numéros vérifiés dans votre console Twilio sont joignables. C'est une aide au test pour les comptes d'essai. Supprimez la variable avant d'accepter des réservations.

admin-show-more =
    { $count ->
        [one] Afficher { $count } de plus
       *[other] Afficher { $count } de plus
    }

# Calendar source form: backend picker (templates/source_form.html)

source-form-backend-help = Choisissez le protocole parlé par votre serveur. EWS vise Exchange 2019/2016/2013 auto-hébergé.

admin-sms-going-live = <strong>Avant la mise en production :</strong> restreignez les pays de destination dans votre passerelle (Twilio parle de Geo Permissions), gardez le compte prépayé sans rechargement automatique, et laissez le captcha actif. Ces trois mesures bornent ensemble le coût d'une tentative de SMS pumping.

troubleshoot-heading = Diagnostic des disponibilités

# Host-side form validation errors (src/web/mod.rs)

form-error-team-name-slug-required = Le nom et l'identifiant sont obligatoires.
form-error-team-name-length = Le nom ne doit pas dépasser 255 caractères.
form-error-team-description-length = La description ne doit pas dépasser 5000 caractères.
form-error-slug-charset = L'identifiant ne peut contenir que des minuscules, des chiffres et des tirets.
form-error-slug-reserved = Cet identifiant est réservé. Veuillez en choisir un autre.
form-error-team-slug-taken = Une équipe utilise déjà cet identifiant.
form-error-title-required = Le titre est obligatoire pour générer un identifiant.
form-error-event-type-slug-taken = Un type d'événement utilise déjà cet identifiant.
form-error-event-type-slug-taken-team = Un type d'événement de cette équipe utilise déjà cet identifiant.
form-error-event-type-title-required = Le titre est obligatoire.
form-error-event-type-title-length = Le titre doit faire 255 caractères au plus.
form-error-event-type-duration = La durée doit être un nombre entier de minutes, de 1 à 1440 (24 heures).
form-error-event-type-number = Les marges, le préavis minimum, le rappel et les invités supplémentaires doivent être des nombres entiers, 0 ou plus.
form-error-event-type-horizon = L'horizon de réservation doit être un nombre entier de jours, 0 ou plus. Laissez-le vide pour ne fixer aucune limite.
form-error-location-required = Les détails du lieu sont obligatoires (lien de visioconférence, numéro de téléphone ou adresse).
form-error-bookable-range-order = La date de fin doit être identique ou postérieure à la date de début.
form-error-bookable-range-invalid = Saisissez les dates au format AAAA-MM-JJ.
form-error-not-team-admin = Vous n'êtes pas administrateur de cette équipe.
form-error-no-account = Aucun profil de planification trouvé. Veuillez contacter un administrateur.
form-error-all-fields-required = Tous les champs sont obligatoires.
form-error-encryption = Erreur de chiffrement.
form-error-connection-failed = Échec de la connexion : { $error }. Vérifiez l'URL et les identifiants, ou cochez « Ignorer le test de connexion » pour enregistrer malgré tout.

# Settings page flash (src/web/mod.rs)

settings-saved = Paramètres enregistrés.

# Profile settings validation and flash messages (src/web/mod.rs)

settings-error-name-length = Le nom doit contenir entre 1 et 255 caractères.
settings-error-username-length = Le nom d'utilisateur doit contenir au moins 2 caractères.
settings-error-username-taken = Ce nom d'utilisateur est déjà pris.
settings-error-booking-email = Veuillez saisir une adresse e-mail de réservation valide.
settings-error-save-failed = Échec de l'enregistrement des paramètres.

# Host-facing error responses (src/web/mod.rs)

error-team-not-found-or-not-admin = Équipe introuvable, ou vous n'en êtes pas administrateur.
error-team-not-found = Équipe introuvable.
error-event-type-not-found = Type d'événement introuvable.
error-decrypt-failed = Impossible de déchiffrer les identifiants enregistrés.
error-source-not-found = Source introuvable.
error-source-no-password = Aucun mot de passe enregistré pour cette source.
error-oauth-invalid-state = Paramètre d'état invalide. Veuillez réessayer.
error-oauth-no-code = Aucun code d'autorisation reçu.
error-oauth-not-configured = OAuth2 Google n'est pas configuré.
error-no-scheduling-account = Aucun profil de planification trouvé.
error-private-event-type-not-found = Type d'événement privé introuvable.
error-access-denied = Accès refusé.

# Guest booking-flow errors (src/web/mod.rs)

error-slot-unavailable = Ce créneau n'est plus disponible.
error-slot-too-soon = Ce créneau n'est plus disponible (trop proche).
error-slot-beyond-horizon = Ce créneau dépasse la période de réservation.
error-slot-not-yet-bookable = Ce créneau n'est pas encore ouvert à la réservation.
error-invite-required = Ce type d'événement nécessite un lien d'invitation.
error-invite-invalid = Lien d'invitation invalide.
error-invite-expired = Ce lien d'invitation a expiré.
error-invite-used = Ce lien d'invitation a déjà été utilisé.
error-invalid-date = Date invalide.
error-invalid-time = Heure invalide.
error-invalid-date-format = Format de date invalide.
error-invalid-time-format = Format d'heure invalide.
error-too-many-bookings = Trop de tentatives de réservation. Veuillez réessayer dans quelques minutes.
error-too-many-requests = Trop de requêtes. Veuillez réessayer plus tard.
error-no-members-available = Aucun membre de l'équipe n'est disponible pour ce créneau.
error-dynamic-group-public-only = Les liens de groupe dynamiques ne sont disponibles que pour les types d'événement publics.
error-user-not-found = Utilisateur introuvable.

# Booking action error page: titles (templates/booking_action_error.html)

bae-title-captcha = Échec de la vérification captcha
bae-title-invalid-booking = Détails de réservation invalides
bae-title-unavailable = Indisponible pour le moment
bae-title-cannot-approve = Impossible d'approuver cette réservation
bae-title-invalid-link = Lien invalide
bae-title-invalid-or-expired = Lien invalide ou expiré
bae-title-booking-not-found = Réservation introuvable
bae-title-already-approved = Déjà approuvée
bae-title-already-declined = Déjà refusée
bae-title-already-cancelled = Déjà annulée
bae-title-booking-cancelled = Réservation annulée
bae-title-booking-declined = Réservation refusée

# Booking action error page: bodies

bae-body-go-back = Veuillez revenir en arrière et réessayer.
bae-body-unavailable = L'hôte n'accepte plus de réservations pour cette date. Veuillez choisir une autre date, ou revenir plus tard.
bae-body-resource-gone = Une ressource requise n'est plus disponible sur ce créneau. Demandez à l'invité d'en choisir un autre.
bae-body-no-claim-token = Aucun jeton de prise en charge fourni.
bae-body-claim-invalid = Ce lien de prise en charge n'est plus valide.
bae-body-booking-gone = Cette réservation n'existe plus.
bae-body-decline-link-invalid = Ce lien de refus est invalide, a expiré, ou la réservation a déjà été traitée.
bae-body-cancel-link-invalid = Ce lien d'annulation est invalide, a expiré, ou la réservation a déjà été annulée.
bae-body-cancel-link-invalid-short = Ce lien d'annulation est invalide ou a expiré.
bae-body-reschedule-link-invalid = Ce lien de reprogrammation est invalide, a expiré, ou la réservation a déjà été traitée.
bae-body-approval-link-invalid = Ce lien d'approbation est invalide ou a expiré.
bae-body-already-approved = Cette réservation a déjà été approuvée.
bae-body-already-declined = Cette réservation a déjà été refusée.
bae-body-already-cancelled = Cette réservation a déjà été annulée.
bae-body-was-cancelled = Cette réservation a été annulée.
bae-body-declined-by-host = Cette réservation a été refusée par l'hôte.

# Booking form validation (src/web/mod.rs)

validate-name-length = Le nom doit contenir entre 1 et 255 caractères.
validate-email-length = L'adresse e-mail doit contenir entre 1 et 255 caractères.
validate-email-invalid = Veuillez saisir une adresse e-mail valide.
validate-notes-length = Les notes ne doivent pas dépasser 5000 caractères.
validate-date-too-far = Impossible de réserver plus d'un an à l'avance.

# Additional guests and dynamic group links (src/web/mod.rs)

guests-not-allowed = Les invités supplémentaires ne sont pas autorisés pour ce type d'événement.
guests-too-many =
    { $max ->
        [one] Vous pouvez ajouter au plus un invité supplémentaire.
       *[other] Vous pouvez ajouter au plus { $max } invités supplémentaires.
    }
guests-invalid-email = Adresse e-mail d'invité supplémentaire invalide : { $email }
dynamic-group-min-usernames = Les liens de groupe dynamique nécessitent au moins deux noms d'utilisateur.
dynamic-group-user-not-found = Utilisateur « { $username } » introuvable.
dynamic-group-user-opted-out = L'utilisateur « { $username } » n'a pas activé les liens de groupe dynamique.

error-slot-unavailable-member = Ce créneau n'est plus disponible ({ $username } a un conflit).

# Pending requests: slot hold, expiry and approval re-check (#234)

sms-expired = Demande expirée : { $event }, le { $date } à { $time } ({ $tz }). Pas de réponse à temps, merci de choisir un autre créneau.
email-pending-expired-status = Demande expirée
email-pending-expired-headline = Votre demande de réservation a expiré : l'hôte n'a pas répondu à temps. Merci de choisir un autre créneau.
email-pending-expired-action = Choisir un autre créneau
bae-body-host-busy = L'hôte (ou un membre requis de l'équipe) n'est plus disponible sur ce créneau. Demandez à l'invité d'en choisir un autre.
bookings-error-generic = Une erreur est survenue.
bookings-error-no-bookable-dates = Ce type d'événement n'a plus de dates réservables : l'invité ne pourrait pas choisir un nouvel horaire. Prolongez d'abord sa période de réservation.
troubleshoot-detail-pending-request = { $title } (demande en attente)
etf-pending-blocks-slot = Les demandes en attente bloquent le créneau
etf-pending-blocks-slot-help = Tant qu'une demande attend votre réponse, personne d'autre ne peut réserver ce créneau, sur ce type d'événement comme sur les autres.
etf-pending-expiry = Expiration des demandes
etf-pending-expiry-help = Une demande restée sans réponse à cette échéance est refusée automatiquement et l'invité est prié de choisir un autre créneau. Elle arrive plus tôt si la réunion devient trop proche pour qu'un autre invité puisse réserver le créneau. Vous recevez un rappel à mi-parcours.
etf-pending-expiry-never = Jamais

etf-pending-expiry-hours =
    { $hours ->
        [one] Après { $hours } heure
       *[other] Après { $hours } heures
    }

# Host, watcher and remaining guest emails (src/email.rs, #209)

email-footer-sent-by = Envoyé par
email-sender-via = { $host } via { $instance }
email-date-at-time = { $date } à { $time }
email-detail-phone = Téléphone :
email-detail-expires = Expiration :
email-detail-new-date = Nouvelle date :
email-detail-new-time = Nouvel horaire :
email-detail-previous = Précédemment :
email-detail-requested = Demandé :
email-detail-originally = Initialement :
email-detail-organizer = Organisateur :
email-detail-booked-by = Réservé par :
email-detail-invited-by = Invité par :
email-detail-message = Message :
email-action-approve = Approuver
email-action-decline = Refuser
email-action-pick-new-time = Choisir un nouveau créneau
email-host-approve-line = Approuver : { $url }
email-host-decline-line = Refuser : { $url }
email-host-approval-dashboard = Connectez-vous à votre tableau de bord pour confirmer ou refuser cette réservation.
email-host-new-subject = Nouvelle réservation : { $event }, { $guest } ({ $date })
email-host-new-heading = Nouvelle réservation !
email-host-new-message = { $guest } a réservé un créneau avec vous.
email-host-confirmed-subject = Confirmation : { $event }, { $guest } ({ $date })
email-host-confirmed-heading = Réservation confirmée
email-host-confirmed-message = Vous avez approuvé la réservation de { $guest }.
email-host-confirmed-footer = L'événement a été ajouté à votre agenda.
email-host-sync-failure-subject = Action requise : agenda non mis à jour pour { $event } ({ $date })
email-host-sync-failure-heading = Votre agenda n'a pas été mis à jour
email-host-sync-failure-intro = Cette réservation a été déplacée, mais calrs n'a pas pu mettre à jour l'événement dans votre agenda Google. L'invité a le nouvel horaire ; votre agenda affiche toujours l'ancien.
email-host-sync-failure-action = Ouvrez l'événement dans Google Agenda et déplacez-le à l'horaire ci-dessus. Déplacez l'événement existant plutôt que de le recréer, sinon le lien Google Meet dont dispose déjà l'invité cessera de fonctionner.
email-writeback-failure-subject = Action requise : { $event } ({ $date }) n'est pas dans votre agenda
email-writeback-failure-heading = Une réservation manque dans votre agenda
email-writeback-failure-intro = calrs n'a pas pu ajouter cette réservation à votre agenda, même après plusieurs tentatives. La réservation est confirmée et l'invité a reçu l'invitation.
email-writeback-failure-action = Ajoutez vous-même l'événement à votre agenda pour ne pas le manquer, et vérifiez la source d'agenda dans calrs. calrs réessaie encore pendant quelques heures ; vous ne recevrez plus cet e-mail pour cette réservation.
email-detail-error = Erreur :
email-source-failure-subject = Action requise : la source d'agenda « { $source } » ne se synchronise plus
email-source-failure-heading = Une source d'agenda ne se synchronise plus
email-source-failure-intro = calrs n'arrive plus à synchroniser cette source d'agenda depuis plus d'une heure. Vos pages de réservation proposent toujours des créneaux, mais ils reposent sur la dernière synchronisation réussie : les événements ajoutés ou déplacés depuis ne sont pas pris en compte.
email-source-failure-action = Vérifiez la source sur la page Sources d'agenda : un mot de passe expiré ou un accès révoqué en est la cause habituelle. Vous ne recevrez de nouvelles de cette source que si elle se rétablit puis échoue à nouveau.
email-detail-source = Source d'agenda :
email-detail-failing-since = En échec depuis :
email-host-reminder-subject = Rappel : { $event }, { $guest } ({ $date })
email-host-reminder-heading = Réservation à venir
email-host-reminder-message = Rappel : vous avez bientôt une réservation avec { $guest }.
email-host-cancel-subject = Annulation : { $event }, { $guest } ({ $date })
email-host-cancel-message-by-host = Vous avez annulé cette réservation.
email-host-cancel-message-by-guest = { $guest } a annulé sa réservation.
email-host-approval-subject = Action requise : { $event }, { $guest } ({ $date })
email-host-approval-heading = Action requise
email-host-approval-message = { $guest } souhaite réserver un créneau avec vous.
email-host-approval-reminder-subject = Rappel : { $guest } attend votre réponse ({ $event }, { $date })
email-host-approval-reminder-heading = Rappel : action requise
email-host-approval-reminder-message = { $guest } attend toujours votre réponse. Sans réponse de votre part, la demande sera automatiquement refusée le { $deadline }.
email-host-approval-footer = Vous pouvez aussi gérer cette demande depuis votre tableau de bord.
email-host-expired-subject = Expiration : { $event } avec { $guest } ({ $date })
email-host-expired-heading = Demande de réservation expirée
email-host-expired-message = Personne n'a répondu à la demande de { $guest } avant l'échéance : elle a été refusée automatiquement et l'invité a été prié de choisir un autre créneau.
email-host-expired-footer = Vous pouvez modifier le délai d'attente des demandes dans les paramètres du type d'événement.
email-host-reschedule-request-subject = Demande de reprogrammation : { $event }, { $guest } ({ $email })
email-host-reschedule-request-message = { $guest } souhaite reprogrammer sa réservation.
email-watcher-claim-subject = À prendre en charge : { $event }, { $guest } ({ $date })
email-watcher-claim-message = Une nouvelle réservation peut être prise en charge. Cliquez ci-dessous pour y participer.
email-watcher-claim-link = Prendre en charge cette réservation : { $url }
email-watcher-claim-footer = Vous pouvez aussi la prendre en charge depuis votre tableau de bord.
email-claim-confirmed-subject = Réservation prise en charge : { $event }, { $guest } ({ $date })
email-claim-confirmed-footer = Vous serez ajouté comme participant à cette réunion.
email-attendee-greeting = Bonjour,
email-attendee-message = Vous avez été ajouté comme participant à une réservation.
email-attendee-rescheduled-message = Une réservation à laquelle vous participez a été déplacée à un nouvel horaire.
email-attendee-cancelled-message = Une réservation à laquelle vous participez a été annulée.
email-attendee-cancelled-subject = Annulation : { $event }
email-original-booking-cancelled = Votre réservation initiale est également annulée.
email-pick-new-time-subject = Reprogrammation : { $event }, merci de choisir un nouveau créneau
email-pick-new-time-message = { $host } doit reprogrammer votre réservation. Merci de choisir un nouveau créneau.
email-pick-new-time-link = Choisir un nouveau créneau : { $url }
email-rescheduled-message = Votre réservation a été reprogrammée par { $host }.
email-reschedule-approved-message = { $host } a confirmé le nouvel horaire de votre réservation.
email-rescheduled-ics-attached-plain = Une invitation mise à jour est jointe.
email-rescheduled-ics-attached-html = Une invitation mise à jour est jointe à cet e-mail.
email-need-to-reschedule = Besoin de reprogrammer ? { $url }
email-invite-subject = { $host } vous invite à réserver : { $event }
email-invite-message = { $host } vous invite à réserver : { $event }
email-invite-link = Cliquez sur le lien ci-dessous pour choisir un créneau :
email-invite-action = Choisir un créneau
email-cancel-reason-calendar-deleted = L'événement a été supprimé de l'agenda de l'hôte.

# Embed snippet generator (templates/embed.html)
embed-page-title = Intégration — { $title }
embed-heading = Intégration : { $title }
embed-subtitle = Générez un extrait de code pour intégrer ce type d'événement sur votre propre site web.
embed-warn-internal = Attention : ce type d'événement est <strong>interne</strong>. Les visiteurs du site intégré auront besoin d'un lien d'invitation valide pour réserver — l'extrait généré n'en contient pas. Envisagez de le rendre public pour une intégration directe.
embed-warn-private = Attention : ce type d'événement est <strong>privé</strong>. Les visiteurs du site intégré auront besoin d'un lien d'invitation valide pour réserver — l'extrait généré n'en contient pas. Envisagez de le rendre public pour une intégration directe.
embed-warn-private-team = Attention : cette équipe est <strong>privée</strong>. Les visiteurs du site intégré ne peuvent pas réserver sans le lien d'invitation de l'équipe — l'extrait généré n'en contient pas. Envisagez de rendre l'équipe publique pour une intégration directe.
embed-need-username = Vous devez définir un <strong>nom d'utilisateur</strong> avant de pouvoir intégrer ce type d'événement — le lien d'intégration est { $path }. { $link }, puis revenez ici.
embed-need-username-link = Définissez un nom d'utilisateur dans les paramètres
embed-username-placeholder = nom-utilisateur
embed-tab-inline = Intégré à la page
embed-tab-floating = Bouton flottant
embed-tab-element = Clic sur un élément
embed-layout = Disposition
embed-theme = Thème
embed-theme-auto = Automatique (selon le visiteur)
embed-brand-color = Couleur de la marque
embed-use-org-color = Utiliser la couleur de l'organisation
embed-button-text = Texte du bouton
embed-button-default = Prendre rendez-vous
embed-button-position = Position du bouton
embed-pos-bottom-right = En bas à droite
embed-pos-bottom-left = En bas à gauche
embed-pos-top-right = En haut à droite
embed-pos-top-left = En haut à gauche
embed-button-colors = Couleurs du bouton
embed-button-fg-placeholder = texte
embed-button-colors-hint = Fond / texte
embed-show-icon = Afficher l'icône de calendrier
embed-element-help = Insérez l'extrait une seule fois dans votre site, puis ajoutez les attributs data indiqués à tout élément cliquable (bouton, lien, div) qui doit ouvrir la fenêtre de réservation.
embed-snippet = Extrait de code
embed-loading = Chargement…
embed-preview = Aperçu en direct
embed-preview-back = retour à l'aperçu
embed-preview-floating-hint = Cliquez sur le bouton pour simuler la fenêtre de réservation
embed-preview-element-hint = Déclencheurs de démonstration — sur votre site, ajoutez les attributs data de l'extrait à n'importe quel élément. Cliquez sur l'un d'eux pour essayer la fenêtre :
embed-preview-text-link = ou cliquez sur ce lien texte
embed-snippet-attr-comment = Ajoutez ces attributs à tout élément cliquable de votre page :

# Leftovers in partly translated pages (host reschedule, impersonation banner, slots, team settings)
host-resched-when = { $date } à { $start } – { $end }
base-impersonating = Vous avez pris l'identité de { $name }
slots-rescheduling-banner = Reprogrammation : { $title } (actuellement : { $date } à { $time })
slots-invite-required-title = La réservation nécessite un lien d'invitation
team-settings-slug-url = Utilisé dans l'URL de l'équipe : { $url }
team-event-type-no-teams = Aucune équipe n'existe pour le moment.
team-event-type-not-team-admin = Vous n'êtes administrateur d'aucune équipe.

# Google Calendar connection errors (src/web/sources.rs)
google-error-not-configured = L'intégration Google Agenda n'est pas configurée. Demandez à votre administrateur de renseigner les identifiants OAuth2 Google dans le panneau d'administration.
google-error-no-base-url = L'URL publique de base n'est pas configurée. Définissez-la via la variable d'environnement CALRS_BASE_URL ou dans le panneau d'administration, section Paramètres système, avant d'utiliser les flux OAuth2.
google-error-authorization = L'autorisation Google a échoué : { $error }
google-error-token-exchange = L'échange de jeton a échoué : { $error }
google-error-account-email = Impossible de récupérer l'adresse e-mail du compte Google : { $error }

# Admin panel notices (src/web/admin.rs)
admin-resource-feed-invalid = URL de flux invalide : { $error }
admin-resource-feed-check-failed = La vérification du flux a échoué : { $error }
admin-resource-caldav-invalid = URL CalDAV invalide : { $error }
admin-resource-credentials-failed = Impossible d'enregistrer les identifiants : { $error }
admin-resource-added = Ressource « { $name } » ajoutée. Événements en cache : { $count }.
admin-resource-updated = Ressource mise à jour. Événements en cache : { $count }.
admin-resource-deleted = Ressource supprimée.
admin-resource-not-found = Ressource introuvable.
admin-resource-synced = Flux synchronisé. Événements en cache : { $count }.
admin-resource-sync-error = La synchronisation a échoué : { $error }
admin-resource-no-caldav-url = Aucune URL CalDAV n'est configurée et aucune n'a pu être déduite de l'URL du flux.
admin-resource-no-write-credentials = Aucun identifiant d'écriture disponible (pas de compte de service, et aucun membre ayant accepté le prêt ne possède de source de calendrier correspondante).
admin-resource-write-passed = Test d'écriture réussi avec { $user } (événement créé : { $created }, supprimé : { $cleaned }).
admin-resource-write-verified = vérifié
admin-resource-write-unverified = non vérifié
admin-resource-write-check-calendar = vérifiez le calendrier de la ressource
admin-resource-write-failed = Le test d'écriture a échoué. Dernière erreur : { $error }
admin-base-url-invalid = L'URL de base doit être une URL http(s) absolue, par ex. https://cal.example.com
admin-company-link-invalid = Le lien de l'entreprise doit commencer par http:// ou https://
admin-smtp-host-required = L'hôte SMTP est obligatoire.
admin-smtp-from-invalid = Veuillez saisir une adresse e-mail d'expéditeur valide.
admin-smtp-port-invalid = Le port SMTP doit être un nombre compris entre 1 et 65535.
admin-smtp-tls-invalid = Le mode TLS doit être « starttls », « tls » ou « none ».
admin-smtp-password-required = Un mot de passe est requis lors de la première configuration SMTP.
admin-sms-switch-needs-secret = Pour changer de passerelle SMS, vous devez saisir l'identifiant secret de la nouvelle passerelle.
admin-sms-settings-invalid = Paramètres SMS : { $error }.
admin-sms-sender-format = Un expéditeur SMS numérique doit être au format international, par ex. +33612345678.
admin-sms-not-configured = Les SMS ne sont pas configurés.
admin-sms-load-failed = Impossible de charger la configuration SMS.
admin-sms-check-accepted = Identifiants acceptés par la passerelle.
admin-sms-bad-number = Ce numéro de téléphone n'est pas utilisable.
admin-sms-test-body = Message de test calrs. Votre passerelle SMS fonctionne.
admin-sms-test-accepted = Message de test accepté par la passerelle.

## Local multi-factor authentication

mfa-title = Authentification à deux facteurs
mfa-invalid = La vérification a échoué. Vérifiez votre mot de passe ou votre code. Chaque code ne fonctionne qu’une fois ; attendez un nouveau code si nécessaire. Après trop de tentatives, attendez 15 minutes. Si la configuration a expiré, recommencez.
mfa-restart = Retour à la connexion
mfa-manage = Gérer l’authentification à deux facteurs
mfa-recovery-help = Conservez ces codes de récupération en lieu sûr. Ils ne sont affichés qu’une fois. Chaque code remplace un code de votre application pour une seule utilisation. Les nouveaux codes remplacent tous les anciens.
mfa-continue = Continuer vers le tableau de bord
mfa-enabled = L’authentification à deux facteurs est activée.
mfa-disabled = L’authentification à deux facteurs est désactivée.
mfa-required-help = Votre administrateur impose l’authentification à deux facteurs aux comptes locaux. Vous ne pouvez pas la désactiver.
mfa-code = Code de l’application ou code de récupération
mfa-regenerate = Générer de nouveaux codes de récupération
mfa-disable = Désactiver l’authentification à deux facteurs
mfa-enable = Configurer l’authentification à deux facteurs
mfa-changes-help = Les modifications nécessitent votre mot de passe. La désactivation ou le remplacement des codes de récupération nécessite aussi un code et déconnecte vos autres sessions.
mfa-back = Retour aux paramètres
mfa-setup-help = Scannez ce QR code avec votre application d’authentification, ou saisissez la clé ci-dessous. Saisissez un code à six chiffres pour terminer. La configuration expire après 10 minutes.
mfa-qr-alt = QR code de configuration de l’application
mfa-login-help = Saisissez un code de votre application d’authentification ou un de vos codes de récupération. Cette connexion expire après 10 minutes.
mfa-totp-code = Code à six chiffres de l’application
mfa-verify = Vérifier
mfa-policy-help = Cette règle concerne les comptes locaux. Le SSO utilise l’authentification multifacteur du fournisseur d’identité. Les administrateurs locaux doivent configurer la leur avant de modifier cette règle. La rendre obligatoire déconnecte les sessions locales sans validation multifacteur et impose la configuration à la prochaine connexion.
mfa-policy = Règle MFA des comptes locaux
mfa-optional = Facultative
mfa-required = Obligatoire
mfa-save-policy = Enregistrer la règle MFA

# First day of the week (#330)
week-start-label = Premier jour de la semaine
week-start-inherit-site = Utiliser le réglage du site
week-start-inherit-owner = Utiliser le réglage du propriétaire

## Booking directory and personal favorites

directory-title = Annuaire des rendez-vous
directory-help = Trouvez le lien de rendez-vous d’un collègue ou d’une équipe. Ajoutez les rendez-vous que vous partagez souvent aux favoris pour les retrouver sur votre tableau de bord.
directory-search = Rechercher par personne, équipe ou titre de rendez-vous
directory-find = Rechercher
directory-favorites-only = Favoris uniquement
directory-favorites = Rendez-vous favoris
directory-browse = Parcourir l’annuaire
directory-generate = Générer un lien d’invitation
directory-invite-help = Invitation à usage unique, valable 7 jours.
directory-add = Ajouter { $title } aux favoris
directory-remove = Retirer { $title } des favoris
directory-added = Ajouté aux favoris.
directory-removed = Retiré des favoris.
directory-error = Cette action n’a pas pu être effectuée. Actualisez la page et réessayez.
directory-manual-copy = Copiez ce lien manuellement :
directory-empty = Aucun rendez-vous trouvé. Essayez une autre recherche.
directory-favorites-empty = Aucun favori trouvé. Ajoutez des rendez-vous aux favoris depuis l’annuaire pour les retrouver ici.
directory-with-hosts = Avec { $names }

## ilia fork: TON payments (docs/ton.md)

# Price on the slot and booking pages
ton-slots-price = Prix : ${ $usd }
ton-slots-price-approx = (≈ { $ton } TON)
ton-slots-price-note = à payer en TON après le choix du créneau

# Event type form
ton-etf-section = Paiement en TON
ton-etf-off-note = Les paiements en TON ne sont pas configurés sur ce serveur. Un type de rendez-vous payant ne peut pas être réservé tant qu’ils ne le sont pas.
ton-etf-enabled = Réservation payante
ton-etf-enabled-help = L’invité paie en TON juste après avoir choisi un créneau. Le créneau lui est réservé pendant le paiement, et une réservation impayée est libérée automatiquement.
ton-etf-price = Prix
ton-etf-price-help = En USD : de 1 $ à 10000 $ ; le montant en TON est calculé au taux de change du moment de la réservation.
ton-etf-window = Délai de paiement (minutes)
ton-etf-window-help = De 10 à 240. Laissez vide pour la valeur du serveur ({ $minutes } min).
ton-form-error-price = Saisissez un prix en dollars américains entre 1 et 10000, avec au plus deux décimales.
ton-form-error-window = Le délai de paiement doit être compris entre 10 et 240 minutes.
ton-form-error-team = Les types de rendez-vous d’équipe ne peuvent pas être payants.

# Booking error pages
ton-bae-title-unavailable = Ce créneau ne peut pas être réservé
ton-bae-body-unavailable = Le paiement est momentanément indisponible, ce créneau ne peut donc pas être réservé pour l’instant. Veuillez réessayer dans quelques minutes.
ton-bae-body-too-late = Il reste trop peu de temps avant ce rendez-vous pour le payer. Veuillez choisir un créneau plus tardif.
ton-bae-body-already-open = Vous avez déjà une réservation impayée avec cette adresse e-mail. Nous pouvons vous envoyer ses liens : vous pourrez la payer, ou l’annuler pour libérer le créneau, puis réserver à nouveau.
ton-bae-body-personal-only = Ce rendez-vous ne peut être réservé que sur la page de son organisateur.
ton-bae-title-unpaid = Réservation impayée
ton-bae-body-unpaid = Cette réservation n’est pas payée, elle ne peut donc pas être approuvée. Pour la confirmer gratuitement, utilisez « Confirmer sans paiement » sur la page des réservations de votre tableau de bord.
ton-bae-body-reschedule-unpaid = Cette réservation n’est pas payée, elle ne peut donc pas être déplacée. Payez-la, ou annulez-la et réservez un autre créneau.
ton-bae-body-pay-link-invalid = Ce lien de paiement n’est pas valide.

# Payment page
ton-pay-page-title = Paiement : { $title }
ton-pay-held-until = Le créneau vous est réservé jusqu’à { $time } ({ $zone }).
ton-pay-left = encore %1
ton-pay-amount-label = Montant à payer
ton-pay-rate = ≈ { $usd } $ · 1 TON = { $rate } $ ({ $sources }, { $time } UTC)
ton-pay-open-wallet = Ouvrir le portefeuille
ton-pay-open-tonkeeper = Ouvrir dans Tonkeeper
ton-pay-qr-alt = QR code contenant les informations de paiement
ton-pay-scan = Ou scannez ce code avec l’application de votre portefeuille.
ton-pay-address = Adresse
ton-pay-comment = Commentaire
ton-pay-copy = Copier
ton-pay-copied = Copié
ton-pay-warn-comment = Le commentaire est obligatoire : c’est grâce à lui que nous reconnaissons votre paiement.
ton-pay-warn-encrypt = Ne chiffrez pas le commentaire.
ton-pay-warn-token = Payez en TON, pas en jetons.
ton-pay-warn-exchange = Vous payez depuis une plateforme d’échange ? Indiquez le code ci-dessus dans le champ « mémo » (commentaire).
ton-pay-warn-deadline = Envoyez le paiement avant { $time }. Un paiement envoyé plus tard risque de ne pas être pris en compte.
ton-pay-check = J’ai payé, vérifier
ton-pay-email = M’envoyer ce lien par e-mail
ton-pay-emailed = Nous avons envoyé ce lien à l’adresse e-mail indiquée lors de la réservation.
ton-pay-email-unavailable = Le lien n’a pas pu être envoyé. Gardez cette page ouverte ou enregistrez son adresse.
ton-pay-checking-now = Nous vérifions votre paiement. Cette page se met à jour automatiquement.
ton-pay-state-waiting = En attente de votre paiement.
ton-pay-state-seen = Votre paiement est visible sur le réseau. Nous attendons qu’il soit définitif, ce qui prend généralement quelques secondes.
ton-pay-state-partial = Nous avons reçu { $paid } TON. Veuillez payer les { $rest } TON restants avec le même commentaire.
ton-pay-state-checking = Le délai de paiement est écoulé. Si vous avez déjà envoyé le paiement, nous le vérifions. Cette page se met à jour automatiquement.
ton-pay-state-paid = Paiement reçu. Confirmation de votre réservation…
ton-pay-state-confirmed = Payé. Votre réservation est confirmée et l’e-mail de confirmation est en route.
ton-pay-state-waived = Aucun paiement n’est nécessaire pour cette réservation. L’organisateur la confirme.
ton-pay-state-busy = Paiement reçu, mais ce créneau n’est plus libre. Nous vous contacterons.
ton-pay-state-expired = Le délai de paiement est écoulé et le créneau a été libéré. Si vous avez tout de même envoyé un paiement, écrivez-nous depuis l’adresse e-mail de votre réservation et nous vous le rembourserons.
ton-pay-state-late = Votre paiement est arrivé après le délai, la réservation n’a donc pas été confirmée. Nous vous contacterons pour le remboursement. Vous pouvez aussi nous écrire depuis l’adresse e-mail de votre réservation.
ton-pay-state-closed = Cette réservation n’est plus active.
ton-pay-state-revoked = Cette adresse de paiement n’est plus utilisée. N’y envoyez rien et réservez à nouveau.
ton-pay-rebook = Réserver un autre créneau

# Leaving the embedded widget
ton-exit-title = Créneau réservé
ton-exit-body = Le créneau vous est réservé jusqu’à { $time } ({ $zone }). Effectuez le paiement pour confirmer votre réservation.
ton-exit-go = Aller au paiement

# Dashboard
ton-dash-badge-awaiting = en attente de paiement · { $amount } TON · jusqu’à { $time }
ton-dash-badge-partial = payé en partie · { $paid } sur { $amount } TON · jusqu’à { $time }
ton-dash-badge-paid = payé · { $amount } TON
ton-dash-badge-overpaid = payé · trop-perçu de { $excess } TON
ton-dash-badge-busy = payé, créneau plus libre
ton-dash-badge-confirm-failed = payée, non confirmée · à confirmer à la main
ton-dash-badge-refund = payé { $amount } TON · remboursement dû
ton-dash-badge-waived = confirmé sans paiement
ton-dash-badge-unpaid = impayé
ton-dash-waive = Confirmer sans paiement
ton-dash-error-unpaid = Cette réservation n’est pas payée, elle ne peut donc être ni confirmée ni déplacée. Utilisez « Confirmer sans paiement » pour la confirmer gratuitement, ou refusez-la.
ton-dash-error-waive-failed = La réservation n’a pas pu être confirmée sans paiement : elle n’est plus en attente de paiement.
ton-waive-page-title = Confirmer sans paiement
ton-waive-heading = Confirmer cette réservation sans paiement ?
ton-waive-body = Cette réservation n’est PAS payée ({ $amount } TON, à régler avant { $time }). Si vous la confirmez, l’invité obtient le rendez-vous gratuitement, et un paiement arrivant plus tard devra être remboursé à la main.
ton-waive-submit = Confirmer gratuitement
ton-waive-back = Retour aux réservations

# Guest emails
ton-email-link-subject = Finalisez votre réservation : { $event }
ton-email-link-message = Vous avez commencé à réserver { $event } avec { $host }. Finalisez votre réservation sur la page ci-dessous avant la fin du délai.
ton-email-link-action = Finaliser la réservation
ton-email-link-line = Finalisez votre réservation : { $url }
ton-email-expired-subject = Non réservé : { $event }, { $date }
ton-email-expired-message = Le paiement pour { $event } n’est pas arrivé à temps, le créneau a donc été libéré. Vous pouvez choisir un autre créneau.
ton-email-expired-partial = Nous avons reçu une partie du paiement. Nous vous contacterons pour le remboursement.
ton-email-rebook-line = Choisir un autre créneau : { $url }

# Owner emails
ton-email-row-amount = Montant :
ton-email-row-paid = Reçu :
ton-email-row-excess = Trop-perçu :
ton-email-row-transfer = Transfert :
ton-email-row-sender = Expéditeur :
ton-email-row-tx = Transaction :
ton-email-row-detail = Détails :
ton-email-amount-value = { $amount } TON ≈ { $usd } $ (1 TON = { $rate } $)
ton-email-sender-wallet = { $address } (un portefeuille)
ton-email-sender-other = { $address } (pas un portefeuille : une plateforme d’échange ou un contrat)
ton-email-dashboard-line = Réservations : { $url }
ton-email-dashboard-action = Ouvrir les réservations
ton-email-refund-rule = Les remboursements se font à la main depuis votre portefeuille : vers l’adresse de l’expéditeur si c’est un portefeuille ; sinon uniquement vers une adresse que l’invité demande depuis l’adresse e-mail de sa réservation, dans un message contenant son lien de paiement ou d’annulation. Le code de réservation, le montant et la transaction sont publics et ne prouvent rien.
ton-email-received-subject = Paiement reçu : { $event }, { $guest } ({ $date })
ton-email-received-heading = Paiement reçu
ton-email-received-message = { $guest } a payé { $amount } TON. La réservation est confirmée.
ton-email-alert-subject = Un paiement en TON demande votre attention : { $what }
ton-email-alert-heading = Un paiement demande votre attention
ton-alert-partial = paiement partiel
ton-alert-partial-body = Une partie du montant dû est arrivée. La réservation reste non confirmée jusqu’à l’arrivée du reste ; sinon, elle est libérée à l’échéance et le montant reçu doit être remboursé.
ton-alert-overpaid = trop-perçu
ton-alert-overpaid-body = Un paiement est arrivé pour une réservation déjà payée. Remboursez-le si cela en vaut la peine.
ton-alert-late = paiement tardif
ton-alert-late-body = Un paiement est arrivé après son échéance. La réservation était déjà libérée et ne peut plus être confirmée. Remboursez le paiement, ou demandez à l’invité de réserver à nouveau et confirmez la nouvelle réservation sans paiement.
ton-alert-unmatched = paiement non reconnu
ton-alert-unmatched-body = Un paiement est arrivé sans code de réservation connu. Retrouvez la réservation à laquelle il correspond, ou remboursez-le.
ton-alert-failed = transfert échoué
ton-alert-failed-body = Un transfert vers l’adresse de paiement a échoué sur le réseau. Les fonds sont peut-être encore sur votre portefeuille : vérifiez la transaction dans l’explorateur.
ton-alert-not-text = transfert sans commentaire lisible
ton-alert-not-text-body = Un transfert avec un commentaire chiffré, un jeton ou un autre contenu est arrivé. Il n’est jamais pris en compte automatiquement : vérifiez-le à la main.
ton-alert-other = transfert inhabituel
ton-alert-other-body = Un transfert n’a pas pu être traité automatiquement. Vérifiez-le dans le journal des paiements.
ton-email-problem-subject = Problème de paiement en TON : { $event } ({ $date })
ton-email-problem-urgent-subject = Urgent : une réservation payée attend votre décision : { $event } ({ $date })
ton-email-problem-heading = Une réservation payée demande votre attention
ton-problem-host-busy = L’invité a payé, mais le créneau n’est plus libre (un événement du calendrier ou une autre réservation). Le créneau reste réservé jusqu’au début du rendez-vous. Demandez à l’invité un nouveau créneau, confirmez la réservation à la main, ou refusez-la et remboursez le paiement.
ton-problem-partial = Seule une partie du montant a été payée, puis la réservation a expiré, a été annulée ou refusée, ou a été confirmée sans paiement. Remboursez le montant reçu, sauf accord contraire avec l’invité.
ton-problem-overpaid = La réservation est payée et confirmée avec plus que le montant dû. Remboursez l’excédent si cela en vaut la peine.
ton-problem-refund-due = La réservation a été payée, puis annulée ou refusée. Remboursez le paiement.
ton-problem-confirmed-unpaid = Une réservation impayée a été trouvée confirmée. Cela ne devrait jamais arriver : vérifiez comment elle a été confirmée.
ton-problem-address-revoked = L’adresse de paiement de cette réservation a été révoquée, et la réservation sera libérée.
ton-email-health-subject = Les paiements en TON demandent votre attention ({ $kind })
ton-email-health-heading = Les paiements en TON demandent votre attention
ton-health-indexer-down = Le suivi des paiements ne parvient pas à joindre l’indexeur. Les paiements ne sont pas vérifiés et les nouvelles réservations payantes sont refusées jusqu’au rétablissement.
ton-health-auth-failed = L’indexeur a refusé la clé d’API (incorrecte, ou destinée à l’autre réseau). Vérifiez CALRS_TON_API_KEY.
ton-health-wrong-network = L’indexeur répond pour un autre réseau que celui configuré. Rien n’est crédité ni expiré tant que ce n’est pas corrigé.
ton-health-wallet-inactive = Le portefeuille de réception n’est pas actif. Les réservations payantes sont refusées tant qu’il n’a pas fait au moins un transfert sortant.
ton-health-lagging = Le suivi des paiements prend du retard sur les transferts reçus à l’adresse (peut-être du spam). Les réservations impayées n’expirent pas tant qu’il n’a pas rattrapé son retard.
ton-health-stuck-finality = Un transfert n’est pas devenu définitif depuis longtemps. Le suivi l’attend avant de lire la suite.
ton-health-watcher-crashed = Le suivi des paiements a planté plusieurs fois et redémarre sans cesse. Consultez le journal du serveur.
ton-health-disabled-with-open = Les paiements en TON sont désactivés alors que des réservations impayées sont ouvertes. Leurs créneaux seront libérés à leur échéance.
ton-health-other-network-open = Des réservations sont ouvertes sur un autre réseau TON que celui configuré. Elles seront libérées à leur échéance.
ton-health-rate-unavailable = Aucun taux de change TON acceptable n’est disponible (sources absentes, divergentes ou hors de votre fourchette). Les réservations payantes sont refusées jusqu’à ce qu’il y en ait un.
ton-health-capacity = La limite de réservations impayées ouvertes ou quotidiennes est atteinte. Les nouvelles réservations payantes sont refusées pour l’instant.
ton-health-other = Le suivi des paiements a signalé un problème. Consultez le journal du serveur.

# TON follow-ups: releases, refunds, contact, resend
ton-resend-submit = M’envoyer les liens
ton-resend-title = Consultez vos e-mails
ton-resend-done = S’il existe une réservation impayée pour cette adresse e-mail, nous y avons envoyé ses liens de paiement et d’annulation.
ton-pay-state-taken = Ce créneau a été pris avant que la réservation soit payée ; elle a donc été libérée. Vous n’avez rien à payer. Veuillez choisir un autre créneau.
ton-pay-state-refund = Cette réservation s’est terminée sans être confirmée. Les { $paid } TON payés vous seront remboursés ; l’hôte vous contactera.
ton-pay-contact = Des questions ? Écrivez à { $host } : { $email }.
ton-pay-copy-address = Copier l’adresse
ton-pay-copy-comment = Copier le commentaire
ton-pay-copy-manual = La copie n’a pas fonctionné. Le texte est sélectionné : copiez-le à la main.
ton-email-row-pay-by = À payer avant :
ton-email-taken-message = Le créneau choisi pour { $event } a été pris avant que la réservation soit payée ; elle a donc été libérée. Vous n’avez rien à payer. N’hésitez pas à choisir un autre créneau.
ton-email-refund-message = Nous avons bien reçu votre paiement pour { $event }, mais le créneau n’a pas pu être confirmé ; la réservation a donc été libérée. { $host } vous remboursera et vous contactera.
ton-problem-host-busy-unpaid = Le créneau de cette réservation impayée a été pris (une demande a été approuvée par-dessus, ou un événement est apparu dans le calendrier) ; la réservation a donc été libérée avant le paiement. Si quelque chose a été reçu, remboursez-le.
ton-problem-confirm-failed = L’invité a payé, mais la réservation n’a pas pu être confirmée automatiquement après plusieurs tentatives (une erreur, pas un hôte occupé : voir le journal du serveur). Le créneau reste réservé jusqu’au début du rendez-vous. Confirmez la réservation à la main, ou refusez-la et remboursez le paiement.
ton-health-capacity-near = Une nouvelle réservation payante a pris la dernière place ouverte ou le dernier quota du jour. Les suivantes seront refusées jusqu’à ce que des réservations soient payées ou libérées. S’il ne s’agit pas d’une vraie demande, quelqu’un bloque peut-être des créneaux sans payer.
ton-health-payment-alerts-held = D’autres virements sont arrivés pour des réservations au sujet desquelles vous avez déjà reçu un e-mail dans l’heure. Ils sont enregistrés sans e-mail individuel : consultez le journal des paiements.
ton-waive-body-partial = Cette réservation n’est payée qu’en partie ({ $paid } sur { $amount } TON reçus, à régler avant { $time }). Si vous la confirmez, l’invité obtient le rendez-vous sans payer les { $rest } TON restants. Remboursez le montant reçu, sauf accord contraire.

## Telegram (ilia fork)

tg-when = { $date }, { $time } ({ $tz })
tg-in-minutes =
    { $count ->
        [one] dans { $count } minute
       *[other] dans { $count } minutes
    }
tg-in-hours =
    { $count ->
        [one] dans { $count } heure
       *[other] dans { $count } heures
    }
tg-in-days =
    { $count ->
        [one] dans { $count } jour
       *[other] dans { $count } jours
    }
tg-reason = Motif : { $reason }
tg-link-line = { $label } : { $url }
tg-guest-confirmed = Réservation confirmée : { $event }, { $when }. Organisateur : { $host }.
tg-guest-pending = Demande envoyée : { $event }, { $when }. Vous recevrez la réponse de l'organisateur ici.
tg-guest-approved = Votre demande est confirmée : { $event }, { $when }.
tg-guest-reschedule-approved = Votre nouvel horaire est confirmé : { $event }, { $when }.
tg-guest-declined = Votre demande a été refusée : { $event }, { $when }.
tg-guest-cancelled-by-host = L'organisateur a annulé : { $event }, { $when }.
tg-guest-cancelled-by-guest = Vous avez annulé votre réservation : { $event }, { $when }.
tg-guest-rescheduled = Réservation déplacée : { $event } est maintenant le { $when }.
tg-guest-reschedule-pending = Nouvel horaire demandé : { $event }, { $when }. Vous recevrez la réponse de l'organisateur ici.
tg-guest-reschedule-requested = L'organisateur vous demande de choisir un autre horaire pour { $event }. Initialement : { $when }.
tg-guest-reminder = Rappel : { $event } commence { $in }, { $when }.
tg-guest-expired = Demande expirée : { $event }, { $when }. L'organisateur n'a pas répondu à temps ; veuillez réserver un autre créneau.
tg-button-cancel = Annuler la réservation
tg-button-reschedule = Reprogrammer
tg-button-calendar = Ajouter au calendrier
tg-button-pick-time = Choisir un nouvel horaire
tg-button-rebook = Réserver un autre créneau
tg-button-bookings = Voir les réservations
tg-host-new-booking = Nouvelle réservation : { $event }, { $when }. Invité : { $guest }, { $email }.
tg-host-approval-request = Demande à approuver : { $event }, { $when }. Invité : { $guest }, { $email }.
tg-host-reschedule-approval = Nouvel horaire à approuver : { $event }, { $when }. Invité : { $guest }, { $email }.
tg-host-rescheduled = { $guest } a déplacé { $event } au { $when }.
tg-host-cancelled = { $guest } a annulé : { $event }, { $when }.
tg-host-guest-linked = { $guest } recevra les notifications Telegram pour { $event }, { $when } : { $username }.
tg-host-guest-linked-differs = Saisi dans le formulaire : { $entered }.
tg-host-no-username = sans nom d'utilisateur
tg-bot-help = Ce bot envoie des notifications sur les réservations de { $site }. Pour les activer, ouvrez le lien de votre page de confirmation ou de votre e-mail ; pour une réservation payante, le lien est sur sa page de paiement. /stop les désactive. Une fois une réservation reliée à ce chat, vous pouvez aussi écrire ici à son organisateur.
tg-bot-linked-guest = C'est fait : les notifications pour cette réservation arriveront ici.
tg-bot-linked-host = C'est fait : les notifications d'organisateur pour { $name } arriveront ici.
tg-bot-link-invalid = Ce lien ne fonctionne plus : il n'est valable que dans un seul chat Telegram et seulement jusqu'à la fin du rendez-vous.
tg-bot-link-replaced = Ce lien a été remplacé. Rouvrez la page de paiement de votre réservation et utilisez le lien Telegram qui s'y trouve.
tg-bot-rate-limited = Trop de tentatives. Veuillez réessayer plus tard.
tg-bot-stopped = Les notifications sont désactivées. Pour les réactiver pour une réservation, rouvrez son lien depuis l'e-mail de confirmation ; les organisateurs créent un nouveau lien dans Profil et paramètres → Telegram.
tg-bot-not-linked = Cette conversation ne reçoit aucune notification.
tg-cmd-help = Fonctionnement du bot
tg-cmd-stop = Désactiver les notifications
tg-book-label = Telegram
tg-book-placeholder = @nom_utilisateur
tg-book-help-optional = Après la réservation, vous pourrez activer les notifications dans Telegram.
tg-book-help-required = Après la réservation, ouvrez le lien Telegram pour recevoir les notifications.
tg-book-invalid-title = Vérifiez votre nom d'utilisateur Telegram
tg-book-invalid = Saisissez un nom d'utilisateur Telegram comme @nom (lettres, chiffres et _, de 4 à 32 caractères).
tg-book-required = Saisissez votre nom d'utilisateur Telegram.
tg-confirmed-title = Recevoir les notifications dans Telegram
tg-confirmed-body = Ouvrez le bot et appuyez sur le bouton en bas du chat : les notifications pour cette réservation y arriveront.
tg-confirmed-button = Ouvrir Telegram
tg-email-button = Notifications dans Telegram
tg-email-plain = Recevoir les notifications dans Telegram : { $url }
tg-settings-nav = Notifications Telegram
tg-settings-title = Telegram
tg-settings-intro = Recevez les notifications d'organisateur dans votre propre conversation Telegram : nouvelles réservations, demandes à approuver, annulations et déplacements par les invités.
tg-settings-off = Telegram n'est pas configuré sur ce serveur.
tg-settings-no-bot-name = Le bot n'est pas encore prêt. Veuillez réessayer dans une minute.
tg-settings-linked = Les notifications d'organisateur vont à { $username }. Lié : { $date }.
tg-settings-not-linked = Votre Telegram n'est pas lié.
tg-settings-link = Lier mon Telegram
tg-settings-link-open = Ouvrir Telegram
tg-settings-link-expires = Dans le chat, appuyez sur le bouton en bas. Le lien fonctionne une fois dans les 15 minutes.
tg-settings-link-limited = Trop de liens cette heure-ci. Veuillez réessayer plus tard.
tg-settings-unlink = Délier
tg-settings-modes = Telegram de l'invité par type de rendez-vous
tg-settings-modes-help = Facultatif ou obligatoire : le formulaire demande le Telegram de l'invité, et la confirmation propose le lien vers le bot.
tg-settings-modes-none = Aucun type de rendez-vous à régler.
tg-settings-mode-off = Désactivé
tg-settings-mode-optional = Facultatif
tg-settings-mode-required = Obligatoire
tg-settings-save = Enregistrer
tg-settings-saved = Enregistré.
tg-settings-team = Équipe { $team }
tg-dashboard-label = Telegram { $username }
tg-dashboard-entered = (saisi : { $entered })
tg-dashboard-not-linked = (pas encore lié)
tg-bot-already-linked = Déjà activé : les notifications pour cette réservation arrivent dans ce chat.
tg-bot-link-invalid-host = Ce lien a expiré : les liens des paramètres ne servent qu'une fois dans les 15 minutes. Créez-en un nouveau dans Profil et paramètres → Telegram.
tg-settings-linked-no-username = Les notifications d'organisateur vont à votre chat Telegram. Lié : { $date }.
tg-bot-description = Ce bot envoie des notifications sur vos réservations sur { $site } : confirmations, changements et rappels. Appuyez sur le bouton ci-dessous pour les activer pour votre réservation. /stop les désactive.
tg-bot-short-description = Notifications de réservation de { $site }.
tg-button-join = Rejoindre

## Telegram: TON payments (ilia fork; docs/ton.md)

tg-guest-payment-due = Votre réservation attend le paiement : { $event }, { $when }. Montant : { $amount } TON (≈ { $usd } $). Payez avant le { $deadline }, sinon le créneau sera libéré.
tg-guest-payment-partial = Une partie du paiement pour { $event }, { $when } est arrivée : { $paid } sur { $amount } TON. Payez les { $rest } TON restants avant le { $deadline }, sinon le créneau sera libéré.
tg-guest-payment-received = Paiement reçu, votre réservation est confirmée : { $event }, { $when }. Organisateur : { $host }.
tg-guest-payment-late = Un paiement de { $value } TON pour { $event }, { $when } est arrivé après la date limite ; il n’a donc pas confirmé la réservation. { $host } vous contactera au sujet du remboursement.
tg-guest-payment-expired = Le délai de paiement pour { $event }, { $when } est écoulé ; le créneau a donc été libéré. Vous pouvez réserver un autre créneau.
tg-guest-payment-taken = Le créneau que vous aviez choisi pour { $event }, { $when } a été pris avant le paiement de la réservation ; la réservation a donc été libérée. Vous n’avez rien à payer. Vous pouvez réserver un autre créneau.
tg-guest-payment-refund = Nous avons bien reçu votre paiement pour { $event }, { $when }, mais le créneau n’a pas pu être confirmé ; la réservation a donc été libérée. { $host } vous remboursera et vous contactera.
tg-guest-payment-refund-part = Une partie du paiement était déjà arrivée : { $host } vous contactera au sujet du remboursement.
tg-guest-payment-held = Nous avons bien reçu votre paiement pour { $event }, { $when }, mais la réservation n’a pas pu être confirmée automatiquement. { $host } vous contactera.
tg-button-pay = Payer
tg-host-payment-received = Payé : { $event }, { $when }. Invité : { $guest }, { $email }. { $paid } TON reçus (≈ { $usd } $) ; la réservation est confirmée.
tg-host-payment-problem = Paiement à vérifier : { $event }, { $when }. Invité : { $guest }, { $email }.
tg-host-payment-transfer = Virement : { $value } TON.
tg-host-payment-amounts = Reçu : { $paid } sur { $amount } TON.
tg-host-payment-guest-telegram = Telegram de l'invité : { $username }.

## ilia fork: booking flow v2 (docs/ton.md, docs/operations.md)
troubleshoot-label-busy = Occupé
ton-etf-currency = Devise
ton-etf-currency-ton = TON
ton-etf-currency-usd = USD (converti en TON)
ton-etf-price-help-ton = En TON : de 1 à 100000, l’invité paie exactement ce montant (100 TON si le champ est vide).
ton-etf-usd-no-rates = Les taux de change ne sont pas configurés sur ce serveur (CALRS_TON_RATE_MIN_USD et CALRS_TON_RATE_MAX_USD) : un prix en USD ne peut pas être réservé. Indiquez le prix en TON.
ton-form-error-price-ton = Saisissez un prix en TON entre 1 et 100000, avec au plus deux décimales.
ton-form-error-currency = Choisissez TON ou USD comme devise du prix.
ton-form-error-telegram = Pour le Telegram de l’invité, choisissez désactivé, facultatif ou obligatoire.
ton-etf-telegram = Telegram de l’invité
ton-etf-telegram-help = Obligatoire par défaut pour une réservation payante : l’invité reçoit alors les rappels de paiement et peut être joint sur Telegram. Vous pouvez le rendre facultatif ou le désactiver.
ton-slots-price-ton = Prix : { $ton } TON
ton-email-amount-value-ton = { $amount } TON
tg-guest-payment-due-ton = Votre réservation attend le paiement : { $event }, { $when }. Montant : { $amount } TON. Payez avant le { $deadline }, sinon le créneau sera libéré.
tg-host-payment-received-ton = Payé : { $event }, { $when }. Invité : { $guest }, { $email }. { $paid } TON reçus ; la réservation est confirmée.
ton-pay-renewed = Le délai a été prolongé ({ $n } sur { $max }).
ton-dash-badge-renewed = délai prolongé { $n } sur { $max }
ton-email-renewed-subject = Votre créneau est toujours réservé : { $event }
ton-email-renewed-message = Nous n’avons pas encore reçu le paiement pour { $event } avec { $host } : votre créneau reste réservé jusqu’au { $deadline }. Si vous avez déjà payé, vous n’avez rien à faire : le paiement sera pris en compte. Sinon, payez d’ici là sur la page ci-dessous, ou annulez la réservation si vos projets ont changé.
tg-guest-payment-renewed = Le paiement pour { $event }, { $when } n’est pas encore arrivé : le créneau vous reste réservé jusqu’au { $deadline }. Payez les { $rest } TON restants d’ici là, sinon le créneau sera libéré. Si vous avez déjà payé, vous n’avez rien à faire : le paiement sera pris en compte.

## ilia fork: messages with guests (docs/operations.md)
msg-page-title = Messages
msg-back = Réservations
msg-guest = Invité : { $guest }
msg-sent = Message envoyé.
msg-empty = Aucun message pour l’instant.
msg-label = Message à l’invité
msg-help-email = L’invité le reçoit par e-mail depuis l’adresse du système ; ses réponses s’affichent ici et dans votre Telegram.
msg-help-no-email = L’e-mail n’est pas configuré sur ce serveur.
msg-help-telegram = L’invité a relié Telegram et le reçoit aussi là-bas.
msg-help-no-telegram = L’invité n’a pas relié Telegram.
msg-help-limits = Au plus 20 messages par réservation et par jour, dont 5 par e-mail.
msg-send = Envoyer
msg-link = Messages
msg-link-count = Messages ({ $count })
msg-from-host = Organisateur
msg-from-guest = Invité
msg-from-attendee = Participant : { $address }
msg-from-other = Autre adresse, pas celle de l'invité : { $address }
msg-via-dashboard = tableau de bord
msg-via-mcp = agent
msg-via-telegram = Telegram
msg-status-email-queued = e-mail en file d’attente
msg-status-email-sent = e-mail envoyé
msg-status-email-failed = e-mail non distribué
msg-status-telegram-queued = Telegram en file d’attente
msg-status-telegram-sent = envoyé sur Telegram
msg-status-telegram-failed = non distribué sur Telegram
msg-error-empty = Écrivez d’abord un message.
msg-error-too-long = Un message peut contenir au plus 2000 caractères.
msg-error-limit = La limite quotidienne de messages pour cette réservation est atteinte. Réessayez demain.
msg-error-no-channel = Impossible de joindre l’invité : l’e-mail n’est pas configuré sur ce serveur et l’invité n’a pas relié Telegram.
msg-error-closed = Les messages ne sont possibles que pour les réservations en attente ou confirmées, jusqu’à une semaine après le rendez-vous.
msg-email-subject = Message de { $host } : { $event }, { $date }
msg-email-intro = { $host } vous a envoyé un message au sujet de { $event } :
msg-email-footer = Pour répondre à { $host }, répondez à cet e-mail.
tg-guest-host-message = Message de { $host } au sujet de { $event }, { $when } :
tg-guest-reply-hint = Pour répondre, écrivez votre réponse ici.
tg-host-guest-message = Message de { $guest } ({ $email }) au sujet de { $event }, { $when } :
tg-host-reply-hint = Répondez à ce message pour répondre à l’invité.
tg-bot-message-sent = Votre message a été transmis à l’organisateur.
tg-bot-message-pick-booking = Plusieurs réservations sont reliées à ce chat. Répondez à un message concernant la réservation voulue.
tg-bot-message-closed = Cette réservation n’accepte plus de messages.
tg-bot-message-after-payment = Vous pourrez écrire à l’organisateur une fois la réservation payée.
tg-bot-message-limit = Trop de messages. Veuillez réessayer plus tard.
tg-bot-reply-sent = Envoyé à l’invité.
tg-bot-reply-not-linked = Cet invité n’a pas relié Telegram, la réponse n’a donc pas été envoyée. Écrivez-lui depuis la réservation dans le tableau de bord.
tg-bot-reply-not-allowed = Réponse non envoyée : la réservation n’est pas la vôtre ou n’accepte plus de messages.
msg-error-internal = Le message n’a pas pu être enregistré, rien n’a donc été envoyé. Réessayez.
msg-link-new = Messages ({ $count } nouveaux)
msg-recent-title = Rendez-vous récents
msg-recent-help = Rendez-vous commencés ou terminés au cours de la dernière semaine. Vous pouvez encore échanger des messages avec leurs invités.
tg-bot-message-sent-shortened = Votre message a été transmis à l’organisateur, raccourci à { $max } caractères.
tg-bot-reply-too-long = Réponse non envoyée : un message peut contenir au plus { $max } caractères.
tg-bot-reply-empty = Réponse non envoyée : elle ne contient aucun texte visible.
tg-bot-reply-failed = La réponse n’a pas pu être enregistrée et n’a donc pas été envoyée. Réessayez.
troubleshoot-not-synced = Les calendriers ne sont lus que du { $from } au { $until } ; les événements en dehors de ces jours ne sont pas chargés et n'apparaissent donc pas ici.
event-type-form-horizon-capped =
    { $days ->
        [one] Ce serveur propose des créneaux au plus { $days } jour à l'avance, quel que soit l'horizon de réservation.
       *[other] Ce serveur propose des créneaux au plus { $days } jours à l'avance, quel que soit l'horizon de réservation.
    }

## ilia fork: the system mailbox (src/inbox)
msg-help-email-no-inbox = L’invité le reçoit par e-mail depuis l’adresse du système. Les réponses par e-mail ne sont pas lues sur ce serveur.
msg-via-email = e-mail
msg-email-footer-no-reply = Ce message a été envoyé par le système de réservation au nom de { $host } ; les réponses ne sont pas lues.
tg-host-guest-email = E-mail de { $guest } ({ $email }) au sujet de { $event }, { $when } :
tg-host-attendee-email = E-mail de { $sender }, participant supplémentaire de la réservation de { $guest } ({ $email }), au sujet de { $event }, { $when } :
tg-host-other-email = E-mail d'une autre adresse, pas celle de l'invité : { $sender }, dans le fil de la réservation de { $guest } ({ $email }) au sujet de { $event }, { $when } :
booking-blocked-body-system = L'hôte exige un préavis d'au moins { $minutes } minutes. Si vous ne pouvez pas être présent, écrivez à <a href="{ $href }">{ $address }</a>.
booking-blocked-body-no-reply = L'hôte exige un préavis d'au moins { $minutes } minutes. Si vous ne pouvez pas être présent, prévenez l'hôte par un autre moyen.

## ilia fork: passkeys and security keys (docs/passkeys-plan.md)

ilia-passkey-sign-in = Se connecter avec une clé d’accès
ilia-mfa-login-help-passkey = Confirmez cette connexion avec votre clé d’accès ou votre clé de sécurité. La connexion expire au bout de 10 minutes.
ilia-mfa-use-totp = Utiliser un code d’application
ilia-mfa-use-recovery = Utiliser un code de récupération
ilia-mfa-use-passkey = Utiliser une clé d’accès
ilia-mfa-use-passkey-instead = Utiliser plutôt une clé d’accès ou une clé de sécurité
ilia-mfa-totp-login-help = Saisissez le code à six chiffres de votre application d’authentification. La connexion expire au bout de 10 minutes.
ilia-mfa-recovery-code = Code de récupération
ilia-mfa-recovery-login-help = Saisissez l’un des codes de récupération que vous avez enregistrés. Chaque code ne fonctionne qu’une fois. La connexion expire au bout de 10 minutes.
ilia-passkey-unsupported = Ce navigateur ne prend pas en charge les clés d’accès ni les clés de sécurité. Choisissez une autre méthode ci-dessous ou ouvrez cette page dans un navigateur à jour.
ilia-passkey-noscript = Les clés d’accès nécessitent JavaScript. Activez-le pour ce site ou choisissez une autre méthode ci-dessous.
ilia-passkey-cancelled = La demande a été annulée ou a expiré. Réessayez ou choisissez une autre méthode.
ilia-passkey-invalid = La clé d’accès ou la clé de sécurité n’a pas pu être vérifiée, ou la demande a expiré. Réessayez ou choisissez une autre méthode.
ilia-passkey-key-type = Ce navigateur ne sait pas lire ce type de clé, elle ne peut donc pas être ajoutée ici. Essayez un autre navigateur, par exemple une version à jour de Firefox ou de Chrome, ou choisissez une autre méthode.
ilia-passkey-wrong-site = Les clés d’accès ne fonctionnent qu’à l’adresse { $origin }. Ouvrez cette page à cette adresse et réessayez, ou choisissez une autre méthode.
ilia-mfa-enroll-help = Choisissez comment vous confirmerez chaque connexion. Une clé d’accès ou une clé de sécurité est le choix le plus sûr : elle ne fonctionne que sur ce site, une fausse copie de la page ne peut donc pas l’utiliser.
ilia-passkey-add-security-key = Ajouter une clé de sécurité
ilia-passkey-add-device = Ajouter une clé d’accès sur cet appareil ou ce téléphone
ilia-passkey-kinds-help = Une clé de sécurité est une clé USB ou NFC, comme une YubiKey. Une clé d’accès sur cet appareil ou ce téléphone utilise Touch ID, Face ID, Windows Hello ou le verrouillage de l’écran du téléphone, et peut être synchronisée avec vos autres appareils.
ilia-mfa-use-app-instead = Utiliser plutôt une application d’authentification
ilia-passkey-name = Nom de la clé
ilia-passkey-name-help = Facultatif, par exemple « YubiKey du trousseau » ou « MacBook ». 64 caractères au maximum.
ilia-passkey-default-security-key = Clé de sécurité { $n }
ilia-passkey-default-device = Clé d’accès { $n }
ilia-passkey-register-help = Appuyez sur un bouton et suivez votre navigateur : insérez la clé de sécurité et touchez-la, ou confirmez avec Touch ID, Face ID, Windows Hello ou votre téléphone.
ilia-passkey-exists = Cette clé est déjà enregistrée.
ilia-mfa-recovery-help = Conservez ces codes de récupération en lieu sûr ; ils ne sont affichés qu’une fois. Chaque code remplace une clé d’accès ou un code d’application pour une connexion. Les nouveaux codes remplacent tous les précédents.
ilia-mfa-status-on = L’authentification à deux facteurs est activée.
ilia-mfa-status-off = L’authentification à deux facteurs est désactivée.
ilia-mfa-setup-intro = Ajoutez une deuxième étape à chaque connexion : une clé d’accès ou une clé de sécurité (YubiKey, Touch ID, Face ID, Windows Hello, votre téléphone), ou une application d’authentification.
ilia-passkey-list = Clés d’accès et clés de sécurité
ilia-passkey-none = Aucune clé d’accès ni clé de sécurité pour l’instant.
ilia-passkey-kind-hardware = Clé matérielle
ilia-passkey-kind-synced = Clé d’accès synchronisée
ilia-passkey-added = Ajoutée le { $date }
ilia-passkey-last-used = Dernière utilisation le { $date }
ilia-passkey-never-used = Pas encore utilisée
ilia-passkey-other-site = Enregistrée pour { $rp_id } ; elle ne fonctionne pas à cette adresse.
ilia-passkey-rename = Renommer
ilia-passkey-delete = Supprimer
ilia-passkey-add = Ajouter une clé d’accès ou une clé de sécurité
ilia-passkey-new-name = Nouveau nom
ilia-passkey-limit = Vous avez atteint le nombre maximal de clés ({ $max }). Supprimez-en une pour en ajouter une autre.
ilia-passkey-backup-advice = Ajoutez une deuxième clé et conservez-la en lieu sûr, pour que la perte d’une clé ne vous bloque pas l’accès.
ilia-passkey-unavailable = Les clés d’accès sont désactivées : l’adresse publique du serveur (CALRS_BASE_URL) n’est pas une adresse https://. Contactez l’administrateur. Les clés déjà ajoutées restent listées et peuvent être supprimées.
ilia-mfa-totp-title = Application d’authentification
ilia-mfa-totp-on = Une application d’authentification est configurée.
ilia-mfa-totp-off = Aucune application d’authentification n’est configurée.
ilia-mfa-totp-add = Configurer une application d’authentification
ilia-mfa-totp-remove = Supprimer l’application d’authentification
ilia-mfa-recovery-title = Codes de récupération
ilia-mfa-recovery-left =
    { $count ->
        [one] { $count } code de récupération restant
       *[other] { $count } codes de récupération restants
    }
ilia-mfa-last-factor = L’authentification à deux facteurs est obligatoire : votre dernière méthode de connexion ne peut donc pas être supprimée. Ajoutez-en d’abord une autre.
ilia-mfa-last-factor-off = C’est votre dernière méthode : la supprimer désactive l’authentification à deux facteurs et efface vos codes de récupération.
ilia-mfa-changes-help = Chaque modification ici demande votre mot de passe et une clé d’accès ou un code, et déconnecte vos autres sessions.
ilia-mfa-confirm-title = Confirmez votre identité
ilia-mfa-confirm-help = Saisissez votre mot de passe, puis confirmez avec une clé d’accès ou un code.
ilia-passkey-confirm = Confirmer avec une clé d’accès
ilia-mfa-confirm-code = Confirmer avec un code
ilia-mfa-action-passkey-add = Vous ajoutez une clé d’accès ou une clé de sécurité.
ilia-mfa-action-passkey-rename = Vous renommez la clé « { $name } ».
ilia-mfa-action-passkey-delete = Vous supprimez la clé « { $name } ».
ilia-mfa-action-totp-add = Vous configurez une application d’authentification.
ilia-mfa-action-totp-remove = Vous supprimez l’application d’authentification.
ilia-mfa-action-regenerate = Vous remplacez tous les codes de récupération.
ilia-mfa-action-policy = Vous réglez la règle d’authentification à deux facteurs des comptes locaux sur :
ilia-mfa-policy-continue = Continuer

## ilia fork: the instance name (CALRS_BRAND_NAME) and the error page (src/web/error_page.rs)
ilia-login-subtitle = Connectez-vous à votre compte { $brand }
ilia-register-subtitle = Créez un nouveau compte { $brand }
ilia-error-not-found = Introuvable
ilia-error-not-found-body = Il n’y a rien à cette adresse. Vérifiez le lien.
ilia-error-gone = Ce lien n’est plus valide
ilia-error-bad-request = Requête invalide
ilia-error-forbidden = Accès restreint

# ilia fork: TON price by demand (the ladder)
ton-slots-ladder-now = Maintenant : { $ton } TON
ton-slots-ladder-now-free = Maintenant : gratuit, l’hôte confirme la demande
ton-slots-ladder-locked = Le prix est fixé au moment de la réservation.
ton-slots-ladder-summary-free = Gratuit tant que le calendrier est vide, puis de { $from } à { $to } TON à mesure qu’il se remplit.
ton-slots-ladder-summary = De { $from } à { $to } TON, selon le remplissage du calendrier.
ton-slots-ladder-all-free = Gratuit ; l’hôte confirme chaque demande.
ton-slots-ladder-fee = Les frais de réseau sont payés par votre portefeuille.
ton-quote-title = Le prix a changé
ton-quote-was = Le prix affiché était de { $ton } TON.
ton-quote-was-free = Une réservation gratuite vous était proposée.
ton-quote-now = Le prix maintenant : { $ton } TON.
ton-quote-now-free = C’est maintenant gratuit : l’hôte confirme la demande.
ton-quote-why = Le calendrier ou les prix ont changé depuis le chargement de la page. Rien n’a encore été réservé.
ton-quote-why-free-used = Vous avez déjà eu une réservation gratuite récemment, cette réservation est donc payante. Rien n’a encore été réservé.
ton-quote-confirm-help = Veuillez confirmer le prix. Rien n’a encore été réservé.
ton-quote-confirm = Réserver pour { $ton } TON
ton-quote-confirm-free = Demander gratuitement
ton-bae-body-free-used = Vous avez déjà eu une réservation gratuite récemment, celle-ci ne peut donc pas être gratuite. Réessayez plus tard.
ton-form-error-mode = Choisissez un prix fixe ou le prix selon la demande.
ton-etf-mode = Tarification
ton-etf-mode-fixed = Prix fixe
ton-etf-mode-ladder = Selon la demande (la grille de prix du site, en TON)
ton-etf-ladder-now = Grille actuelle :
ton-etf-ladder-help = Le prix est le palier correspondant au nombre de vos réservations à venir dans les prochains jours (confirmées ou payées, demandes gratuites en attente de votre approbation et réservations d’équipe que vous animez ; les impayées ne comptent pas) : gratuit au début (vous confirmez chaque réservation gratuite), puis plus élevé à mesure que votre calendrier se remplit. Les invités voient le prix actuel et paient le prix affiché. La grille est fixée par l’administrateur.

# ilia fork: the TON settings page (/dashboard/admin/ton)
nav-admin-ton = Paiements TON
ton-etf-ladder-edit = Modifier la grille
ton-admin-page-title = Paiements TON
ton-admin-heading = Paiements TON
ton-admin-intro = Ces réglages s'appliquent aux paiements TON de ce serveur. Une valeur définie dans l'environnement du serveur l'emporte, puis celle enregistrée ici ou par config apply, puis la valeur par défaut.
ton-admin-saved-live = Enregistré. Les nouveaux réglages s'appliquent dès maintenant ; les factures ouvertes gardent leurs montants et leurs délais.
ton-admin-saved-off = Enregistré. Les paiements sont désactivés sur ce serveur (CALRS_TON_NETWORK n'est pas défini) ; les réglages s'appliqueront quand ils seront activés.
ton-admin-saved-restart = Enregistré. Les réglages prendront effet après le redémarrage de calrs.
ton-admin-saved-invalid = Enregistré, mais l'ensemble des réglages du serveur n'est pas valide : les paiements restent désactivés. Voir l'état ci-dessous.
ton-admin-saved-none = Rien n'a changé.
ton-admin-errors = Rien n'a été enregistré : corrigez les champs signalés ci-dessous.
ton-admin-status = État
ton-admin-status-on = Les paiements sont activés
ton-admin-status-off = Les paiements sont désactivés
ton-admin-network = Réseau
ton-admin-address = Adresse du portefeuille
ton-admin-address-raw = Forme brute
ton-admin-indexer = Indexeur
ton-admin-watcher = Surveillance
ton-admin-watcher-ready = prête
ton-admin-watcher-not-ready = pas prête : { $reason }
ton-admin-last-step = dernière étape
ton-admin-open-invoices = Factures ouvertes
ton-admin-api-key = Clé API de l'indexeur
ton-admin-coingecko-key = Clé CoinGecko
ton-admin-secret-credentials = définie (credentials systemd)
ton-admin-secret-env = définie (environnement)
ton-admin-not-set = non défini
ton-admin-env-only = Le réseau, l'adresse du portefeuille, l'indexeur et les clés API se définissent uniquement dans l'environnement du serveur et les credentials systemd, jamais sur cette page.
ton-admin-src-env = défini par l'environnement ({ $var })
ton-admin-src-apply = défini par config apply
ton-admin-src-panel = défini ici
ton-admin-src-default = par défaut
ton-admin-locked-env = Pour le modifier ici, retirez { $var } de l'environnement du serveur et redémarrez calrs.
ton-admin-locked-apply = Modifiez-le dans le document de config apply, ou retirez-le de sa section ton pour le confier à cette page.
ton-admin-default-hint = Laissez vide pour la valeur par défaut : { $value }.
ton-admin-blank-default = Laissez vide pour aucune valeur.
ton-admin-group-pricing = Prix selon la demande
ton-admin-group-timing = Délais des factures
ton-admin-group-rate = Taux de change (prix en USD)
ton-admin-group-alerts = Alertes
ton-admin-f-ladder = Grille (TON selon le nombre de réservations)
ton-admin-f-free_per_guest_days = Une réservation gratuite par invité sur (jours, 0 = sans limite)
ton-admin-f-window_min = Délai de paiement (minutes)
ton-admin-f-grace_min = Délai de grâce après le délai (minutes)
ton-admin-f-fuse_hours = Marge de sécurité après la grâce (heures)
ton-admin-f-renew_min_lead_hours = Prolonger un délai impayé si la réunion est au moins à cette distance (heures)
ton-admin-f-renew_window_min = Délai prolongé (minutes)
ton-admin-f-max_renewals = Prolongations par facture
ton-admin-f-underpay_tolerance_ton = Sous-paiement accepté (TON)
ton-admin-f-rate_min_usd = Taux le plus bas accepté (USD par TON)
ton-admin-f-rate_max_usd = Taux le plus haut accepté (USD par TON)
ton-admin-f-rate_max_spread_pct = Écart maximal entre les sources (%)
ton-admin-f-rate_max_age_min = Âge maximal du taux (minutes)
ton-admin-f-rate_max_jump_pct = Saut maximal par rapport au dernier taux (%)
ton-admin-f-alert_email = E-mail des alertes (vide : tous les administrateurs)
ton-admin-ladder-help = Paliers séparés par des virgules : le prix à 0, 1, 2… réservations confirmées dans les jours à venir ; le dernier palier est le plafond. « free » seulement au début : une réservation gratuite attend l'accord du propriétaire.
ton-admin-ladder-row = À partir de %1 réservations
ton-admin-ladder-add = Ajouter un palier
ton-admin-ladder-remove = Retirer
ton-admin-free = gratuit
ton-admin-preview = Aperçu
ton-admin-preview-button = Aperçu de la grille
ton-admin-previewing = Aperçu de la grille saisie ci-dessus. Rien n'est encore enregistré.
ton-admin-ladder-invalid = La grille en vigueur n'est pas valide : { $error }
ton-admin-preview-n = Réservations à venir
ton-admin-preview-price = Prix
ton-admin-preview-or-more = { $n } ou plus
ton-admin-my-demand = Vos réservations à venir : { $n }. Votre prix selon la grille : { $price }.
ton-admin-types = Types d'événement payants
ton-admin-types-none = Aucun type d'événement n'a de prix.
ton-admin-type = Type d'événement
ton-admin-type-owner = Propriétaire
ton-admin-type-price = Prix
ton-admin-type-now = Maintenant
ton-admin-mode-ladder = selon la demande
ton-admin-type-off = prix désactivé
ton-admin-type-below-ceiling = Au plafond de la grille ou en dessous : un prix « à tout moment » est normalement au-dessus.
ton-admin-save = Enregistrer
ton-admin-audit = Modifications récentes
ton-admin-audit-none = Aucune modification pour l'instant.
ton-admin-audit-at = Quand
ton-admin-audit-who = Qui
ton-admin-audit-key = Réglage
ton-admin-audit-change = Modification
ton-admin-audit-default = (par défaut)
ton-admin-via-apply = config apply

## ilia fork: the debug booking page (CALRS_DEBUG_BOOKING, src/web/debug_booking.rs)

nav-admin-debug-booking = Réservation de test
dbg-title = Réservation de test
dbg-intro = Réservez un rendez-vous de test comme le ferait un invité, sans remplir le formulaire invité à chaque fois. La réservation passe par le même chemin que la page de réservation publique : les vérifications, le prix et la facture TON, le blocage du créneau et les e-mails. Le paiement n'est jamais sauté ni marqué comme reçu : payez la facture depuis votre portefeuille ou annulez la réservation.
dbg-switch-note = Désactivez cette page une fois le débogage terminé, avec l'interrupteur en haut de la page Paiements TON :
dbg-cancelled = Réservations de test annulées : { $count }.
dbg-no-types = Vous n'avez aucun type de rendez-vous personnel activé.
dbg-event-type = Type de rendez-vous
dbg-priced = payant
dbg-private-short = privé
dbg-choose = Afficher
dbg-price = Prix
dbg-price-now-free = gratuit maintenant (vous approuvez la demande)
dbg-price-now-ton = { $ton } TON maintenant
dbg-price-ladder = selon la demande, fixé à la réservation
dbg-price-free-used = cet invité a déjà eu une réservation gratuite récemment, le premier prix s’applique donc
dbg-free = gratuit
dbg-private = Ce type de rendez-vous n'est pas public : une invitation à usage unique pour votre propre adresse est créée pour chaque réservation de test.
dbg-ton-off = Les paiements TON sont désactivés sur ce serveur : un type de rendez-vous payant ne peut pas être réservé pour l'instant.
dbg-captcha-on = Le captcha est activé pour les formulaires de réservation. Cette page ne peut pas le résoudre : tant qu'il est activé, le chemin de réservation refuse les réservations de test.
dbg-err-username = Votre compte n'a pas de nom d'utilisateur, il n'a donc pas de pages de réservation.
dbg-slot = Horaire
dbg-tz = Horaires en { $tz }, le fuseau horaire de votre profil.
dbg-no-slots = Aucun créneau libre dans les 31 prochains jours.
dbg-name = Nom
dbg-email = E-mail
dbg-email-help = Uniquement vos propres adresses : votre e-mail de connexion et votre e-mail de réservation.
dbg-telegram-username = Nom d'utilisateur Telegram
dbg-telegram-required = obligatoire pour ce type de rendez-vous
dbg-notes = Notes
dbg-notes-default = Réservation de test depuis la page de débogage.
dbg-own-chat = Telegram : envoyer les messages destinés à l'invité dans ma propre conversation Telegram liée
dbg-own-chat-off = Telegram est désactivé sur ce serveur (pas de jeton de bot).
dbg-own-chat-linked = Votre conversation liée : { $who }.
dbg-own-chat-help = Le côté invité de la réservation (ce qu'il faut payer, la confirmation, le rappel, une annulation) arrive dans la conversation où le bot vous écrit déjà, comme si vous y aviez ouvert le lien de l'invité. Case décochée, la page de paiement affiche le lien de l'invité comme d'habitude.
dbg-own-chat-not-linked = Votre conversation Telegram n'est pas liée. Liez-la d'abord, puis revenez ici :
dbg-own-chat-no-username = sans nom d'utilisateur
dbg-test-note = C'est une vraie réservation, marquée comme test. Tant qu'elle existe, elle bloque le créneau, compte dans le prix selon la demande et envoie les e-mails et messages habituels à votre propre adresse. Annulez-la quand vous avez terminé.
dbg-submit = Réserver comme invité
dbg-list = Mes réservations de test
dbg-list-none = Aucune réservation de test pour l'instant.
dbg-col-when = Quand
dbg-col-type = Type de rendez-vous
dbg-col-status = Statut
dbg-col-payment = Paiement
dbg-col-telegram = Telegram
dbg-open-pay = page de paiement
dbg-own-chat-yes = ma conversation
dbg-cancel-all = Annuler toutes mes réservations de test ({ $count })
dbg-cancel-all-help = Annule chacune de vos réservations de test encore valables (une demande en attente est refusée) comme le fait le tableau des réservations : les factures ouvertes sont fermées et les notifications habituelles envoyées. Les autres réservations ne sont pas touchées.
dbg-cancel-reason = Réservation de test annulée depuis la page de débogage.
dbg-impersonating = Arrêtez l'usurpation d'identité pour utiliser la page de réservation de test.
dbg-err-type = Choisissez l'un de vos propres types de rendez-vous.
dbg-err-email = Utilisez l'une de vos propres adresses.
dbg-err-slot = Choisissez un horaire dans la liste.
dbg-err-own-chat = Liez d'abord votre conversation Telegram, ou décochez la case Telegram.
dbg-own-chat-failed = La réservation de test a été créée, mais les messages destinés à l’invité n’ont pas pu être dirigés vers votre conversation Telegram : la conversation a été dissociée, le bot arrêté ou le lien a expiré entre-temps. La réservation reste valable et bloque le créneau : ouvrez sa page de paiement dans la liste ci-dessous, ou annulez-la.
dbg-st-pending = en attente
dbg-st-confirmed = confirmée
dbg-st-cancelled = annulée
dbg-st-declined = refusée
dbg-inv-open = en attente de paiement
dbg-inv-paid = payée
dbg-inv-confirmed = payée et confirmée
dbg-inv-expired = expirée
dbg-inv-closed = fermée
dbg-inv-waived = confirmée sans paiement
dbg-banner = DÉBOGAGE · réservation de test
dbg-badge-test = TEST
ton-admin-debug-title = Débogage
ton-admin-debug-intro = Réservation de test : réservez vos propres types de rendez-vous comme un invité (le vrai prix, la facture, la page de paiement et Telegram), sans remplir le formulaire invité. Réservé aux administrateurs. Tant qu'elle est désactivée, sa page répond 404 et rien n'y renvoie.
ton-admin-debug-state-on = Réservation de test : activée
ton-admin-debug-state-off = Réservation de test : désactivée
ton-admin-debug-turn-on = Activer
ton-admin-debug-turn-off = Désactiver
ton-admin-debug-open = Ouvrir la réservation de test
ton-admin-debug-locked-env = Fixé par l'environnement du serveur ({ $var }={ $value }). Pour le changer ici, retirez { $var } de l'environnement et redémarrez calrs.
ton-admin-debug-locked-apply = Fixé par config apply (site.ton.debug_booking). Modifiez-le dans le document de config apply, ou retirez la clé pour le confier à cette page.
ton-admin-debug-saved-on = La réservation de test est activée.
ton-admin-debug-saved-off = La réservation de test est désactivée.
ton-admin-debug-saved-locked = Rien n'a changé : l'interrupteur est verrouillé.
ton-admin-f-debug_booking = Réservation de test
