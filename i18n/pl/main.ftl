# Booking confirmation page (templates/confirmed.html)

confirmed-page-title-pending = Rezerwacja oczekująca
confirmed-page-title-booked = Rezerwacja potwierdzona

confirmed-heading-reschedule-requested = Prośba o zmianę terminu
confirmed-heading-rescheduled = Termin zmieniony!
confirmed-heading-pending = Oczekuje na potwierdzenie
confirmed-heading-booked = Zarezerwowano!

confirmed-subtitle-reschedule-requested = Twoja prośba o zmianę terminu została wysłana do { $host }. Otrzymasz e-mail na adres { $email }, gdy zostanie zatwierdzona.
confirmed-subtitle-rescheduled = Twoja rezerwacja została przeniesiona. E-mail potwierdzający został wysłany na adres { $email }.
confirmed-subtitle-pending = Twoja prośba o rezerwację została wysłana do { $host }. Otrzymasz e-mail na adres { $email }, gdy zostanie potwierdzona.
confirmed-subtitle-booked = E-mail potwierdzający został wysłany na adres { $email }.

confirmed-detail-event = Wydarzenie:
confirmed-detail-date = Data:
confirmed-detail-time = Godzina:
confirmed-detail-with = Z:
confirmed-detail-location = Miejsce:
confirmed-detail-notes = Notatki:
confirmed-detail-additional-guests = Dodatkowi goście:

confirmed-book-another = Zarezerwuj inny termin

confirmed-add-to-calendar = Dodaj do kalendarza

# Slot picker (templates/slots.html)

## ilia-ov-slots
slots-time-format-24 = 24 h
slots-time-format-12 = 12 h
slots-prev-month = Poprzedni miesiąc
slots-next-month = Następny miesiąc
slots-prev-week = Poprzedni tydzień
slots-next-week = Następny tydzień
book-remove-guest = Usuń gościa
ilia-error-back-to-site = Wróć do strony

slots-location-video = Wideokonferencja
slots-location-phone = Rozmowa telefoniczna
slots-location-google-meet = Google Meet

slots-tz-label = Twoja strefa czasowa
slots-time-format-label = Format czasu

## ilia-ov5-guest
tz-city-midway = Midway
tz-city-hawaii = Hawaje
tz-city-alaska = Alaska
tz-city-los-angeles = Los Angeles
tz-city-denver = Denver
tz-city-chicago = Chicago
tz-city-new-york = Nowy Jork
tz-city-sao-paulo = São Paulo
tz-city-cape-verde = Republika Zielonego Przylądka
tz-city-london = Londyn
tz-city-paris = Paryż, Bruksela
tz-city-helsinki = Helsinki, Kijów
tz-city-moscow = Moskwa
tz-city-dubai = Dubaj
tz-city-india = Indie
tz-city-bangkok = Bangkok
tz-city-shanghai = Szanghaj
tz-city-tokyo = Tokio
tz-city-sydney = Sydney
tz-city-auckland = Auckland

tz-group-common = Popularne strefy czasowe
tz-region-africa = Afryka
tz-region-america = Ameryka
tz-region-antarctica = Antarktyda
tz-region-arctic = Arktyka
tz-region-asia = Azja
tz-region-atlantic = Ocean Atlantycki
tz-region-australia = Australia
tz-region-europe = Europa
tz-region-indian = Ocean Indyjski
tz-region-pacific = Ocean Spokojny
slots-offset-note-day = Godziny w dniu %1 są w %2 (zmiana czasu %3).
slots-offset-note-range = Zmiana czasu %1: od tego dnia godziny są w %2.

slots-view-month = Widok miesiąca
slots-view-week = Widok tygodnia
slots-view-column = Widok listy

slots-weekday-mon = Pon
slots-weekday-tue = Wt
slots-weekday-wed = Śr
slots-weekday-thu = Czw
slots-weekday-fri = Pt
slots-weekday-sat = Sob
slots-weekday-sun = Nd

slots-weekday-mon-short = P
slots-weekday-tue-short = W
slots-weekday-wed-short = Ś
slots-weekday-thu-short = C
slots-weekday-fri-short = P
slots-weekday-sat-short = S
slots-weekday-sun-short = N

slots-select-date = Wybierz datę

## ilia-ov3-guest
# The deferred slot load failed (slots.html); the booking form's TON price note (partials/ton_price.html).
slots-load-failed = Nie udało się wczytać dostępności. Odśwież stronę.
ton-book-price-note = płatność w TON po rezerwacji

slots-loading-availability = Ładowanie dostępności...
slots-click-highlighted = Kliknij wyróżnioną datę, aby zobaczyć dostępne godziny
slots-bookable-range-both = Rezerwacja możliwa od { $from } do { $until }
slots-bookable-range-from = Rezerwacja możliwa od { $from }
slots-bookable-range-until = Rezerwacja możliwa do { $until }
slots-no-longer-bookable = Tego wydarzenia nie można już zarezerwować.
slots-no-times-month = Brak dostępnych godzin w tym miesiącu
slots-no-times-day = Brak dostępnych godzin w tym dniu
slots-no-availability-participants = Brak wspólnej dostępności wszystkich uczestników w tym miesiącu
slots-week-more = więcej

# Booking form (templates/book.html)

book-page-title = Zarezerwuj { $title }
book-back-to-times = Powrót do godzin
book-name-label = Twoje imię
book-name-placeholder = Jan Kowalski
book-email-label = E-mail
book-email-placeholder = jan@example.com
book-email-invalid = Proszę podać pełny adres e-mail wraz z domeną (np. jane@example.com).
book-notes-label = Notatki
book-notes-optional = (opcjonalne)
book-notes-placeholder = Czy jest coś, co chcesz omówić?
book-additional-guests-label = Dodatkowi goście
book-additional-guests-hint = (opcjonalne, do { $max })
book-add-guest-btn = + Dodaj gościa
book-guest-email-placeholder = wspolpracownik@example.com
book-phone-label = Numer telefonu
book-phone-placeholder = 601 234 567
book-phone-help = Numery lokalne są w porządku; jeśli nie zaczniesz od +, przyjmujemy { $country }.
book-phone-optional-consequence = Zostaw puste, jeśli wolisz nie dostawać SMS-ów o tej rezerwacji.
book-phone-required = Ta rezerwacja wymaga numeru telefonu.
book-phone-invalid-title = Nieprawidłowy numer telefonu
book-phone-invalid = Proszę podać numer, na który możemy wysłać SMS, albo zostawić pole puste.
book-phone-country-search = Szukaj
book-phone-country-label = Wybierz kraj
book-phone-country-none = Nie wybrano kraju
book-phone-country-no-results = Żaden kraj nie pasuje do wyszukiwania
captcha-label = Weryfikacja bezpieczeństwa
captcha-initial-state = Potwierdź, że jesteś człowiekiem
captcha-verifying = Weryfikacja...
captcha-solved = Jesteś człowiekiem
captcha-error = Błąd
captcha-troubleshooting = Rozwiązywanie problemów
captcha-wasm-disabled = Włącz WASM dla znacznie szybszego rozwiązywania
captcha-verify-aria = Kliknij, aby potwierdzić, że jesteś człowiekiem
captcha-verifying-aria = Weryfikacja, proszę czekać
captcha-verified-aria = Zweryfikowano
captcha-required = Proszę potwierdzić, że jesteś człowiekiem
captcha-error-aria = Wystąpił błąd, spróbuj ponownie
book-confirm-button = Potwierdź rezerwację

# SMS notifications (src/sms/message.rs).
#
# These are text messages, billed per 160-character segment (70 if the text
# contains any character outside the GSM-7 alphabet, which includes most
# accented letters). Keep them short and plain.

sms-confirmed = Rezerwacja potwierdzona: { $event }, { $date } o { $time } ({ $tz }).
sms-cancelled = Rezerwacja odwołana: { $event }, { $date } o { $time } ({ $tz }).
sms-rescheduled = Rezerwacja przeniesiona: { $event } odbędzie się { $date } o { $time } ({ $tz }).
sms-reminder = Przypomnienie: { $event } zaczyna się { $date } o { $time } ({ $tz }).

# Shared labels used across the cancel / decline / approve / reschedule / claim flows

common-detail-guest = Gość:
common-detail-reason = Powód:
common-reason-optional = (opcjonalne)
common-close-page = Możesz zamknąć tę stronę.

# Cancel flow (booking_cancel_form.html, booking_cancelled_guest.html)

cancel-page-title = Anuluj rezerwację
cancel-heading = Anuluj rezerwację
cancel-subtitle = Zamierzasz anulować swoją rezerwację.
cancel-reason-label = Powód
cancel-reason-placeholder-host = Wyjaśnij organizatorowi powód...
cancel-button = Anuluj rezerwację
cancelled-heading = Rezerwacja anulowana
cancelled-subtitle = Twoja rezerwacja została anulowana, a organizator został powiadomiony.

# Decline flow (booking_decline_form.html, booking_declined.html)

decline-page-title = Odrzuć rezerwację
decline-heading = Odrzuć rezerwację
decline-subtitle = Zamierzasz odrzucić tę prośbę o rezerwację.
decline-reason-placeholder-guest = Wyjaśnij gościowi powód...
decline-button = Odrzuć rezerwację
declined-heading = Rezerwacja odrzucona
declined-subtitle = Rezerwacja została odrzucona, a gość został powiadomiony.

# Approve flow (booking_approve_form.html, booking_approved.html)

approve-page-title = Zatwierdź rezerwację
approve-heading = Zatwierdź rezerwację
approve-subtitle = Zamierzasz zatwierdzić tę prośbę o rezerwację.
approve-button = Zatwierdź rezerwację
approved-heading = Rezerwacja zatwierdzona
approved-subtitle = Rezerwacja została potwierdzona, a e-mail z potwierdzeniem został wysłany na adres { $email }.

# Claim flow (booking_claim_form.html, booking_claimed.html, booking_already_claimed.html)

claim-page-title = Przejmij rezerwację
claim-heading = Przejmij rezerwację
claim-subtitle = Zamierzasz przejąć tę rezerwację. Zostaniesz dodany jako uczestnik.
claim-assigned-to = Przypisana do:
claim-button = Przejmij tę rezerwację
claimed-page-title = Rezerwacja przejęta
claimed-heading = Rezerwacja przejęta
claimed-subtitle = Przejąłeś tę rezerwację. Zaproszenie kalendarza zostało wysłane na Twój adres e-mail.
already-claimed-page-title = Już przejęta
already-claimed-heading = Już przejęta
already-claimed-subtitle = Ta rezerwacja została już przejęta przez { $name }.

# Generic error page (booking_action_error.html)

action-error-page-title = Błąd akcji rezerwacji

# Host-initiated reschedule (booking_host_reschedule.html)

host-resched-page-title = Zmień termin rezerwacji — calrs
host-resched-heading = Zmień termin rezerwacji
host-resched-subtitle = Spowoduje to wysłanie do { $guest } e-maila z prośbą o wybranie nowego terminu.
host-resched-currently = Obecnie:
host-resched-button = Wyślij prośbę o zmianę terminu
host-resched-cancel-link = Anuluj

# Guest reschedule confirmation (booking_reschedule_confirm.html)

resched-confirm-page-title = Potwierdź zmianę terminu
resched-confirm-heading = Potwierdź zmianę terminu
resched-confirm-subtitle = Zamierzasz przenieść swoją rezerwację na nowy termin.
resched-was = Było:
resched-new = Nowy:
resched-button = Potwierdź zmianę terminu
resched-back-to-picker = Powrót do wyboru terminu

# Base layout chrome (templates/base.html)

base-loader-checking = Sprawdzanie dostępności
base-loader-please-wait = Proszę czekać, ładowanie najnowszych danych kalendarza...
base-stop-impersonating = Zakończ podszywanie się
base-theme-toggle = Przełącz motyw
base-language-choose = Wybierz język
base-powered-by = Zasilane przez

# Profile (templates/profile.html)

profile-pick-event-type-invite = Wybierz typ wydarzenia, aby zarezerwować termin.
profile-no-event-type = Nie ma jeszcze dostępnych typów wydarzeń.

# Month and weekday names + per-locale date format patterns.
# Used by server-side date formatters in src/i18n.rs.

common-month-1 = styczeń
common-month-2 = luty
common-month-3 = marzec
common-month-4 = kwiecień
common-month-5 = maj
common-month-6 = czerwiec
common-month-7 = lipiec
common-month-8 = sierpień
common-month-9 = wrzesień
common-month-10 = październik
common-month-11 = listopad
common-month-12 = grudzień

common-weekday-long-mon = poniedziałek
common-weekday-long-tue = wtorek
common-weekday-long-wed = środa
common-weekday-long-thu = czwartek
common-weekday-long-fri = piątek
common-weekday-long-sat = sobota
common-weekday-long-sun = niedziela

# Format patterns are parametric per locale to handle word order. Translators
# pick where each placeholder lands. Example outputs:
#   EN: April 2026  /  Tuesday, March 12, 2026
#   FR: avril 2026  /  mardi 12 mars 2026
#   ES: abril 2026  /  martes, 12 de marzo de 2026
common-format-month-year = { $month } { $year }
common-format-long-date = { $weekday }, { $day } { $monthnum ->
    [1] stycznia
    [2] lutego
    [3] marca
    [4] kwietnia
    [5] maja
    [6] czerwca
    [7] lipca
    [8] sierpnia
    [9] września
    [10] października
    [11] listopada
   *[12] grudnia
} { $year }
common-format-date =
    { $day } { $monthnum ->
    [1] stycznia
    [2] lutego
    [3] marca
    [4] kwietnia
    [5] maja
    [6] czerwca
    [7] lipca
    [8] sierpnia
    [9] września
    [10] października
    [11] listopada
    *[12] grudnia
    } { $year }

# Email signatures and shared bits (src/email.rs)

email-signature = — calrs
email-action-reschedule = Zmień termin
email-action-cancel-booking = Anuluj rezerwację

# Email: guest booking confirmation

# Kept to "event — date": Exchange titles the guest appointment after the
# email Subject header, not the ICS SUMMARY (#157).
email-confirm-subject = { $event } — { $date }
email-confirm-greeting = Cześć { $name },
email-confirm-headline = Twoja rezerwacja została potwierdzona!
email-confirm-ics-attached-plain = Zaproszenie do kalendarza w załączniku.
email-confirm-ics-attached-html = Zaproszenie do kalendarza w załączniku tego e-maila.
email-confirm-need-to-cancel = Chcesz anulować? { $url }

# Email: guest reminder

email-reminder-subject = Przypomnienie: { $event } o { $time }
email-reminder-headline = Twoje spotkanie wkrótce się rozpocznie.

# Email: guest cancellation

email-cancel-subject = Anulowano: { $event } — { $date }
email-cancel-headline-by-host = Twoja rezerwacja została anulowana przez { $host }.
email-cancel-headline-by-guest = Twoja rezerwacja została anulowana.
email-cancel-ics-attached-plain = Anulowanie kalendarza w załączniku.
email-cancel-ics-attached-html = Anulowanie kalendarza w załączniku tego e-maila.

# Confirmation email: notice-window policy lines (src/email.rs)

email-confirm-cancel-notice = Uwaga: odwołanie wymaga co najmniej { $minutes } minut wyprzedzenia.
email-confirm-reschedule-notice = Uwaga: przeniesienie wymaga co najmniej { $minutes } minut wyprzedzenia.

# Event type form: cancel/reschedule minimum notice (templates/event_type_form.html)

## ilia-ov-eventform
# The collapsed "Payment in TON" section's summary: no payment, or the price comes from the demand ladder.
ton-etf-hint-off = Bezpłatnie
ton-etf-hint-ladder = Według popytu

# Google Meet (templates/event_type_form.html, src/google_meet.rs)
event-type-form-location-google-meet = Google Meet (link generowany automatycznie)
event-type-form-location-google-meet-hint = Przy potwierdzeniu tworzony jest osobny link Google Meet, należący do przypisanego organizatora. Każdy organizator (ty lub każdy uprawniony członek zespołu) musi mieć połączony Kalendarz Google i wybrany kalendarz do zapisu.
google-meet-prereq-no-host = Google Meet wymaga organizatora z połączonym Kalendarzem Google.
google-meet-prereq-no-eligible = Google Meet wymaga co najmniej jednego uprawnionego członka zespołu z połączonym Kalendarzem Google.
google-meet-prereq-missing = Google Meet wymaga, aby każdy organizator miał połączony Kalendarz Google i wybrany kalendarz do zapisu. Wciąż brakuje: { $names }. Połącz je w Panel → Źródła kalendarzy.
google-meet-unavailable-title = Google Meet jest niedostępny
google-meet-dynamic-group-unavailable = Organizator musi mieć połączony Kalendarz Google i wybrany kalendarz do zapisu.

event-type-form-cancel-notice-label = Minimalne wyprzedzenie przy odwołaniu
event-type-form-reschedule-notice-label = Minimalne wyprzedzenie przy przenoszeniu
event-type-form-notice-help = Zostaw puste, aby nie wprowadzać ograniczenia.
event-type-form-resources-label = Wymagane zasoby
event-type-form-resources-hint = Terminy są proponowane tylko wtedy, gdy wybrane zasoby są dostępne, zgodnie z trybem poniżej.
event-type-form-resources-mode-all = Wszystkie wybrane zasoby muszą być wolne
event-type-form-resources-mode-round-robin = Wystarczy jeden wolny zasób (zostanie przypisany do rezerwacji)
event-type-form-notice-unit-minutes = minut
event-type-form-notice-unit-hours = godzin
event-type-form-notice-unit-days = dni
event-type-form-booking-horizon-label = Horyzont rezerwacji
event-type-form-booking-horizon-help = Z jakim wyprzedzeniem goście mogą rezerwować. Puste oznacza brak limitu, 0 tylko dzisiaj.
event-type-form-bookable-range-label = Okres rezerwacji
event-type-form-bookable-from = od
event-type-form-bookable-until = do
event-type-form-bookable-range-help = Rezerwować można tylko w te dni. Puste oznacza brak limitu. Inne godziny w pojedyncze dni ustawisz w wyjątkach.
event-type-form-bookable-range-past-warning = Ten okres już się zakończył. Typ wydarzenia nie jest już widoczny publicznie.
event-type-form-bookable-range-too-far-warning = Ten okres zaczyna się za ponad rok. Goście mogą rezerwować najwyżej rok wcześniej, więc rezerwacje otworzą się { $date }.
event-type-form-bookable-range-horizon-warning = Horyzont rezerwacji kończy się przed początkiem tego okresu. Goście mogą rezerwować od { $date }.
event-type-form-bookable-range-bookings-outside =
    { $count ->
        [one] 1 istniejąca rezerwacja wypada poza okresem rezerwacji.
        [few] { $count } istniejące rezerwacje wypadają poza okresem rezerwacji.
        [many] { $count } istniejących rezerwacji wypada poza okresem rezerwacji.
       *[other] { $count } istniejących rezerwacji wypada poza okresem rezerwacji.
    }
event-type-form-bookable-range-bookings-link = Zobacz rezerwacje
event-type-form-saved = Zapisano.

# Booking confirmation: cancel/reschedule policy notices (templates/confirmed.html)

confirmed-cancel-notice-info = Odwołanie wymaga co najmniej { $minutes } minut wyprzedzenia przed spotkaniem.
confirmed-reschedule-notice-info = Przeniesienie wymaga co najmniej { $minutes } minut wyprzedzenia przed spotkaniem.

# Booking action blocked page (templates/booking_action_blocked.html)

booking-blocked-title-cancel = Tej rezerwacji nie można już odwołać przez internet
booking-blocked-title-reschedule = Tej rezerwacji nie można już przenieść przez internet
booking-blocked-body = Gospodarz wymaga co najmniej { $minutes } minut wyprzedzenia. Jeśli nie możesz się pojawić, napisz bezpośrednio na <a href="mailto:{ $host_email }">{ $host_email }</a>.

# Dashboard event types listing (templates/dashboard_event_types.html)

## ilia-ov-dates
# Booking times on the dashboard lists: the day ("Today", "Tomorrow", a weekday
# or a date), then the start and end times and the host timezone.
dashboard-when-today = Dziś
dashboard-when-tomorrow = Jutro
dashboard-when-range = { $date }, { $start } — { $end } { $tz }

dashboard-event-types-copy = Kopiuj
dashboard-event-types-copied = Skopiowano!
dashboard-event-types-copy-title = Kopiuj link do rezerwacji
dashboard-event-types-copy-failed = Nie udało się skopiować

# Dashboard sidebar and shared chrome (templates/dashboard_base.html)

## ilia-ov-shell
nav-menu = Menu
event-types-more-actions = Więcej działań

nav-section-scheduling = Planowanie
nav-overview = Przegląd
nav-event-types = Typy wydarzeń
nav-bookings = Rezerwacje
nav-teams = Zespoły
nav-section-shared-links = Udostępnione linki
nav-invite-links = Linki z zaproszeniem
nav-section-calendars = Kalendarze
nav-sources = Źródła
nav-section-personal = Osobiste
nav-settings = Profil i ustawienia
nav-troubleshoot = Diagnostyka
nav-section-admin = Administracja
nav-admin-panel = Panel administracyjny
nav-sign-out = Wyloguj się
nav-release-notes = Zobacz informacje o wersji

# Timezone mismatch banner (templates/dashboard_base.html)

tz-banner-text = Strefa czasowa twojej przeglądarki to { $detected }, ale twoja strefa rezerwacji jest ustawiona na { $current }.
tz-banner-update = Zaktualizuj
tz-banner-dismiss = Zamknij

# Markdown editor toolbar (templates/dashboard_base.html)

editor-link-prompt = Podaj adres URL:
editor-link-default-label = tekst linku
editor-placeholder-text = tekst
editor-nothing-to-preview = Nie ma czego podejrzeć

# Dashboard overview (templates/dashboard_overview.html)

overview-page-title = Panel
overview-welcome = Witaj, { $name }
overview-public-page = Strona publiczna:
overview-avail-banner-title = Domyślna dostępność
overview-avail-banner-body = Twoje domyślne godziny pracy ustawiono na poniedziałek–piątek, 9:00–17:00. Są używane, gdy inni dodają cię do dynamicznych spotkań grupowych.
overview-avail-banner-cta = Sprawdź swoją dostępność
overview-dismiss = Zamknij
overview-getting-started = Pierwsze kroki
overview-getting-started-help = Wykonaj te kroki, aby zacząć przyjmować rezerwacje.
overview-step-connect-calendar = Podłącz kalendarz
overview-step-first-event-type = Utwórz swój pierwszy typ wydarzenia
overview-step-share-link = Udostępnij swój link do rezerwacji
overview-pending-approval = Oczekuje na zatwierdzenie
overview-booking-with = { $title } z { $guest }
overview-badge-pending = oczekuje
overview-guest-booked = Zarezerwowane przez gościa:
overview-confirm = Potwierdź
overview-decline = Odrzuć
overview-stat-event-types = Typy wydarzeń
overview-stat-upcoming = Nadchodzące rezerwacje
overview-stat-pending = Oczekujące na zatwierdzenie
overview-stat-sources = Źródła kalendarzy
overview-quick-actions = Utwórz typ wydarzenia
overview-action-public-title = Publiczna strona rezerwacji
overview-action-public-desc = Udostępnij link — każdy może wybrać termin i zarezerwować czas u ciebie.
overview-action-team-title = Planowanie zespołowe
overview-action-team-desc = Rozdzielaj rezerwacje między członków zespołu albo znajdź termin, w którym wszyscy są wolni.
overview-action-team-desc-empty = Najpierw utwórz zespół, a potem skonfiguruj wspólne typy wydarzeń.
overview-action-private-title = Prywatne, tylko z zaproszeniem
overview-action-private-desc = Generuj jednorazowe linki dla wybranych osób. Nikt inny nie zarezerwuje.
overview-action-shared-title = Udostępnione linki z zaproszeniem
overview-action-shared-desc = Każdy współpracownik w zespole może wygenerować link do rezerwacji i udostępnić go na zewnątrz.
overview-action-reason-calendar = Najpierw podłącz kalendarz
overview-action-reason-ask-admin = Poproś administratora o utworzenie zespołu
overview-action-reason-team-admin = Wymaga zespołu — najpierw utwórz jakiś
overview-action-reason-team-member = Wymaga zespołu — poproś administratora

# Dashboard bookings (templates/dashboard_bookings.html)

## ilia-ov-lists
team-form-slug-placeholder = inzynieria
troubleshoot-label-past = Już minęło
troubleshoot-legend-beyond-horizon = Poza horyzontem lub okresem

bookings-page-title = Rezerwacje
bookings-pending-approval = Oczekuje na zatwierdzenie
bookings-available-to-claim = Do przejęcia
bookings-upcoming = Nadchodzące rezerwacje
bookings-with = { $title } z { $guest }
bookings-guest-booked = Zarezerwowane przez gościa:
bookings-resource = Zasób:
bookings-confirm = Potwierdź
bookings-reschedule = Przenieś
bookings-decline = Odrzuć
bookings-claim = Przejmij
bookings-badge-awaiting-reschedule = oczekuje na przeniesienie
bookings-badge-writeback-failed = brak w twoim kalendarzu
bookings-writeback-failed-detail = Zapis do kalendarza nie powiódł się: { $error }
bookings-cancel = Odwołaj
bookings-reason-placeholder = Powód (opcjonalnie)
bookings-confirm-cancel = Potwierdź odwołanie
bookings-back = Wstecz
bookings-empty = Nie ma jeszcze nadchodzących rezerwacji.<br>Udostępnij swoje { $link }, aby inni mogli rezerwować czas u ciebie.
bookings-empty-link-label = linki do typów wydarzeń

# Dashboard teams listing (templates/dashboard_teams.html)

teams-page-title = Zespoły
teams-heading = Zespoły
teams-new = Nowy
teams-badge-public = publiczny
teams-badge-private = prywatny
teams-settings = Ustawienia
teams-view = Zobacz
teams-empty = Nie ma jeszcze zespołów.
teams-empty-admin = { $link }, aby współpracować ze swoim zespołem.
teams-empty-admin-link-label = Utwórz jakiś
teams-empty-member = Zespoły tworzą administratorzy. Poproś ich o utworzenie zespołu i dodanie cię jako członka.

# Dashboard invite links (templates/dashboard_internal.html)

invite-links-page-title = Linki z zaproszeniem
invite-links-heading = Linki z zaproszeniem
invite-links-new = Nowe wydarzenie wewnętrzne
invite-links-help = Generuj jednorazowe linki do rezerwacji dla wewnętrznych typów wydarzeń. Każdy zalogowany współpracownik może tu tworzyć i udostępniać linki.
invite-links-duration = { $minutes } min
invite-links-hosted-by = Prowadzi { $host }
invite-links-get-link = Pobierz link
invite-links-invites = Zaproszenia
invite-links-empty = Nie ma jeszcze wewnętrznych typów wydarzeń.<br>{ $link } z widocznością „Wewnętrzny”, aby każdy współpracownik mógł generować linki do rezerwacji.
invite-links-empty-link-label = Utwórz typ wydarzenia
invite-links-js-generating = Generowanie...
invite-links-js-copied = Skopiowano!
invite-links-js-error = Błąd

teams-member-count =
    { $count ->
        [one] { $count } członek
        [few] { $count } członkowie
        [many] { $count } członków
       *[other] { $count } członków
    }

# Dashboard calendar sources (templates/dashboard_sources.html)

sources-page-title = Źródła kalendarzy
sources-heading = Źródła kalendarzy
sources-add = Dodaj
sources-last-sync = Ostatnia synchronizacja:
sources-sync-failing = Synchronizacja nie działa od { $since } UTC: { $error }
sources-sync = Synchronizuj
sources-full-resync = Pełna resynchronizacja
sources-full-resync-title = Wyczyść pamięć podręczną i pobierz ponownie wszystkie wydarzenia z serwera
sources-test = Testuj
sources-reconnect = Połącz ponownie
sources-reconnect-title = Powtórz proces zgody Google
sources-edit = Edytuj
sources-remove = Usuń
sources-remove-confirm = Usunąć źródło „{ $name }”? Skasuje to wszystkie wydarzenia zsynchronizowane z tego źródła.
sources-no-write-calendar = Nie wybrano kalendarza do zapisu. Potwierdzone rezerwacje zostają w calrs i nie trafiają do tego kalendarza. Wybierz jeden poniżej, aby włączyć zapis.
sources-write-bookings-to = Zapisuj rezerwacje w:
sources-write-none = Żaden (nie zapisuj)
sources-empty = Nie podłączono żadnych źródeł kalendarza. { $link }, aby sprawdzać dostępność.
sources-empty-link-label = Dodaj jedno
sources-calendars = Kalendarze
sources-calendars-synced = Synchronizowane kalendarze: { $synced } z { $total }
source-calendars-page-title = Kalendarze do synchronizacji
source-calendars-heading = Kalendarze do synchronizacji
source-calendars-back = Wróć do źródeł kalendarzy
source-calendars-help = Wybierz, które kalendarze ze źródła { $name } calrs synchronizuje. Niezaznaczone kalendarze nigdy nie są pobierane i nie blokują Twojej dostępności.
source-calendars-select-all = Zaznacz wszystkie
source-calendars-select-none = Odznacz wszystkie
source-calendars-write-badge = Tu zapisywane są rezerwacje
source-calendars-write-hint = Potwierdzone rezerwacje są zapisywane w tym kalendarzu niezależnie od tego, czy jest synchronizowany.
source-calendars-save = Zapisz
source-calendars-saved = Zapisano. Nowo zaznaczone kalendarze zostaną pobrane przy następnej synchronizacji; użyj przycisku Synchronizuj na stronie źródeł, aby pobrać je teraz.
source-calendars-empty = Nie znaleziono jeszcze żadnych kalendarzy w tym źródle. Uruchom synchronizację, aby je wykryć.

# Dashboard event types listing (templates/dashboard_event_types.html)

event-types-page-title = Typy wydarzeń
event-types-heading = Typy wydarzeń
event-types-new = Nowy
event-types-badge-disabled = wyłączony
event-types-badge-internal = wewnętrzny
event-types-badge-private = prywatny
event-types-badge-resources = zasoby
event-types-badge-active = aktywny
event-types-badge-expired = wygasły
event-types-badge-range-from = od { $date }
event-types-badge-range-until = do { $date }
event-types-send-invites = Wyślij zaproszenia
event-types-duration = { $minutes } min
event-types-mode-collective = zbiorowy
event-types-mode-round-robin = rotacyjny
event-types-edit = Edytuj
event-types-disable = Wyłącz
event-types-enable = Włącz
event-types-embed = Osadź
event-types-overrides = Wyjątki
event-types-team-settings = Ustawienia zespołu
event-types-invites = Zaproszenia
event-types-view-public = Zobacz stronę publiczną
event-types-view-page = Zobacz stronę
event-types-delete = Usuń
event-types-delete-confirm = Usunąć typ wydarzenia „{ $title }”? Tej operacji nie można cofnąć.
event-types-duplicate = Duplikuj
event-types-duplicate-title = { $title } (kopia)
event-types-empty = Nie ma jeszcze typów wydarzeń. { $link }, aby zacząć przyjmować rezerwacje.
event-types-empty-link-label = Utwórz jakiś

# Markdown editor toolbar (templates/settings.html, templates/team_form.html)

editor-bold = Pogrubienie (Ctrl+B)
editor-italic = Kursywa (Ctrl+I)
editor-strikethrough = Przekreślenie
editor-code = Kod w tekście
editor-link = Wstaw link (Ctrl+K)
editor-toggle-preview = Pokaż lub ukryj podgląd
editor-preview = Podgląd

# Profile and settings (templates/settings.html)

## ilia-ov-settings
settings-choose-file = Wybierz plik
settings-no-file = Nie wybrano pliku
admin-provider-local = Hasło
admin-provider-oidc = SSO
admin-theme-vates-desc = Czerwień i kosmiczny błękit

settings-page-title = Ustawienia
settings-heading = Profil i ustawienia
settings-public-page-label = Twoja publiczna strona rezerwacji
settings-copy = Kopiuj
settings-copied = Skopiowano!
settings-open = Otwórz
settings-avatar = Awatar
settings-upload = Prześlij
settings-remove = Usuń
settings-display-name = Wyświetlana nazwa
settings-display-name-placeholder = Twoje imię i nazwisko
settings-username = Nazwa użytkownika
settings-username-hint = (używana w twoim adresie rezerwacji)
settings-username-pattern-title = Tylko małe litery, cyfry i myślniki
settings-username-help = Twoja publiczna strona rezerwacji:
settings-title = Stanowisko
settings-title-placeholder = np. Inżynierka oprogramowania, Menedżer produktu
settings-title-help = Widoczne na twoim profilu publicznym i na pasku bocznym.
settings-bio = Opis
settings-bio-placeholder = Napisz coś o sobie...
settings-bio-help = Widoczne na twojej publicznej stronie rezerwacji. Obsługuje **pogrubienie**, *kursywę*, ~~przekreślenie~~, `kod` i [linki](url).
settings-booking-email = E-mail do rezerwacji
settings-booking-email-help = Ten adres pojawi się na twoich publicznych stronach rezerwacji i w powiadomieniach e-mail. Zostaw puste, aby użyć adresu logowania.
settings-booking-email-warning = Upewnij się, że ten adres istnieje u twojego dostawcy poczty. W przeciwnym razie powiadomienia nie zostaną dostarczone.
settings-timezone = Strefa czasowa
settings-timezone-help = Twoje reguły dostępności i godziny rezerwacji są liczone w tej strefie czasowej.
settings-language = Język
settings-language-auto = Automatycznie (język przeglądarki)
settings-language-help = Wybierz język interfejsu albo zostaw Automatycznie, aby podążać za ustawieniem przeglądarki.
settings-public-page-language = Język publicznych stron rezerwacji
settings-public-page-language-visitor = Język każdego odwiedzającego
settings-public-page-language-host = Zawsze mój język
settings-public-page-language-help = Język Twojego publicznego profilu, stron rezerwacji i e-maili wysyłanych do gości. „Zawsze mój język” używa języka wybranego powyżej.
settings-error-public-page-language = Wybierz prawidłowy język publicznych stron rezerwacji.
settings-dynamic-group = Pozwól innym dodawać mnie do dynamicznych linków grupowych
settings-dynamic-group-help = Po włączeniu inni użytkownicy mogą tworzyć doraźne adresy spotkań zbiorowych, które obejmują ciebie (np. { $example }).
settings-lend-resource = Udostępnij mój dostęp do kalendarza na potrzeby rezerwacji zasobów
settings-lend-resource-help = Gdy rezerwacja musi zająć wspólny zasób (laboratorium demo, salę spotkań), do którego twoje konto kalendarza ma prawo zapisu, pozwól calrs użyć do tego twoich zapisanych danych logowania.
settings-default-availability = Domyślna dostępność
settings-default-availability-help = Twoje domyślne godziny pracy. Używane w dynamicznych linkach grupowych, gdy inni dodają cię do spotkania.
settings-copy-to-all = Skopiuj na wszystkie dni
settings-copy-to-all-title = Skopiuj przedziały z pierwszego włączonego dnia na wszystkie pozostałe włączone dni
settings-add-window = Dodaj przedział godzinowy
settings-remove-window = Usuń przedział
settings-save = Zapisz ustawienia
settings-appearance = Wygląd
settings-theme-system = Systemowy
settings-theme-light = Jasny
settings-theme-dark = Ciemny

# Sign in (templates/auth/login.html)

## ilia-ov-auth
ilia-mfa-recovery-copy = Skopiuj kody
ilia-mfa-recovery-copied = Skopiowano kody
ilia-mfa-recovery-copy-manual = Nie udało się skopiować. Kody są zaznaczone: skopiuj je ręcznie.

login-page-title = Logowanie
login-heading = Zaloguj się
login-subtitle = Zaloguj się na swoje konto calrs
login-sso = Zaloguj się przez SSO
login-or = lub
login-email = E-mail
login-password = Hasło
login-submit = Zaloguj się e-mailem
login-no-account = Nie masz jeszcze konta? { $link }
login-register-link = Zarejestruj się

# Registration (templates/auth/register.html)

register-page-title = Rejestracja
register-heading = Utwórz konto
register-subtitle = Zarejestruj nowe konto calrs
register-domains-limited = Rejestracja jest ograniczona do: { $domains }
register-name = Imię i nazwisko
register-name-placeholder = Twoje imię i nazwisko
register-email = E-mail
register-password = Hasło
register-password-hint = (min. 12 znaków)
register-submit = Utwórz konto
register-have-account = Masz już konto? { $link }
register-signin-link = Zaloguj się

# Authentication errors (src/auth.rs)

auth-error-rate-limited = Zbyt wiele prób logowania. Proszę spróbować ponownie później.
auth-error-invalid-credentials = Nieprawidłowy e-mail lub hasło
auth-error-internal = Błąd wewnętrzny
auth-error-registration-disabled = Rejestracja jest wyłączona.
auth-error-name-length = Imię i nazwisko muszą mieć od 1 do 255 znaków
auth-error-email-length = Adres e-mail musi mieć od 1 do 255 znaków
auth-error-email-invalid = Proszę podać prawidłowy adres e-mail
auth-error-email-domain = Domena e-mail jest niedozwolona
auth-error-password-length = Hasło musi mieć co najmniej 12 znaków
auth-error-email-taken = Ten adres e-mail jest już zarejestrowany
auth-error-create-failed = Nie udało się utworzyć konta

# Calendar source test and write-back setup (templates/source_test.html, templates/source_write_setup.html)

source-test-page-title = Źródło kalendarza
source-test-sync-heading = Synchronizacja: { $name }
source-test-heading = Test połączenia
source-write-page-title = Skonfiguruj zapis do kalendarza
source-write-back = Wróć do panelu
source-write-heading = Gdzie mają trafiać rezerwacje?
source-write-help = Gdy ktoś zarezerwuje z tobą spotkanie, calrs może automatycznie utworzyć wydarzenie w twoim kalendarzu. Wybierz, do którego kalendarza zapisywać rezerwacje dla { $name }.
source-write-save = Zapisz
source-write-skip = Pomiń na razie
source-write-sync-results = Wyniki synchronizacji

source-write-event-count =
    { $count ->
        [one] { $count } wydarzenie
        [few] { $count } wydarzenia
        [many] { $count } wydarzeń
       *[other] { $count } wydarzeń
    }

# Date overrides (templates/overrides.html)

overrides-page-title = Wyjątki dat
overrides-heading = Wyjątki dat
overrides-back-teams = Wróć do zespołów
overrides-back-event-types = Wróć do typów wydarzeń
overrides-intro = Dodaj wyjątki dla konkretnych dat w { $title }
overrides-add-heading = Dodaj nowy wyjątek
overrides-date = Data
overrides-type = Rodzaj wyjątku
overrides-type-blocked = Zablokuj cały dzień
overrides-type-custom = Własne godziny
overrides-start-time = Godzina rozpoczęcia
overrides-end-time = Godzina zakończenia
overrides-add-submit = Dodaj wyjątek
overrides-existing = Istniejące wyjątki
overrides-badge-blocked = zablokowany
overrides-badge-custom = własne godziny
overrides-delete = Usuń
overrides-delete-confirm = Usunąć ten wyjątek?
overrides-empty = Nie ma jeszcze wyjątków dat.<br>Użyj formularza powyżej, aby zablokować konkretne dni (święta, dni wolne) albo ustawić własne godziny.

# Public team page (templates/team_profile.html)

team-profile-subtitle = Wybierz typ wydarzenia, aby zarezerwować termin.
team-profile-empty = Nie ma jeszcze dostępnych typów wydarzeń.

# Availability troubleshoot (templates/troubleshoot.html, src/web/mod.rs)

## ilia-ov2-lists
# The troubleshoot timeline's legend entry for time already gone today.
troubleshoot-legend-past = Czas miniony
# Accessible names of the troubleshoot page's day arrows.
troubleshoot-prev-day = Poprzedni dzień
troubleshoot-next-day = Następny dzień
# Accessible name of a member's role select on the team settings page.
team-settings-role-of = Rola: { $name }

troubleshoot-page-title = Diagnostyka
troubleshoot-empty = Nie znaleziono typów wydarzeń. { $link }, aby zacząć diagnozować dostępność.
troubleshoot-empty-link-label = Utwórz jakiś
troubleshoot-subtitle = Sprawdź, dlaczego terminy dla { $title } są dostępne albo zablokowane
troubleshoot-duration = { $minutes } min
troubleshoot-buffer-before = { $minutes } min buforu przed
troubleshoot-buffer-after = { $minutes } min buforu po
troubleshoot-min-notice = { $minutes } min wyprzedzenia
troubleshoot-blocked-override = Zablokowane przez wyjątek daty (dzień wolny)
troubleshoot-custom-hours-active = Aktywny wyjątek z własnymi godzinami (zastępuje reguły tygodniowe)
troubleshoot-legend-available = Dostępne
troubleshoot-legend-calendar-event = Wydarzenie z kalendarza
troubleshoot-legend-booking = Rezerwacja
troubleshoot-legend-resource = Zasób zajęty
troubleshoot-legend-outside = Poza godzinami
troubleshoot-legend-buffer = Bufor / minimalne wyprzedzenie
troubleshoot-blocked-slots = Zablokowane terminy
troubleshoot-none-date-blocked = Ta data jest zablokowana przez wyjątek dostępności (dzień wolny). Brak dostępnych terminów.
troubleshoot-none-custom-hours = Aktywny jest wyjątek z własnymi godzinami, ale żaden przedział nie pasuje. Sprawdź ustawienia wyjątku.
troubleshoot-none-no-rules = Brak reguł dostępności dla tego dnia tygodnia. Tego typu wydarzenia nie można zarezerwować { $date }.
troubleshoot-none-all-bookable = Brak zablokowanych terminów w godzinach dostępności. Można rezerwować o każdej porze.
troubleshoot-label-outside = Poza dostępnością
troubleshoot-label-available = Dostępne
troubleshoot-label-min-notice = Minimalne wyprzedzenie ({ $minutes } min)
troubleshoot-label-beyond-horizon = Poza horyzontem rezerwacji ({ $days } dni)
troubleshoot-label-outside-date-range = Poza okresem rezerwacji
troubleshoot-label-buffer = Bufor ({ $minutes } min)
troubleshoot-label-resource-busy = Zasób zajęty: { $names }
troubleshoot-detail-around = Wokół: { $label }
troubleshoot-detail-around-booking = Wokół rezerwacji od { $guest }
troubleshoot-reason-calendar-event = Wydarzenie z kalendarza: { $label }
troubleshoot-reason-booking = Rezerwacja: { $label }

# Invite management (templates/invite_form.html)

invites-heading = Zaproszenia
invites-back-teams = Wróć do zespołów
invites-back-event-types = Wróć do typów wydarzeń
invites-intro = Wyślij linki z zaproszeniem do { $title }
invites-capped = <strong>Wpis ograniczono do { $max } odbiorców na jedno wysłanie.</strong> Resztę wyślij w kolejnej turze.
invites-failed-hint = — szczegóły znajdziesz w logach serwera.
invites-quick-link = Szybki link
invites-quick-link-help = Wygeneruj jednorazowy link i skopiuj go do schowka.
invites-get-link = Pobierz link
invites-or-email = Albo wyślij e-mailem
invites-recipients = Odbiorcy
invites-recipients-hint = (jeden adres w wierszu, maksymalnie { $max })
invites-message = Wiadomość osobista
invites-message-hint = (opcjonalnie, trafia do wszystkich odbiorców)
invites-message-placeholder = Chętnie pokażę ci demo...
invites-expires-in = Wygasa za
invites-expires-days = { $days } dni
invites-expires-never = Nigdy
invites-allow-multiple = Zezwól na wiele rezerwacji na odbiorcę
invites-send = Wyślij zaproszenia
invites-sent-heading = Wysłane zaproszenia
invites-badge-expired = wygasłe
invites-badge-used = wykorzystane
invites-badge-active = aktywne
invites-sent-by = Wysłane przez { $name }
invites-uses = { $used }/{ $max } użyć
invites-expires-at = Wygasa { $date }
invites-copy-link = Kopiuj link
invites-delete = Usuń
invites-delete-confirm = Usunąć to zaproszenie?
invites-empty = Nie wysłano jeszcze żadnych zaproszeń. Użyj formularza powyżej, aby przesłać komuś link do rezerwacji.
invites-js-generating = Generowanie...
invites-js-copied = Skopiowano!
invites-js-error = Błąd

invites-sent-count =
    { $count ->
        [one] Wysłano { $count } zaproszenie.
        [few] Wysłano { $count } zaproszenia.
        [many] Wysłano { $count } zaproszeń.
       *[other] Wysłano { $count } zaproszeń.
    }

invites-skipped-invalid =
    { $count ->
        [one] Pominięto { $count } nieprawidłowy wiersz:
        [few] Pominięto { $count } nieprawidłowe wiersze:
        [many] Pominięto { $count } nieprawidłowych wierszy:
       *[other] Pominięto { $count } nieprawidłowych wierszy:
    }

invites-skipped-duplicate =
    { $count ->
        [one] Pominięto { $count } zduplikowany wiersz:
        [few] Pominięto { $count } zduplikowane wiersze:
        [many] Pominięto { $count } zduplikowanych wierszy:
       *[other] Pominięto { $count } zduplikowanych wierszy:
    }

invites-failed =
    { $count ->
        [one] { $count } zaproszenie nie powiodło się (baza lub SMTP):
        [few] { $count } zaproszenia nie powiodły się (baza lub SMTP):
        [many] { $count } zaproszeń nie powiodło się (baza lub SMTP):
       *[other] { $count } zaproszeń nie powiodło się (baza lub SMTP):
    }

# Calendar source form (templates/source_form.html)

source-form-title-edit = Edytuj źródło kalendarza
source-form-title-add = Dodaj kalendarz
source-form-heading-edit = Edytuj źródło kalendarza
source-form-heading-add = Podłącz kalendarz
source-form-subtitle-edit = Zaktualizuj połączenie. Zostaw hasło puste, aby zachować dotychczasowe. Po zmianie adresu lub nazwy użytkownika uruchom synchronizację, aby odświeżyć listę wykrytych kalendarzy.
source-form-subtitle-add = Podłącz serwer CalDAV albo Microsoft Exchange (EWS), aby calrs mógł sprawdzać dostępność, gdy goście rezerwują spotkania.
source-form-backend = Backend

## ilia-ov3-dash
# The "Other / generic CalDAV" preset on the new-source form; it also becomes the default source name.
source-form-preset-other = Inny serwer CalDAV

source-form-preset = Ustawienie wstępne
source-form-connect-google = Połącz z Google
source-form-google-unavailable = Kalendarz Google jest niedostępny. Skontaktuj się z administracją.
source-form-name = Wyświetlana nazwa
source-form-name-placeholder = Mój kalendarz
source-form-url-caldav = Adres CalDAV
source-form-url-ews = Adres punktu końcowego EWS
source-form-username = Nazwa użytkownika
source-form-password = Hasło
source-form-password-keep = Zostaw puste, aby zachować dotychczasowe
source-form-password-placeholder = Hasło aplikacji albo hasło konta
source-form-skip-test = Pomiń test połączenia
source-form-skip-test-help = Użyj tego, jeśli test się zawiesza (zdarza się w niektórych instalacjach BlueMind lub Zimbry). Połączenie możesz przetestować później.
source-form-save = Zapisz zmiany
source-form-add = Dodaj źródło kalendarza
source-form-help-google-configured = Kliknij przycisk poniżej, aby zezwolić calrs na dostęp do twojego Kalendarza Google.
source-form-help-google-unconfigured = Integracja z Kalendarzem Google nie jest jeszcze skonfigurowana. Poproś administrację o wprowadzenie danych OAuth2 Google w panelu administracyjnym.

# Calendar source form: provider help (templates/source_form.html)

source-form-help-bluemind = <strong>BlueMind</strong> — Użyj punktu końcowego DAV swojego serwera BlueMind.<br> Zwykle: <code>https://mail.yourcompany.com/dav/</code><br> Nazwa użytkownika to twój <strong>adres e-mail</strong> (np. <code>alice@yourcompany.com</code>), a nie sama nazwa logowania.<br> Jeśli test połączenia się zawiesza, zaznacz „Pomiń test połączenia” i zsynchronizuj bezpośrednio.
source-form-help-nextcloud = <strong>Nextcloud</strong> — Użyj katalogu głównego WebDAV, a nie adresu konkretnego kalendarza.<br> Zwykle: <code>https://cloud.example.com/remote.php/dav</code>
source-form-help-fastmail = <strong>Fastmail</strong> — Podaj pełny adres e-mail w ścieżce adresu.<br> Przykład: <code>https://caldav.fastmail.com/dav/calendars/user/you@fastmail.com/</code><br> Użyj hasła aplikacji (Settings &rarr; Privacy &amp; Security &rarr; Integrations).
source-form-help-icloud = <strong>iCloud</strong> — Użyj <code>https://caldav.icloud.com/</code><br> Potrzebujesz hasła aplikacji z <a href="https://appleid.apple.com" target="_blank" style="color: var(--accent);">appleid.apple.com</a> (Bezpieczeństwo &rarr; Hasła aplikacji).
source-form-help-zimbra = <strong>Zimbra</strong> — Użyj punktu końcowego DAV swojego serwera Zimbra.<br> Zwykle: <code>https://mail.example.com/dav/</code>
source-form-help-sogo = <strong>SOGo</strong> — Użyj punktu końcowego DAV SOGo.<br> Zwykle: <code>https://mail.example.com/SOGo/dav/</code>
source-form-help-radicale = <strong>Radicale</strong> — Użyj adresu głównego serwera.<br> Zwykle: <code>https://cal.example.com/</code>
source-form-help-baikal = <strong>Baikal</strong>: Użyj punktu końcowego DAV swojego serwera Baikal.<br> Zwykle: <code>https://cal.example.com/dav.php/</code>
source-form-help-exchange = <strong>Microsoft Exchange (EWS)</strong>. Użyj punktu końcowego SOAP:<br> <code>https://mail.example.com/EWS/Exchange.asmx</code><br> Nazwa użytkownika to adres skrzynki; hasło musi działać z HTTP Basic po TLS (włącz je na skrzynce serwisowej, jeśli w twojej organizacji Basic jest wyłączone).<br> Pamiętaj też, aby wybrać <strong>Microsoft Exchange (EWS)</strong> na liście Backend powyżej.
source-form-help-google = <strong>Kalendarz Google</strong>: połączenie przez OAuth2. Hasło nie jest potrzebne.<br>
source-form-help-other = Podaj <strong>główny adres DAV</strong> swojego serwera CalDAV — nie adres pojedynczego kalendarza ani link publiczny.<br> calrs sam wykryje twoje kalendarze przez PROPFIND (RFC 4791).

# Markdown editor toolbar, short labels (templates/team_form.html, templates/team_settings.html)

editor-bold-short = Pogrubienie
editor-italic-short = Kursywa
editor-link-short = Wstaw link

# Team creation (templates/team_form.html)

team-form-heading = Nowy zespół
team-form-name = Nazwa zespołu
team-form-name-placeholder = Inżynieria
team-form-slug = Identyfikator
team-form-slug-hint = (identyfikator przyjazny adresom URL)
team-form-slug-pattern-title = Tylko małe litery, cyfry i myślniki
team-form-description = Opis
team-form-optional = (opcjonalnie)
team-form-description-placeholder = Czym zajmuje się ten zespół...
team-form-description-help = Widoczny na stronie zespołu. Obsługuje **pogrubienie**, *kursywę* i [linki](url).
team-form-visibility = Widoczność
team-form-public = Publiczny
team-form-private = Prywatny
team-form-visibility-help = Zespoły prywatne dostają token zaproszenia do udostępniania. Publiczne są widoczne na stronie profilu zespołu.
team-form-members = Członkowie
team-form-members-help = Zostaniesz automatycznie dodany jako administrator zespołu. Dodaj pojedynczych użytkowników albo powiąż grupy OIDC.
team-form-search-placeholder = Szukaj użytkowników lub grup...
team-form-search-users = Użytkownicy
team-form-search-groups = Grupy OIDC
team-form-you = (ty)
team-form-submit = Utwórz zespół

# Team settings (templates/team_settings.html)

team-settings-page-title = Ustawienia
team-settings-subtitle = Ustawienia zespołu — mogą je edytować administratorzy zespołu.
team-settings-public-url = Adres publiczny
team-settings-public-url-help = Przez ten link może rezerwować każdy.
team-settings-invite-link = Link z zaproszeniem
team-settings-invite-link-help = Udostępnij ten link, aby dać dostęp do strony rezerwacji tego prywatnego zespołu.
team-settings-avatar = Awatar zespołu
team-settings-profile = Profil
team-settings-description-placeholder = Opowiedz o tym zespole...
team-settings-description-help = Widoczny na publicznej stronie rezerwacji zespołu. Obsługuje **pogrubienie**, *kursywę* i [linki](url).
team-settings-visibility-help = Zespoły publiczne są widoczne na stronie profilu zespołu. Prywatne wymagają linku z zaproszeniem.
team-settings-members-help = Zarządzaj składem tego zespołu. Dodaj pojedynczych użytkowników albo powiąż grupy OIDC, aby synchronizowały się automatycznie.
team-settings-role-member = Członek
team-settings-role-admin = Administrator
team-settings-oidc-group = Grupa OIDC
team-settings-remove = Usuń
team-settings-save = Zapisz zmiany
team-settings-danger-zone = Strefa zagrożenia
team-settings-danger-help = Trwale usuń ten zespół. Typy wydarzeń zostaną odłączone, a nie usunięte. Tej operacji nie można cofnąć.
team-settings-delete = Usuń ten zespół
team-settings-delete-confirm = Usunąć zespół „{ $name }”? Tej operacji nie można cofnąć.

# Event type form (templates/event_type_form.html)

etf-heading-edit = Edytuj typ wydarzenia
etf-heading-new = Nowy typ wydarzenia
etf-disabled-banner = Ten typ wydarzenia jest wyłączony, więc nikt nie może go jeszcze zarezerwować.
etf-team = Zespół
etf-team-hint = (opcjonalnie — zostaw puste dla osobistego typu wydarzenia)
etf-team-personal = Osobisty
etf-scheduling-mode = Tryb przydzielania
etf-mode-round-robin = Rotacyjny — przydziel jednemu dostępnemu członkowi
etf-mode-collective = Zbiorowy — wszyscy członkowie muszą być dostępni
etf-scheduling-mode-help = Tryb rotacyjny przydziela rezerwację jednemu dostępnemu członkowi (najpierw najmniej obciążonemu). Tryb zbiorowy wymaga, aby wszyscy członkowie byli wolni w tym samym czasie.
etf-title = Tytuł
etf-title-placeholder = 30-minutowa rozmowa wstępna
etf-slug = Identyfikator
etf-slug-placeholder = tworzony automatycznie z tytułu
etf-description-placeholder = Krótka rozmowa wstępna, żeby omówić...
etf-description-help = Widoczny na stronie rezerwacji. Obsługuje **pogrubienie**, *kursywę* i [linki](url).
etf-location = Miejsce
etf-location-link = Rozmowa wideo (stały adres)
etf-location-jitsi = Jitsi (pokój tworzony automatycznie)
etf-location-webhook = Webhook (własny dostawca)
etf-location-phone = Telefon
etf-location-in-person = Osobiście
etf-location-custom = Własne
etf-location-details = Szczegóły
etf-location-details-placeholder = https://meet.example.com/moj-pokoj
etf-pattern-placeholder = Zostaw puste, aby użyć domyślnego wzorca organizacji
etf-duration = Czas trwania (minuty)
etf-slot-interval = Odstęp między terminami (minuty)
etf-slot-interval-placeholder = Tak jak czas trwania
etf-slot-interval-help = Jak często zaczynają się terminy. Zostaw puste, aby odpowiadały czasowi trwania.
etf-required-members = Wymagani członkowie
etf-required-members-help = Wszyscy zaznaczeni członkowie muszą być wolni, aby termin został zaproponowany. Odznacz osoby, które chcesz wykluczyć (ich dostępność zostanie pominięta).
etf-member-priority = Priorytet członków
etf-member-priority-help = Członkowie z wyższym priorytetem dostają rezerwacje w pierwszej kolejności, jeśli są dostępni. Przy równym priorytecie decyduje liczba ostatnich rezerwacji.
etf-member-timezone-title = Strefa czasowa członka. Jego osobiste godziny pracy są liczone w tej strefie.
etf-priority-high = Wysoki
etf-priority-medium = Średni
etf-priority-low = Niski
etf-section-availability = Dostępność
etf-timezone-help = Godziny poniżej są liczone w tej strefie czasowej. Przy zespołowych typach wydarzeń wybierz strefę pracy zespołu (niekoniecznie strefę osoby tworzącej).
etf-reset-default = Przywróć moje domyślne
etf-reset-default-title = Zastąp te godziny domyślną dostępnością z twojego profilu
etf-availability-prefilled = Wypełnione na podstawie twojej { $link }. Możesz to tutaj zmienić dla tego typu wydarzenia.
etf-availability-prefilled-link = domyślnej dostępności
etf-section-buffers = Bufory i wyprzedzenie
etf-buffer-before = Bufor przed (min)
etf-buffer-after = Bufor po (min)
etf-min-notice = Minimalne wyprzedzenie
etf-min-notice-help = Z jakim wyprzedzeniem trzeba rezerwować.
etf-section-limits = Limity rezerwacji
etf-first-slot-only = Jeden termin dziennie
etf-first-slot-only-help = Pokazuj tylko najwcześniejszy dostępny termin każdego dnia.

## ilia-ov4-dash
etf-freq-per-member = Na członka zespołu

etf-freq-limit = Ogranicz częstotliwość rezerwacji
etf-freq-limit-help = Ogranicz, ile razy w danym okresie można zarezerwować to wydarzenie.
etf-add-limit = Dodaj limit
etf-section-options = Opcje rezerwacji
etf-requires-confirmation = Wymaga potwierdzenia
etf-requires-confirmation-help = Rezerwacje pozostaną oczekujące, dopóki nie zatwierdzisz ich w panelu.
etf-sms = Powiadomienia SMS
etf-sms-off = Wyłączone, nie pytamy o numer
etf-sms-optional = Opcjonalne, goście mogą podać numer
etf-sms-required = Wymagane, goście muszą podać numer
etf-sms-help = Wysyła gościowi SMS, oprócz e-maila, gdy jego rezerwacja zostanie potwierdzona, przeniesiona, odwołana albo ma się zaraz zacząć. Kto zostawi pole puste, po prostu nie dostanie SMS-a. Wymaga bramki SMS w { $link }.
etf-admin-panel-link = panelu administracyjnym
etf-additional-guests = Dodatkowi goście
etf-guests-none = Goście nie mogą dodawać innych osób
etf-additional-guests-help = Pozwól osobie rezerwującej zaprosić kolejnych uczestników, którzy dostaną zaproszenie do kalendarza.
etf-default-view = Domyślny widok kalendarza
etf-view-month = Miesiąc — siatka kalendarza z listą terminów
etf-view-week = Tydzień — kolumny dla 7 dni z terminami
etf-view-column = Kolumna — dni na liście wraz z terminami
etf-view-week-short = tygodniowy
etf-view-column-short = kolumnowy
etf-default-view-help = Widok, który goście zobaczą najpierw. Mogą go zmienić w dowolnej chwili.
etf-conflict-calendars = Kalendarze do sprawdzania kolizji
etf-conflict-calendars-help = Wybierz, które kalendarze sprawdzać pod kątem kolizji. Bez wyboru używane są wszystkie.
etf-no-resources = Nie skonfigurowano jeszcze wspólnych zasobów. Dodaj jakiś (laboratorium demo, salę spotkań) w { $link }, aby móc go tutaj wymagać.
etf-section-access = Dostęp i powiadomienia
etf-visibility-public = Publiczny — widoczny w twoim profilu
etf-visibility-internal = Wewnętrzny — każdy współpracownik może generować linki z zaproszeniem
etf-visibility-private = Prywatny — tylko przez link z zaproszeniem
etf-visibility-help = Określa, kto może zobaczyć i zarezerwować ten typ wydarzenia.
etf-vis-internal = Wewnętrzny
etf-reminder = Przypomnienie o rezerwacji
etf-reminder-none = Bez przypomnienia
etf-reminder-help = Wyślij przypomnienie e-mailem tobie i twojemu gościowi przed spotkaniem.
etf-dynamic-group = Dynamiczny link grupowy
etf-dynamic-group-help = Utwórz doraźny link do spotkania, który sprawdza dostępność twoją i innych użytkowników.
etf-dynamic-group-search = Znajdź użytkownika do dodania...
etf-dynamic-group-note = Pokazywani są tylko użytkownicy, którzy zezwalają na dynamiczne linki grupowe.
etf-dynamic-group-url = Adres linku grupowego
etf-watcher-teams = Zespoły obserwujące
etf-watcher-teams-help = Wybrane zespoły dostaną powiadomienie o każdej rezerwacji. Ich członkowie mogą przejąć rezerwację, aby w niej uczestniczyć.
etf-save = Zapisz zmiany
etf-create = Utwórz typ wydarzenia
etf-js-loading = Wczytywanie...
etf-js-no-default = Brak wartości domyślnej
etf-js-reset-done = Przywrócono!
etf-js-error = Błąd
etf-js-remove-limit = Usuń limit
etf-period-day = Dziennie
etf-period-week = Tygodniowo
etf-period-month = Miesięcznie
etf-period-year = Rocznie

# Event type form: runtime summary hints (templates/event_type_form.html)

# %1 and %2 are substituted client-side; the values are only known once a field is edited.

etf-hint-no-days = Nie ustawiono dni
etf-hint-every-day = Codziennie
etf-fmt-day-one = %1 dzień
etf-fmt-day-other = %1 dni
etf-fmt-hours = %1 godz.
etf-fmt-minutes = %1 min
etf-hint-buffer-both = %1 min przed, %2 min po
etf-hint-buffer-before = %1 min buforu przed
etf-hint-buffer-after = %1 min buforu po
etf-hint-notice = %1 wyprzedzenia
etf-hint-no-buffers = Bez buforów, można rezerwować o każdej porze
etf-hint-max = Maks. %1
etf-hint-period-day = /dzień
etf-hint-period-week = /tydzień
etf-hint-period-month = /miesiąc
etf-hint-period-year = /rok
etf-hint-no-limits = Bez limitów
etf-hint-confirmation-required = Wymaga potwierdzenia
etf-hint-auto-confirmed = Potwierdzane automatycznie
etf-hint-extra-guests-one = do %1 dodatkowego gościa
etf-hint-extra-guests-other = do %1 dodatkowych gości
etf-hint-view = widok %1
etf-hint-reminder = przypomnienie %1 wcześniej
etf-hint-no-reminder = bez przypomnienia

etf-guests-up-to =
    { $count ->
        [one] Do { $count } dodatkowego gościa
        [few] Do { $count } dodatkowych gości
        [many] Do { $count } dodatkowych gości
       *[other] Do { $count } dodatkowych gości
    }

etf-reminder-hours =
    { $count ->
        [one] { $count } godzinę wcześniej
        [few] { $count } godziny wcześniej
        [many] { $count } godzin wcześniej
       *[other] { $count } godzin wcześniej
    }

etf-reminder-days =
    { $count ->
        [one] { $count } dzień wcześniej
        [few] { $count } dni wcześniej
        [many] { $count } dni wcześniej
       *[other] { $count } dni wcześniej
    }

# Event type form: preset banners and meeting-pattern help (templates/event_type_form.html)
# Literal braces are escaped as {"{"} because Fluent reads a bare { as a placeable.

etf-preset-public = Tworzysz <strong>publiczny</strong> typ wydarzenia &mdash; zarezerwuje każdy, kto ma link.
etf-preset-private = Tworzysz <strong>prywatny</strong> typ wydarzenia &mdash; zarezerwują tylko osoby, które zaprosisz.
etf-preset-internal = Tworzysz <strong>wewnętrzny</strong> typ wydarzenia &mdash; każdy współpracownik może udostępnić link do rezerwacji.
etf-preset-team = Tworzysz <strong>zespołowy</strong> typ wydarzenia &mdash; rezerwacje są rozdzielane między członków zespołu.
etf-pattern-hint = Opcjonalny własny wzorzec. Znaczniki: <code>{"{"}username{"}"}</code>, <code>{"{"}event{"}"}</code>, <code>{"{"}date{"}"}</code>, <code>{"{"}random{"}"}</code>. Zostaw puste, aby użyć domyślnego wzorca organizacji ustawionego przez administrację.
etf-pattern-random-warning = Ten wzorzec nie zawiera znacznika <code>{"{"}random{"}"}</code>. Dwie rezerwacje tego typu wydarzenia w tym samym dniu trafią do tego samego pokoju, a drugi gość może wejść na spotkanie pierwszego. Stałych pokojów używaj tylko wtedy, gdy właśnie o to ci chodzi.
etf-webhook-hint = Adres spotkania dla każdej rezerwacji pobierany jest z webhooka skonfigurowanego przez administrację w Administracja &rarr; Webhook spotkań. Tutaj adres nie jest potrzebny.

# Admin panel (templates/admin.html)

admin-page-title = Administracja
admin-heading = Panel administracyjny
admin-action-refused = Odmowa wykonania:
admin-error-last-admin = To ostatni aktywny administrator. Najpierw nadaj uprawnienia administratora innemu użytkownikowi lub włącz innego administratora.
admin-logo = Logo firmy
admin-logo-help = Wyświetlane na publicznych stronach rezerwacji. PNG, JPEG, GIF lub WebP, maks. 2 MB.
admin-logo-refused = Nie zapisano logo. Prześlij obraz PNG, JPEG, GIF lub WebP o rozmiarze maks. 2 MB.
admin-favicon = Favicon
admin-favicon-help = Wyświetlana na kartach przeglądarki i w zakładkach. Zalecane: kwadratowy PNG o rozmiarze co najmniej 32×32 piksele, maks. 512 KB. Bez niej wyświetlany jest krab calrs.
admin-favicon-refused = Nie zapisano favicony. Prześlij obraz PNG, JPEG, GIF lub WebP o rozmiarze maks. 512 KB.
admin-upload-save-failed = Nie udało się zapisać pliku w katalogu danych serwera. Szczegóły znajdziesz w logu serwera.

# User and team avatar uploads (#307)
avatar-upload-refused = Obraz nie został zapisany. Prześlij obraz PNG, JPEG, GIF lub WebP o rozmiarze do 2 MB.
avatar-upload-save-failed = Nie udało się zapisać obrazu na serwerze. Spróbuj ponownie później albo poproś administratora o sprawdzenie dziennika serwera.
admin-company-link = Link firmowy
admin-company-link-help = Na publicznych stronach rezerwacji logo prowadzi pod ten adres. Zostaw puste, aby nie tworzyć linku.
admin-theme = Motyw
admin-theme-help = Wybierz motyw kolorystyczny dla wszystkich stron. Przełącznik jasny/ciemny działa niezależnie — motywy dostosowują się do obu trybów.
admin-theme-default = Domyślny
admin-theme-default-desc = Czysty błękit
admin-theme-nord-desc = Arktyczny szron
admin-theme-dracula-desc = Ciemny fiolet
admin-theme-gruvbox-desc = Ciepłe retro
admin-theme-solarized-desc = Klasyk Ethana
admin-theme-tokyo-desc = Neonowe miasto
admin-theme-custom = Własny
admin-theme-custom-desc = Twoje kolory
admin-custom-colors = Własne kolory
admin-color-accent = Kolor akcentu
admin-color-accent-hover = Akcent po najechaniu
admin-color-bg = Tło
admin-color-surface = Powierzchnia
admin-color-text = Tekst
admin-save-theme = Zapisz motyw
admin-users = Użytkownicy ({ $count })
admin-user-filter = Filtruj po nazwie lub adresie e-mail…
admin-badge-admin = administrator
admin-badge-disabled = wyłączony
admin-impersonate = Wciel się
admin-demote = Odbierz uprawnienia
admin-promote = Nadaj uprawnienia
admin-disable = Wyłącz
admin-enable = Włącz
admin-delete = Usuń
admin-no-users-match = Żaden użytkownik nie pasuje do filtra.
admin-no-users = Nie ma jeszcze użytkowników.
admin-groups = Grupy ({ $count })
admin-group-filter = Filtruj po nazwie grupy…
admin-group-name = Nazwa grupy
admin-weight = waga:
admin-no-groups-match = Żadna grupa nie pasuje do filtra.
admin-no-groups = Nie zsynchronizowano jeszcze żadnych grup. Grupy są pobierane automatycznie od twojego dostawcy OIDC.
admin-auth-settings = Ustawienia logowania
admin-registration-enabled = Rejestracja włączona
admin-allowed-domains = Dozwolone domeny e-mail
admin-allowed-domains-hint = (oddzielone przecinkami, puste oznacza wszystkie)
admin-save-auth = Zapisz ustawienia logowania
admin-system-settings = Ustawienia systemowe
admin-base-url = Adres bazowy
admin-base-url-help = Publiczny adres tej instancji. Używany przy przekierowaniach OIDC i w linkach w e-mailach (zatwierdź/odrzuć, odwołaj, przypomnienia).
admin-private-hosts = Lista dozwolonych hostów prywatnych
admin-private-hosts-help = Nazwy hostów, oddzielone przecinkami, którym wolno wskazywać prywatne lub zarezerwowane adresy IP w źródłach CalDAV/EWS (wyjątek od ochrony przed SSRF). Dodawaj tylko hosty, które kontrolujesz (na przykład serwer kalendarza w tej samej sieci Dockera). Zostaw puste, aby ochrona działała dla wszystkich hostów.
admin-german-formal = Forma zwracania się po niemiecku
admin-german-formal-du = Nieformalna (du)
admin-german-formal-sie = Formalna (Sie)
admin-german-formal-help = Jak strony i e-maile zwracają się do odwiedzających, których przeglądarka prosi o niemiecki. Użytkownicy, którzy wybrali język w Profil i ustawienia, zachowują swój.
admin-unset-env = Usuń zmienną środowiskową, aby móc edytować to tutaj.
admin-save-system = Zapisz ustawienia systemowe
admin-status = Stan:
admin-status-enabled = włączone
admin-status-disabled = wyłączone
admin-status-disabled-paren = (wyłączone)
admin-status-configured = skonfigurowane
admin-status-not-configured = nieskonfigurowane
admin-via-environment = (przez środowisko)
admin-issuer = Wystawca:
admin-client-id = Identyfikator klienta:
admin-instance = Instancja:
admin-oidc-settings = Ustawienia OIDC
admin-oidc-enabled = OIDC włączone
admin-issuer-url = Adres wystawcy
admin-client-id-label = Identyfikator klienta
admin-client-secret = Sekret klienta
admin-keep-current-hint = (zostaw puste, aby zachować obecny)
admin-keep-current-set-hint = (zostaw puste, aby zachować obecny — jest już ustawiony)
admin-keep-unchanged = Zostaw puste, aby nic nie zmieniać
admin-oidc-auto-register = Automatycznie rejestruj nowych użytkowników z OIDC
admin-save-oidc = Zapisz ustawienia OIDC
admin-google = Kalendarz Google (OAuth2)
admin-save-google = Zapisz ustawienia OAuth2 Google
admin-captcha = Captcha
admin-instance-url = Adres instancji
admin-site-key = Klucz witryny
admin-secret = Sekret
admin-widget-url = Adres skryptu widżetu
admin-widget-url-help = Zmień, jeśli CDN jest zablokowany. Zmiany działają od razu po zapisaniu.
admin-wasm-url = Adres pliku WASM widżetu
admin-wasm-url-help = Skąd widżet pobiera swój solver. Wskaż plik WASM udostępniany przez twój serwer Cap, aby strony rezerwacji nie łączyły się z cdn.jsdelivr.net. Zostaw puste, aby użyć wartości domyślnej.
admin-captcha-wasm-url-invalid = Adres pliku WASM widżetu musi być bezwzględnym adresem http(s).
admin-captcha-disable-help = Zostaw adres instancji, klucz witryny i sekret puste, aby wyłączyć captchę na stronach rezerwacji.
admin-save-captcha = Zapisz ustawienia captchy
admin-resources = Zasoby
admin-resources-help = Wspólne zasoby do rezerwacji (laboratorium demo, sale spotkań) oparte na kanale kalendarza. Po powiązaniu z typami wydarzeń zajęty zasób blokuje rezerwacje.
admin-resource-stats = Wydarzenia w pamięci podręcznej: { $events } &middot; Powiązany z { $attached } typami wydarzeń
admin-never = nigdy
admin-resource-sync-failed = (ostatnia próba nie powiodła się: { $error })
admin-writeback-enabled = Zapis: włączony ({ $via })
admin-writeback-readonly = Zapis: tylko do odczytu
admin-teams-allowed = Dozwolone zespoły:
admin-teams-allowed-none = brak (tylko administratorzy globalni)
admin-sync-now = Synchronizuj teraz
admin-test-write = Przetestuj zapis
admin-delete-resource-confirm = Usunąć ten zasób? Typy wydarzeń, które go używają, przestaną go sprawdzać.
admin-name = Nazwa
admin-name-help = Zostaw puste, aby pobrać nazwę z kanału.
admin-feed-url = Adres kanału ICS (adres publikacji)
admin-feed-url-help = BlueMind: publiczny lub prywatny adres kalendarza danego zasobu.
admin-caldav-url = Adres kolekcji CalDAV (do zapisu)
admin-caldav-url-help = Opcjonalne. W BlueMind jest wyprowadzany automatycznie z adresu kanału.
admin-caldav-username = Nazwa użytkownika CalDAV
admin-caldav-password = Hasło CalDAV
admin-resource-teams = Zespoły uprawnione do korzystania z tego zasobu
admin-resource-teams-help = Administratorzy tych zespołów mogą powiązać zasób ze swoimi zespołowymi typami wydarzeń. Puste: tylko administratorzy globalni.
admin-no-teams = Nie ma jeszcze zespołów.
admin-save-resource = Zapisz zasób
admin-add-resource = Dodaj zasób
admin-jitsi = Jitsi (automatycznie tworzone linki do spotkań)
admin-jitsi-help = Gdy miejscem typu wydarzenia jest „Jitsi (pokój tworzony automatycznie)”, calrs buduje dla każdej rezerwacji nowy adres pokoju, dodając poniższy wzorzec do twojego adresu bazowego Jitsi. Nie jest potrzebne żadne wywołanie zewnętrznego API.
admin-display-name = Wyświetlana nazwa
admin-jitsi-display-name-placeholder = np. Meet DYB
admin-jitsi-display-name-help = Pokazywana gościom w wyborze terminu i w formularzu rezerwacji. Puste oznacza „Rozmowa wideo”.
admin-room-pattern = Wzorzec nazwy pokoju
admin-jitsi-disable-help = Zostaw adres bazowy pusty, aby wyłączyć automatyczne tworzenie linków Jitsi.
admin-save-jitsi = Zapisz ustawienia Jitsi
admin-meeting-webhook = Webhook spotkań (własny dostawca)
admin-webhook-url = Adres webhooka
admin-webhook-display-name-placeholder = np. Zoom, Whereby, Custom Meet
admin-webhook-display-name-help = Pokazywana gościom zamiast ogólnej etykiety „Rozmowa wideo”.
admin-authentication = Uwierzytelnianie
admin-auth-none = Brak
admin-auth-hmac = HMAC-SHA256 (nagłówek X-Calrs-Signature)
admin-shared-secret = Wspólny sekret
admin-webhook-disable-help = Zostaw adres pusty, aby wyłączyć webhook spotkań.
admin-save-webhook = Zapisz ustawienia webhooka
admin-smtp = Ustawienia SMTP
admin-smtp-test-sent = Wysłano testowy e-mail.
admin-smtp-test-failed = Nie udało się wysłać testowego e-maila. Sprawdź logi serwera i swoje ustawienia SMTP.
admin-smtp-env-error = Błąd konfiguracji SMTP ze środowiska:
admin-smtp-host = Host:
admin-smtp-from = Nadawca:
admin-smtp-enabled = SMTP włączone
admin-host = Host
admin-port = Port
admin-tls-mode = Tryb TLS
admin-tls-starttls = STARTTLS (port 587)
admin-tls-implicit = Niejawny TLS (port 465)
admin-tls-none = Brak, bez szyfrowania (tylko lokalny MTA)
admin-smtp-username-hint = (zostaw puste dla przekaźnika bez uwierzytelniania)
admin-from-email = Adres nadawcy
admin-from-name = Nazwa nadawcy
admin-save-smtp = Zapisz ustawienia SMTP
admin-send-test-email = Wyślij testowy e-mail na
admin-send-test-email-hint = (domyślnie adres twojego konta)
admin-send-test-email-btn = Wyślij testowy e-mail
admin-smtp-clear-confirm = Usunąć konfigurację SMTP zapisaną w bazie danych?
admin-clear-db-config = Wyczyść konfigurację z bazy danych
admin-sms = Ustawienia SMS
admin-sms-help = Opcjonalne. SMS-y są wysyłane tylko przy rezerwacjach typów wydarzeń z włączonymi „Powiadomieniami SMS” i tylko wtedy, gdy gość podał numer telefonu.
admin-sms-test-sent = Wysłano wiadomość testową.
admin-sms-test-checked = Dane logowania zaakceptowane.
admin-sms-test-error = Bramka SMS odrzuciła żądanie.
admin-sms-captcha-warning = Formularz rezerwacji jest publiczny, a numer odbiorcy podaje gość, więc SMS bez captchy to otwarty przekaźnik, za który ktoś inny może obciążyć twój rachunek. Skonfiguruj captchę powyżej i ogranicz kraje docelowe w ustawieniach swojej bramki.
admin-sms-sent-today = Wysłane dzisiaj:
admin-sms-of-cap = z { $cap }
admin-sms-config-error = Błąd konfiguracji SMS:
admin-sms-gateway = Bramka:
admin-sms-account = Konto:
admin-sms-sender = Nadawca:
admin-sms-enabled = SMS włączone
## ilia-ov3-admin
admin-sms-provider-webhook = Własny webhook
admin-sms-webhook-secret = Sekret HMAC
admin-sms-gateway-label = Bramka
admin-required-on-switch = Wymagane przy zmianie bramki
## ilia-ov2-settings
admin-sms-sender-label = Nadawca
admin-sms-api-base-url = Bazowy URL API
admin-sms-region-base-url = Bazowy URL regionu
admin-sms-webhook-url = URL webhooka
admin-sms-twilio-sender-hint = Numer Twilio w formacie E.164 (+15551234567) lub alfanumeryczny identyfikator nadawcy, tam gdzie kraj docelowy na to pozwala.
admin-sms-twilio-base-url-hint = Opcjonalne. Domyślnie https://api.twilio.com. Używaj https, tym połączeniem przechodzą dane uwierzytelniające.
admin-sms-gatewayapi-sender-hint = Do 11 znaków alfanumerycznych lub 15 cyfr.
admin-sms-gatewayapi-base-url-hint = Opcjonalne. Domyślnie https://gatewayapi.com; użyj https://gatewayapi.eu, aby ruch pozostał w UE.
admin-sms-sevenio-sender-hint = Do 11 znaków alfanumerycznych lub 16 cyfr.
admin-sms-sevenio-base-url-hint = Opcjonalne. Domyślnie https://gateway.seven.io.
admin-sms-webhook-sender-hint = Opcjonalne. Przekazywane do twojego endpointu jako „sender”.
admin-sms-webhook-base-url-hint = calrs wysyła tutaj POST {"{"}"to", "text", "sender"{"}"} i oczekuje odpowiedzi 2xx.

admin-sms-docs = Dokumentacja API { $provider }
admin-sms-country = Domyślny numer kierunkowy kraju
admin-sms-country-hint = (używany, gdy goście podają numer lokalny)
admin-sms-daily-cap = Dzienny limit
admin-sms-daily-cap-hint = (wiadomości dziennie dla całej instancji, 0 oznacza brak limitu)
admin-sms-daily-cap-help = Po przekroczeniu limitu calrs przestaje wysyłać SMS-y i nadal wysyła e-maile, więc żadna rezerwacja nie zawiedzie z powodu wyczerpanego budżetu na SMS-y.
admin-save-sms = Zapisz ustawienia SMS
admin-send-test-sms = Wyślij wiadomość testową na
admin-send-test-sms-hint-check = (zostaw puste, aby tylko sprawdzić dane logowania)
admin-send-test-sms-hint-e164 = (format E.164)
admin-test-gateway = Przetestuj bramkę
admin-sms-clear-confirm = Usunąć konfigurację SMS zapisaną w bazie danych?
admin-sms-allow-all = Pozwól każdemu użytkownikowi włączać SMS-y w swoich typach wydarzeń
admin-sms-allow-all-help = Domyślnie wyłączone: SMS-y zużywają środki z konta skonfigurowanego tutaj, więc tylko administratorzy mogą przełączyć typ wydarzenia w tryb SMS.
admin-save-policy = Zapisz zasadę
admin-page-of = Strona %1 z %2
admin-show-more-js = Pokaż o %1 więcej
admin-show-fewer = Pokaż mniej

# Admin panel: strings carrying markup or literal braces (templates/admin.html)

admin-delete-user-confirm = Trwale usunąć użytkownika { $email }?{"\u000A"}{"\u000A"}Usunie to jego konto, profil planowania, źródła kalendarzy, typy wydarzeń oraz wszystkie dane należące wyłącznie do niego. Przeszłe rezerwacje zostaną usunięte razem z jego typami wydarzeń.{"\u000A"}{"\u000A"}W przypadku użytkowników OIDC/SSO: jeśli automatyczna rejestracja jest włączona, ta osoba zostanie utworzona ponownie przy następnym logowaniu.{"\u000A"}{"\u000A"}Tej operacji nie można cofnąć.
admin-system-settings-help = Publiczny adres i ustawienia bezpieczeństwa sieci. Można je też ustawić zmiennymi środowiskowymi <code>CALRS_BASE_URL</code> i <code>CALRS_ALLOW_PRIVATE_HOSTS</code>. Gdy zmienna środowiskowa jest ustawiona, <strong>ma pierwszeństwo</strong> przed wartością poniżej.
admin-set-by-env = — ustawione przez środowisko ({ $var }), ma pierwszeństwo przed zapisaną wartością
admin-google-help = Aby włączyć integrację z Kalendarzem Google, utwórz dane OAuth2 w <a href="https://console.cloud.google.com/apis/credentials" target="_blank" style="color: var(--accent);">Google Cloud Console</a>. Włącz <strong>CalDAV API</strong> (odczyt zajętości i zapis rezerwacji) oraz <strong>Google Calendar API</strong> (linki Google Meet), a następnie dodaj { $redirect_uri } jako dozwolony adres przekierowania.
admin-room-pattern-help = Dostępne znaczniki: <code>{"{"}username{"}"}</code> (gospodarz), <code>{"{"}event{"}"}</code> (identyfikator typu wydarzenia), <code>{"{"}date{"}"}</code> (RRRRMMDD), <code>{"{"}random{"}"}</code> (8 znaków). Domyślnie: { $default }.
admin-room-pattern-warning = Bez <code>{"{"}random{"}"}</code> nazwa pokoju jest przewidywalna: dwoje gości rezerwujących ten sam typ wydarzenia tego samego dnia trafi do jednego pokoju i zobaczy nawzajem swoje spotkania. Stałe pokoje są dozwolone (na przykład jeden pokój osobisty na gospodarza), ale włączaj to tylko wtedy, gdy rozumiesz konsekwencje.
admin-meeting-webhook-help = Gdy miejscem typu wydarzenia jest „Webhook (własny dostawca)”, calrs wysyła przy potwierdzeniu dane rezerwacji metodą POST na ten adres i oczekuje w odpowiedzi treści JSON <code>{"{"}"url": "https://..."{"}"}</code>.
admin-auth-hmac-help = Przy HMAC calrs wysyła <code>X-Calrs-Signature: sha256=&lt;hex&gt;</code> policzone z surowej treści żądania.
admin-tls-none-warning = Wybieraj <strong>Brak</strong> tylko dla przekaźnika na tej maszynie, który nie obsługuje STARTTLS albo ma certyfikat podpisany samodzielnie. Poczta, a wraz z nią wszelkie dane logowania, pójdzie wtedy otwartym tekstem.
admin-smtp-env-error-help = Popraw zmienne środowiskowe <code>CALRS_SMTP_*</code> albo usuń je, aby zarządzać SMTP z bazy danych tutaj.
admin-smtp-env-managed = Zarządzane przez <strong>zmienne środowiskowe</strong> (mają pierwszeństwo przed bazą danych). Zmień zmienne <code>CALRS_SMTP_*</code> albo usuń je, aby zarządzać SMTP stąd.
admin-smtp-env-help = Możesz też skonfigurować to zmiennymi środowiskowymi (mają pierwszeństwo przed tym): <code>CALRS_SMTP_HOST</code>, <code>CALRS_SMTP_PORT</code>, <code>CALRS_SMTP_TLS_MODE</code> (<code>starttls</code>, <code>tls</code> lub <code>none</code>), <code>CALRS_SMTP_USERNAME</code>, <code>CALRS_SMTP_PASSWORD</code>, <code>CALRS_SMTP_FROM_EMAIL</code>, <code>CALRS_SMTP_FROM_NAME</code>. Obowiązkowe są tylko <code>CALRS_SMTP_HOST</code> i <code>CALRS_SMTP_FROM_EMAIL</code>; pomiń nazwę użytkownika i hasło, aby przekazywać pocztę przez lokalny MTA bez uwierzytelniania.
admin-sms-env-error-help = Popraw zmienne środowiskowe <code>CALRS_SMS_*</code> albo usuń je, aby zarządzać SMS-ami z bazy danych tutaj.
admin-sms-env-managed = Zarządzane przez <strong>zmienne środowiskowe</strong> (mają pierwszeństwo przed bazą danych). Zmień zmienne <code>CALRS_SMS_*</code> albo usuń je, aby zarządzać SMS-ami stąd.
admin-sms-env-help = Możesz też skonfigurować to zmiennymi środowiskowymi (mają pierwszeństwo przed tym): <code>CALRS_SMS_PROVIDER</code>, <code>CALRS_SMS_API_KEY</code>, <code>CALRS_SMS_API_SECRET</code>, <code>CALRS_SMS_SENDER</code>, <code>CALRS_SMS_BASE_URL</code>, <code>CALRS_SMS_DAILY_CAP</code>, <code>CALRS_SMS_DEFAULT_COUNTRY_CODE</code>.
admin-sms-trial-warning = <strong>Tryb próbny Twilio jest włączony</strong> (<code>CALRS_SMS_TWILIO_TRIAL</code>). Goście dostają predefiniowany szablon Twilio <code>sms_appointment_reminders</code> zamiast prawdziwej wiadomości, a dotrzeć można tylko na numery zweryfikowane w twojej konsoli Twilio. To pomoc przy testowaniu na kontach próbnych. Usuń tę zmienną, zanim zaczniesz przyjmować rezerwacje.

admin-show-more =
    { $count ->
        [one] Pokaż o { $count } więcej
        [few] Pokaż o { $count } więcej
        [many] Pokaż o { $count } więcej
       *[other] Pokaż o { $count } więcej
    }

# Calendar source form: backend picker (templates/source_form.html)

source-form-backend-help = Wybierz protokół, którym mówi twój serwer. EWS dotyczy Exchange 2019/2016/2013 we własnej serwerowni.

admin-sms-going-live = <strong>Zanim ruszysz produkcyjnie:</strong> ogranicz kraje docelowe w swojej bramce (w Twilio nazywa się to Geo Permissions), trzymaj konto przedpłacone bez automatycznego doładowania i zostaw włączoną captchę. Te trzy rzeczy razem ograniczają koszt ewentualnej próby SMS pumpingu.

troubleshoot-heading = Diagnostyka dostępności

# Host-side form validation errors (src/web/mod.rs)

form-error-team-name-slug-required = Nazwa i identyfikator są wymagane.
form-error-team-name-length = Nazwa może mieć najwyżej 255 znaków.
form-error-team-description-length = Opis może mieć najwyżej 5000 znaków.
form-error-slug-charset = Identyfikator może zawierać tylko małe litery, cyfry i myślniki.
form-error-slug-reserved = Ten identyfikator jest zarezerwowany. Proszę wybrać inny.
form-error-team-slug-taken = Zespół o tym identyfikatorze już istnieje.
form-error-title-required = Do wygenerowania identyfikatora potrzebny jest tytuł.
form-error-event-type-slug-taken = Typ wydarzenia o tym identyfikatorze już istnieje.
form-error-event-type-slug-taken-team = W tym zespole istnieje już typ wydarzenia o tym identyfikatorze.
form-error-event-type-title-required = Tytuł jest wymagany.
form-error-event-type-title-length = Tytuł może mieć najwyżej 255 znaków.
form-error-event-type-duration = Czas trwania musi być liczbą całkowitą minut od 1 do 1440 (24 godziny).
form-error-event-type-number = Bufory, minimalne wyprzedzenie, przypomnienie i liczba dodatkowych gości muszą być liczbami całkowitymi, 0 lub więcej.
form-error-event-type-horizon = Horyzont rezerwacji musi być liczbą całkowitą dni, 0 lub więcej. Zostaw puste pole, aby nie ustawiać limitu.
form-error-location-required = Szczegóły miejsca są wymagane (na przykład link do rozmowy wideo, numer telefonu albo adres).
form-error-bookable-range-order = Data końcowa musi być taka sama jak data początkowa lub późniejsza.
form-error-bookable-range-invalid = Wpisz daty w formacie RRRR-MM-DD.
form-error-not-team-admin = Nie jesteś administratorem tego zespołu.
form-error-no-account = Nie znaleziono profilu planowania. Proszę skontaktować się z administracją.
form-error-all-fields-required = Wszystkie pola są wymagane.
form-error-encryption = Błąd szyfrowania.
form-error-connection-failed = Połączenie nie powiodło się: { $error }. Sprawdź adres i dane logowania albo zaznacz „Pomiń test połączenia”, aby mimo to zapisać.

# Settings page flash (src/web/mod.rs)

settings-saved = Zapisano ustawienia.

# Profile settings validation and flash messages (src/web/mod.rs)

settings-error-name-length = Imię i nazwisko muszą mieć od 1 do 255 znaków.
settings-error-username-length = Nazwa użytkownika musi mieć co najmniej 2 znaki.
settings-error-username-taken = Ta nazwa użytkownika jest już zajęta.
settings-error-booking-email = Proszę podać prawidłowy adres e-mail do rezerwacji.
settings-error-save-failed = Nie udało się zapisać ustawień.

# Host-facing error responses (src/web/mod.rs)

error-team-not-found-or-not-admin = Nie znaleziono zespołu albo nie jesteś jego administratorem.
error-team-not-found = Nie znaleziono zespołu.
error-event-type-not-found = Nie znaleziono typu wydarzenia.
error-decrypt-failed = Nie udało się odszyfrować zapisanych danych logowania.
error-source-not-found = Nie znaleziono źródła.
error-source-no-password = To źródło nie ma zapisanego hasła.
error-oauth-invalid-state = Nieprawidłowy parametr stanu. Proszę spróbować ponownie.
error-oauth-no-code = Nie otrzymano kodu autoryzacji.
error-oauth-not-configured = Google OAuth2 nie jest skonfigurowane.
error-no-scheduling-account = Nie znaleziono profilu planowania.
error-private-event-type-not-found = Nie znaleziono prywatnego typu wydarzenia.
error-access-denied = Odmowa dostępu.

# Guest booking-flow errors (src/web/mod.rs)

error-slot-unavailable = Ten termin nie jest już dostępny.
error-slot-too-soon = Ten termin nie jest już dostępny (zbyt blisko).
error-slot-beyond-horizon = Ten termin wykracza poza okno rezerwacji.
error-slot-not-yet-bookable = Ten termin nie jest jeszcze dostępny do rezerwacji.
error-invite-required = Ten typ wydarzenia wymaga linku z zaproszeniem.
error-invite-invalid = Nieprawidłowy link z zaproszeniem.
error-invite-expired = Ten link z zaproszeniem wygasł.
error-invite-used = Ten link z zaproszeniem został już wykorzystany.
error-invalid-date = Nieprawidłowa data.
error-invalid-time = Nieprawidłowa godzina.
error-invalid-date-format = Nieprawidłowy format daty.
error-invalid-time-format = Nieprawidłowy format godziny.
error-too-many-bookings = Zbyt wiele prób rezerwacji. Proszę spróbować ponownie za kilka minut.
error-too-many-requests = Zbyt wiele żądań. Proszę spróbować ponownie później.
error-no-members-available = Żaden członek zespołu nie jest dostępny w tym terminie.
error-dynamic-group-public-only = Dynamiczne linki grupowe są dostępne tylko dla publicznych typów wydarzeń.
error-user-not-found = Nie znaleziono użytkownika.

# Booking action error page: titles (templates/booking_action_error.html)

bae-title-captcha = Weryfikacja captcha nie powiodła się
bae-title-invalid-booking = Nieprawidłowe dane rezerwacji
bae-title-unavailable = Obecnie niedostępne
bae-title-cannot-approve = Nie można zatwierdzić tej rezerwacji
bae-title-invalid-link = Nieprawidłowy link
bae-title-invalid-or-expired = Nieprawidłowy lub wygasły link
bae-title-booking-not-found = Nie znaleziono rezerwacji
bae-title-already-approved = Już zatwierdzona
bae-title-already-declined = Już odrzucona
bae-title-already-cancelled = Już odwołana
bae-title-booking-cancelled = Rezerwacja odwołana
bae-title-booking-declined = Rezerwacja odrzucona

# Booking action error page: bodies

bae-body-go-back = Proszę wrócić i spróbować ponownie.
bae-body-unavailable = Gospodarz nie przyjmuje już rezerwacji na ten dzień. Proszę wybrać inną datę albo zajrzeć później.
bae-body-resource-gone = Wymagany zasób nie jest już dostępny o tej porze. Poproś gościa o wybranie innego terminu.
bae-body-no-claim-token = Nie podano tokenu.
bae-body-claim-invalid = Ten link nie jest już ważny.
bae-body-booking-gone = Ta rezerwacja już nie istnieje.
bae-body-decline-link-invalid = Ten link odrzucenia jest nieprawidłowy, wygasł albo rezerwacja została już obsłużona.
bae-body-cancel-link-invalid = Ten link odwołania jest nieprawidłowy, wygasł albo rezerwacja została już odwołana.
bae-body-cancel-link-invalid-short = Ten link odwołania jest nieprawidłowy albo wygasł.
bae-body-reschedule-link-invalid = Ten link przeniesienia jest nieprawidłowy, wygasł albo rezerwacja została już obsłużona.
bae-body-approval-link-invalid = Ten link zatwierdzenia jest nieprawidłowy albo wygasł.
bae-body-already-approved = Ta rezerwacja została już zatwierdzona.
bae-body-already-declined = Ta rezerwacja została już odrzucona.
bae-body-already-cancelled = Ta rezerwacja została już odwołana.
bae-body-was-cancelled = Ta rezerwacja została odwołana.
bae-body-declined-by-host = Ta rezerwacja została odrzucona przez gospodarza.

# Booking form validation (src/web/mod.rs)

validate-name-length = Imię i nazwisko muszą mieć od 1 do 255 znaków.
validate-email-length = Adres e-mail musi mieć od 1 do 255 znaków.
validate-email-invalid = Proszę podać prawidłowy adres e-mail.
validate-notes-length = Notatki nie mogą przekraczać 5000 znaków.
validate-date-too-far = Nie można rezerwować z wyprzedzeniem większym niż rok.

# Additional guests and dynamic group links (src/web/mod.rs)

guests-not-allowed = Ten typ wydarzenia nie dopuszcza dodatkowych gości.
guests-too-many =
    { $max ->
        [one] Można dodać najwyżej jednego dodatkowego gościa.
        [few] Można dodać najwyżej { $max } dodatkowych gości.
        [many] Można dodać najwyżej { $max } dodatkowych gości.
       *[other] Można dodać najwyżej { $max } dodatkowych gości.
    }
guests-invalid-email = Nieprawidłowy adres e-mail dodatkowego gościa: { $email }
dynamic-group-min-usernames = Linki grupy dynamicznej wymagają co najmniej dwóch nazw użytkowników.
dynamic-group-user-not-found = Nie znaleziono użytkownika „{ $username }”.
dynamic-group-user-opted-out = Użytkownik „{ $username }” nie włączył linków grupy dynamicznej.

error-slot-unavailable-member = Ten termin nie jest już dostępny ({ $username } ma konflikt).

# Pending requests: slot hold, expiry and approval re-check (#234)

sms-expired = Prośba wygasła: { $event }, { $date } o { $time } ({ $tz }). Brak odpowiedzi, wybierz inny termin.
email-pending-expired-status = Prośba wygasła
email-pending-expired-headline = Twoja prośba o rezerwację wygasła: gospodarz nie odpowiedział na czas. Wybierz inny termin.
email-pending-expired-action = Wybierz inny termin
bae-body-host-busy = Gospodarz (lub wymagany członek zespołu) nie jest już wolny o tej porze. Poproś gościa o wybranie innego terminu.
bookings-error-generic = Coś poszło nie tak.
bookings-error-no-bookable-dates = Ten typ wydarzenia nie ma już dostępnych dat, więc gość nie mógłby wybrać nowego terminu. Najpierw wydłuż jego zakres dat.
troubleshoot-detail-pending-request = { $title } (oczekująca prośba)
etf-pending-blocks-slot = Oczekujące prośby blokują termin
etf-pending-blocks-slot-help = Dopóki prośba czeka na twoją odpowiedź, nikt inny nie może zarezerwować tego czasu, ani w tym, ani w innych twoich typach wydarzeń.
etf-pending-expiry = Wygasanie próśb
etf-pending-expiry-help = Prośba, na którą do tego czasu nie odpowiesz, zostaje automatycznie odrzucona, a gość zostaje poproszony o wybranie innego terminu. Termin kończy się wcześniej, gdy spotkanie jest zbyt blisko, by ktoś inny mógł jeszcze zarezerwować ten czas. W połowie czasu dostajesz jedno przypomnienie.
etf-pending-expiry-never = Nigdy

etf-pending-expiry-hours =
    { $hours ->
        [one] Po { $hours } godzinie
        [few] Po { $hours } godzinach
        [many] Po { $hours } godzinach
       *[other] Po { $hours } godziny
    }

# Host, watcher and remaining guest emails (src/email.rs, #209)

email-footer-sent-by = Wysłane przez
email-sender-via = { $host } przez { $instance }
email-date-at-time = { $date } o { $time }
email-detail-phone = Telefon:
email-detail-expires = Wygasa:
email-detail-new-date = Nowa data:
email-detail-new-time = Nowa godzina:
email-detail-previous = Poprzednio:
email-detail-requested = Proponowany termin:
email-detail-originally = Pierwotnie:
email-detail-organizer = Organizator:
email-detail-booked-by = Zarezerwowane przez:
email-detail-invited-by = Zaproszenie od:
email-detail-message = Wiadomość:
email-action-approve = Zatwierdź
email-action-decline = Odrzuć
email-action-pick-new-time = Wybierz nowy termin
email-host-approve-line = Zatwierdź: { $url }
email-host-decline-line = Odrzuć: { $url }
email-host-approval-dashboard = Zaloguj się do panelu, aby potwierdzić lub odrzucić tę rezerwację.
email-host-new-subject = Nowa rezerwacja: { $event }, { $guest } ({ $date })
email-host-new-heading = Nowa rezerwacja!
email-host-new-message = { $guest } zarezerwował(a) u Ciebie termin.
email-host-confirmed-subject = Potwierdzono: { $event }, { $guest } ({ $date })
email-host-confirmed-heading = Rezerwacja potwierdzona
email-host-confirmed-message = Zatwierdziłeś tę rezerwację ({ $guest }).
email-host-confirmed-footer = Wydarzenie zostało dodane do Twojego kalendarza.
email-host-sync-failure-subject = Wymagane działanie: kalendarz nie został zaktualizowany dla { $event } ({ $date })
email-host-sync-failure-heading = Twój kalendarz nie został zaktualizowany
email-host-sync-failure-intro = Termin tej rezerwacji został zmieniony, ale calrs nie mógł zaktualizować wydarzenia w Twoim Kalendarzu Google. Gość ma nowy termin, a Twój kalendarz wciąż pokazuje stary.
email-host-sync-failure-action = Otwórz wydarzenie w Kalendarzu Google i przenieś je na powyższy termin. Przenieś istniejące wydarzenie zamiast tworzyć je od nowa, inaczej link Google Meet, który gość już ma, przestanie działać.
email-writeback-failure-subject = Wymagane działanie: { $event } ({ $date }) nie ma w twoim kalendarzu
email-writeback-failure-heading = W twoim kalendarzu brakuje rezerwacji
email-writeback-failure-intro = calrs nie udało się dodać tej rezerwacji do twojego kalendarza, nawet po kilku próbach. Rezerwacja jest potwierdzona, a gość ma zaproszenie.
email-writeback-failure-action = Dodaj wydarzenie do kalendarza samodzielnie, żeby go nie przegapić, i sprawdź źródło kalendarza w calrs. calrs będzie próbować jeszcze przez kilka godzin; tego e-maila dla tej rezerwacji już nie dostaniesz.
email-detail-error = Błąd:
email-source-failure-subject = Wymagane działanie: źródło kalendarza „{ $source }” się nie synchronizuje
email-source-failure-heading = Źródło kalendarza przestało się synchronizować
email-source-failure-intro = calrs od ponad godziny nie może zsynchronizować tego źródła kalendarza. Twoje strony rezerwacji nadal pokazują terminy, ale opierają się na ostatniej udanej synchronizacji, więc wydarzenia dodane lub przeniesione od tego czasu nie są brane pod uwagę.
email-source-failure-action = Sprawdź źródło na stronie Źródła kalendarzy: zwykle przyczyną jest wygasłe hasło lub cofnięty dostęp. O tym źródle usłyszysz ponownie tylko wtedy, gdy wróci do działania i znów przestanie działać.
email-detail-source = Źródło kalendarza:
email-detail-failing-since = Błędy od:
email-host-reminder-subject = Przypomnienie: { $event }, { $guest } ({ $date })
email-host-reminder-heading = Nadchodząca rezerwacja
email-host-reminder-message = Przypomnienie: wkrótce masz rezerwację. Gość: { $guest }.
email-host-cancel-subject = Anulowano: { $event }, { $guest } ({ $date })
email-host-cancel-message-by-host = Anulowałeś tę rezerwację.
email-host-cancel-message-by-guest = { $guest } anulował(a) swoją rezerwację.
email-host-approval-subject = Wymagane działanie: { $event }, { $guest } ({ $date })
email-host-approval-heading = Wymagane działanie
email-host-approval-message = { $guest } chce zarezerwować u Ciebie termin.
email-host-approval-reminder-subject = Przypomnienie: { $guest } czeka na Twoją odpowiedź ({ $event }, { $date })
email-host-approval-reminder-heading = Przypomnienie: wymagane działanie
email-host-approval-reminder-message = { $guest } wciąż czeka na Twoją odpowiedź. Jeśli nie odpowiesz, prośba zostanie automatycznie odrzucona (termin: { $deadline }).
email-host-approval-footer = Możesz tym też zarządzać w swoim panelu.
email-host-expired-subject = Wygasła: { $event }, { $guest } ({ $date })
email-host-expired-heading = Prośba o rezerwację wygasła
email-host-expired-message = Nikt nie odpowiedział na prośbę ({ $guest }) przed terminem, więc została automatycznie odrzucona, a gość został poproszony o wybranie innego terminu.
email-host-expired-footer = Czas oczekiwania próśb możesz zmienić w ustawieniach typu wydarzenia.
email-host-reschedule-request-subject = Prośba o zmianę terminu: { $event }, { $guest } ({ $email })
email-host-reschedule-request-message = { $guest } chce zmienić termin swojej rezerwacji.
email-watcher-claim-subject = Do przejęcia: { $event }, { $guest } ({ $date })
email-watcher-claim-message = Nowa rezerwacja jest dostępna do przejęcia. Kliknij poniżej, aby dołączyć jako uczestnik.
email-watcher-claim-link = Przejmij tę rezerwację: { $url }
email-watcher-claim-footer = Możesz ją też przejąć w swoim panelu.
email-claim-confirmed-subject = Rezerwacja przejęta: { $event }, { $guest } ({ $date })
email-claim-confirmed-footer = Zostaniesz dodany jako uczestnik tego spotkania.
email-attendee-greeting = Cześć,
email-attendee-message = Dodano Cię jako uczestnika rezerwacji.
email-attendee-rescheduled-message = Termin rezerwacji, w której uczestniczysz, został zmieniony.
email-attendee-cancelled-message = Rezerwacja, w której uczestniczysz, została anulowana.
email-attendee-cancelled-subject = Anulowano: { $event }
email-original-booking-cancelled = Twoja pierwotna rezerwacja również została anulowana.
email-pick-new-time-subject = Zmiana terminu: { $event }, wybierz nowy termin
email-pick-new-time-message = { $host } musi zmienić termin Twojej rezerwacji. Wybierz nowy termin.
email-pick-new-time-link = Wybierz nowy termin: { $url }
email-rescheduled-message = { $host } zmienił(a) termin Twojej rezerwacji.
email-reschedule-approved-message = { $host } potwierdził(a) nowy termin Twojej rezerwacji.
email-rescheduled-ics-attached-plain = Zaktualizowane zaproszenie do kalendarza w załączniku.
email-rescheduled-ics-attached-html = Zaktualizowane zaproszenie do kalendarza w załączniku tego e-maila.
email-need-to-reschedule = Chcesz zmienić termin? { $url }
email-invite-subject = { $host } zaprasza Cię do rezerwacji: { $event }
email-invite-message = { $host } zaprasza Cię do rezerwacji: { $event }
email-invite-link = Kliknij poniższy link, aby wybrać termin:
email-invite-action = Wybierz termin
email-cancel-reason-calendar-deleted = Gospodarz usunął wydarzenie z kalendarza.

# Embed snippet generator (templates/embed.html)
embed-page-title = Osadzanie — { $title }
embed-heading = Osadzanie: { $title }
embed-subtitle = Wygeneruj fragment kodu, aby osadzić ten typ wydarzenia na swojej stronie internetowej.
embed-warn-internal = Uwaga: ten typ wydarzenia jest <strong>wewnętrzny</strong>. Odwiedzający osadzoną stronę będą potrzebować ważnego linku z zaproszeniem, aby zarezerwować — wygenerowany kod go nie zawiera. Rozważ zmianę widoczności na publiczną, aby osadzić go bezpośrednio.
embed-warn-private = Uwaga: ten typ wydarzenia jest <strong>prywatny</strong>. Odwiedzający osadzoną stronę będą potrzebować ważnego linku z zaproszeniem, aby zarezerwować — wygenerowany kod go nie zawiera. Rozważ zmianę widoczności na publiczną, aby osadzić go bezpośrednio.
embed-warn-private-team = Uwaga: ten zespół jest <strong>prywatny</strong>. Odwiedzający osadzoną stronę nie mogą zarezerwować bez linku z zaproszeniem do zespołu — wygenerowany kod go nie zawiera. Rozważ zmianę zespołu na publiczny, aby osadzić go bezpośrednio.
embed-need-username = Musisz ustawić <strong>nazwę użytkownika</strong>, zanim osadzisz ten typ wydarzenia — link do osadzenia to { $path }. { $link } i wróć tutaj.
embed-need-username-link = Ustaw nazwę użytkownika w ustawieniach
embed-username-placeholder = nazwa-uzytkownika
embed-tab-inline = W treści strony
embed-tab-floating = Pływający przycisk
embed-tab-element = Kliknięcie elementu
embed-layout = Układ
embed-theme = Motyw
embed-theme-auto = Automatyczny (jak u odwiedzającego)
embed-brand-color = Kolor marki
embed-use-org-color = Użyj koloru organizacji
embed-button-text = Tekst przycisku
embed-button-default = Umów spotkanie
embed-button-position = Pozycja przycisku
embed-pos-bottom-right = Na dole po prawej
embed-pos-bottom-left = Na dole po lewej
embed-pos-top-right = Na górze po prawej
embed-pos-top-left = Na górze po lewej
embed-button-colors = Kolory przycisku
embed-button-fg-placeholder = tekst
embed-button-colors-hint = Tło / tekst
embed-show-icon = Pokaż ikonę kalendarza
embed-element-help = Wklej kod raz na swojej stronie, a następnie dodaj pokazane atrybuty data do każdego klikalnego elementu (przycisk, link, div), który ma otwierać okno rezerwacji.
embed-snippet = Fragment kodu
embed-loading = Ładowanie…
embed-preview = Podgląd na żywo
embed-preview-back = wróć do podglądu
embed-preview-floating-hint = Kliknij przycisk, aby zasymulować okno rezerwacji
embed-preview-element-hint = Przykładowe wyzwalacze — na swojej stronie dodaj atrybuty data z kodu do dowolnego elementu. Kliknij którykolwiek, aby wypróbować okno:
embed-preview-text-link = lub kliknij ten link tekstowy
embed-snippet-attr-comment = Dodaj te atrybuty do dowolnego klikalnego elementu na swojej stronie:

# Leftovers in partly translated pages (host reschedule, impersonation banner, slots, team settings)
host-resched-when = { $date }, { $start } – { $end }
base-impersonating = Podszywasz się pod { $name }
slots-rescheduling-banner = Zmiana terminu: { $title } (obecnie { $date }, { $time })
slots-invite-required-title = Rezerwacja wymaga linku z zaproszeniem
team-settings-slug-url = Używane w adresie URL zespołu: { $url }
team-event-type-no-teams = Nie ma jeszcze żadnych zespołów.
team-event-type-not-team-admin = Nie jesteś administratorem żadnego zespołu.

# Google Calendar connection errors (src/web/sources.rs)
google-error-not-configured = Integracja z Kalendarzem Google nie jest skonfigurowana. Poproś administratora o ustawienie danych OAuth2 Google w panelu administracyjnym.
google-error-no-base-url = Publiczny bazowy adres URL nie jest skonfigurowany. Ustaw go przez zmienną środowiskową CALRS_BASE_URL lub w panelu administracyjnym w sekcji Ustawienia systemowe, zanim użyjesz procesów OAuth2.
google-error-authorization = Autoryzacja Google nie powiodła się: { $error }
google-error-token-exchange = Wymiana tokenu nie powiodła się: { $error }
google-error-account-email = Nie udało się pobrać adresu e-mail konta Google: { $error }

# Admin panel notices (src/web/admin.rs)
admin-resource-feed-invalid = Nieprawidłowy adres URL kanału: { $error }
admin-resource-feed-check-failed = Sprawdzenie kanału nie powiodło się: { $error }
admin-resource-caldav-invalid = Nieprawidłowy adres URL CalDAV: { $error }
admin-resource-credentials-failed = Nie udało się zapisać danych logowania: { $error }
admin-resource-added = Dodano zasób „{ $name }”. Wydarzenia w pamięci podręcznej: { $count }.
admin-resource-updated = Zaktualizowano zasób. Wydarzenia w pamięci podręcznej: { $count }.
admin-resource-deleted = Usunięto zasób.
admin-resource-not-found = Nie znaleziono zasobu.
admin-resource-synced = Zsynchronizowano kanał. Wydarzenia w pamięci podręcznej: { $count }.
admin-resource-sync-error = Synchronizacja nie powiodła się: { $error }
admin-resource-no-caldav-url = Nie skonfigurowano adresu URL CalDAV i nie udało się go wyprowadzić z adresu URL kanału.
admin-resource-no-write-credentials = Brak danych do zapisu (brak konta usługi, a żaden członek, który wyraził zgodę, nie ma pasującego źródła kalendarza).
admin-resource-write-passed = Test zapisu zakończony powodzeniem dla { $user } (utworzono: { $created }, usunięto: { $cleaned }).
admin-resource-write-verified = potwierdzono
admin-resource-write-unverified = niepotwierdzono
admin-resource-write-check-calendar = sprawdź kalendarz zasobu
admin-resource-write-failed = Test zapisu nie powiódł się. Ostatni błąd: { $error }
admin-base-url-invalid = Bazowy adres URL musi być bezwzględnym adresem http(s), np. https://cal.example.com
admin-company-link-invalid = Link firmowy musi zaczynać się od http:// lub https://
admin-smtp-host-required = Host SMTP jest wymagany.
admin-smtp-from-invalid = Podaj prawidłowy adres e-mail nadawcy.
admin-smtp-port-invalid = Port SMTP musi być liczbą od 1 do 65535.
admin-smtp-tls-invalid = Tryb TLS musi mieć wartość „starttls”, „tls” lub „none”.
admin-smtp-password-required = Przy pierwszej konfiguracji SMTP wymagane jest hasło.
admin-sms-switch-needs-secret = Zmiana bramki SMS wymaga podania danych uwierzytelniających tej bramki.
admin-sms-settings-invalid = Ustawienia SMS: { $error }.
admin-sms-sender-format = Numeryczny nadawca SMS musi być w formacie międzynarodowym, np. +48512345678.
admin-sms-not-configured = SMS nie jest skonfigurowany.
admin-sms-load-failed = Nie udało się wczytać konfiguracji SMS.
admin-sms-check-accepted = Bramka zaakceptowała dane uwierzytelniające.
admin-sms-bad-number = Ten numer telefonu nie nadaje się do użycia.
admin-sms-test-body = Wiadomość testowa calrs. Twoja bramka SMS działa.
admin-sms-test-accepted = Bramka przyjęła wiadomość testową.

## Local multi-factor authentication

mfa-title = Uwierzytelnianie dwuskładnikowe
mfa-invalid = Weryfikacja nie powiodła się. Sprawdź hasło lub kod. Każdego kodu możesz użyć tylko raz; w razie potrzeby poczekaj na nowy. Po zbyt wielu próbach odczekaj 15 minut. Jeśli konfiguracja wygasła, zacznij od nowa.
mfa-restart = Wróć do logowania
mfa-manage = Zarządzaj uwierzytelnianiem dwuskładnikowym
mfa-recovery-help = Zapisz te kody odzyskiwania w bezpiecznym miejscu. Są wyświetlane tylko raz. Każdy kod jednorazowo zastępuje kod z aplikacji. Nowe kody zastępują wszystkie poprzednie.
mfa-continue = Przejdź do panelu
mfa-enabled = Uwierzytelnianie dwuskładnikowe jest włączone.
mfa-disabled = Uwierzytelnianie dwuskładnikowe jest wyłączone.
mfa-required-help = Twój administrator wymaga uwierzytelniania dwuskładnikowego dla kont lokalnych. Nie możesz go wyłączyć.
mfa-code = Kod z aplikacji lub kod odzyskiwania
mfa-regenerate = Wygeneruj nowe kody odzyskiwania
mfa-disable = Wyłącz uwierzytelnianie dwuskładnikowe
mfa-enable = Skonfiguruj uwierzytelnianie dwuskładnikowe
mfa-changes-help = Zmiany wymagają hasła. Wyłączenie uwierzytelniania lub wymiana kodów wymaga też kodu i wylogowuje pozostałe sesje.
mfa-back = Wróć do ustawień
mfa-setup-help = Zeskanuj ten kod QR aplikacją uwierzytelniającą lub wpisz poniższy klucz ręcznie. Wpisz sześciocyfrowy kod, aby zakończyć konfigurację. Konfiguracja wygasa po 10 minutach.
mfa-qr-alt = Kod QR do konfiguracji aplikacji
mfa-login-help = Wpisz kod z aplikacji uwierzytelniającej lub jeden z zapisanych kodów odzyskiwania. To logowanie wygasa po 10 minutach.
mfa-totp-code = Sześciocyfrowy kod z aplikacji
mfa-verify = Zweryfikuj
mfa-policy-help = Ta zasada dotyczy kont lokalnych. SSO korzysta z MFA dostawcy tożsamości. Lokalni administratorzy muszą skonfigurować MFA przed zmianą tej zasady. Wymaganie MFA wylogowuje lokalne sesje bez weryfikacji MFA i wymusza konfigurację przy następnym logowaniu.
mfa-policy = Zasada MFA dla kont lokalnych
mfa-optional = Opcjonalne
mfa-required = Wymagane
mfa-save-policy = Zapisz zasadę MFA

# First day of the week (#330)
week-start-label = Pierwszy dzień tygodnia
week-start-inherit-site = Użyj ustawienia witryny
week-start-inherit-owner = Użyj ustawienia właściciela

## Booking directory and personal favorites

directory-title = Katalog spotkań
directory-help = Znajdź link do spotkania współpracownika lub zespołu. Dodaj często udostępniane spotkania do ulubionych, aby mieć je na pulpicie.
directory-search = Szukaj według osoby, zespołu lub tytułu spotkania
directory-find = Szukaj
directory-favorites-only = Tylko ulubione
directory-favorites = Ulubione spotkania
directory-browse = Przeglądaj katalog
directory-generate = Wygeneruj link zaproszenia
directory-invite-help = Zaproszenie jednorazowe, ważne przez 7 dni.
directory-add = Dodaj { $title } do ulubionych
directory-remove = Usuń { $title } z ulubionych
directory-added = Dodano do ulubionych.
directory-removed = Usunięto z ulubionych.
directory-error = Nie udało się wykonać tej czynności. Odśwież stronę i spróbuj ponownie.
directory-manual-copy = Skopiuj ten link ręcznie:
directory-empty = Nie znaleziono spotkań. Spróbuj innego wyszukiwania.
directory-favorites-empty = Nie znaleziono ulubionych. Dodaj spotkania do ulubionych w katalogu, aby zobaczyć je tutaj.
directory-with-hosts = Prowadzący: { $names }

## ilia fork: TON payments (docs/ton.md)

# Price on the slot and booking pages
ton-slots-price = Cena: { $usd } $
ton-slots-price-approx = (≈ { $ton } TON)
ton-slots-price-note = płatność w TON po wybraniu terminu

# Event type form
ton-etf-section = Płatność w TON
ton-etf-off-note = Płatności w TON nie są skonfigurowane na tym serwerze. Płatnego typu wydarzenia nie da się zarezerwować, dopóki nie zostaną skonfigurowane.
ton-etf-enabled = Płatna rezerwacja
ton-etf-enabled-help = Goście płacą w TON zaraz po wybraniu terminu. Termin jest dla nich zarezerwowany w trakcie płatności, a nieopłacona rezerwacja zostaje automatycznie zwolniona.
ton-etf-price = Cena
ton-etf-price-help = W USD: od 1 $ do 10000 $; kwota w TON jest liczona według kursu z chwili rezerwacji.
ton-etf-window = Czas na płatność (minuty)
ton-etf-window-help = Od 10 do 240. Zostaw puste, aby użyć wartości serwera ({ $minutes } min).
ton-form-error-price = Podaj cenę w dolarach amerykańskich od 1 do 10000, z najwyżej dwoma miejscami po przecinku.
ton-form-error-window = Czas na płatność musi wynosić od 10 do 240 minut.
ton-form-error-team = Typy wydarzeń zespołu nie mogą być płatne.

# Booking error pages
ton-bae-title-unavailable = Nie można zarezerwować tego terminu
ton-bae-body-unavailable = Płatność jest chwilowo niedostępna, więc tego terminu nie można teraz zarezerwować. Spróbuj ponownie za kilka minut.
ton-bae-body-too-late = Do tego spotkania zostało za mało czasu, aby je opłacić. Wybierz późniejszy termin.
ton-bae-body-already-open = Masz już nieopłaconą rezerwację na ten adres e-mail. Możemy wysłać Ci jej linki: tam możesz ją opłacić albo anulować, aby zwolnić termin, a potem zarezerwować ponownie.
ton-bae-body-personal-only = To spotkanie można zarezerwować tylko na własnej stronie gospodarza.
ton-bae-title-unpaid = Rezerwacja nieopłacona
ton-bae-body-unpaid = Ta rezerwacja nie jest opłacona, więc nie można jej zatwierdzić. Aby potwierdzić ją bezpłatnie, użyj opcji „Potwierdź bez płatności” na stronie rezerwacji w panelu.
ton-bae-body-reschedule-unpaid = Ta rezerwacja nie jest opłacona, więc nie można jej przenieść. Opłać ją albo anuluj i zarezerwuj inny termin.
ton-bae-body-pay-link-invalid = Ten link do płatności jest nieprawidłowy.

# Payment page
ton-pay-page-title = Płatność: { $title }
ton-pay-held-until = Termin jest dla ciebie zarezerwowany do { $time } ({ $zone }).
ton-pay-left = zostało %1
ton-pay-amount-label = Kwota do zapłaty
ton-pay-rate = ≈ { $usd } $ · 1 TON = { $rate } $ ({ $sources }, { $time } UTC)
ton-pay-open-wallet = Inny portfel
ton-pay-open-tonkeeper = Otwórz w Tonkeeper
ton-pay-qr-alt = Kod QR z danymi płatności
ton-pay-scan = Albo zeskanuj ten kod aplikacją portfela.
ton-pay-qr-alt-tonkeeper = Kod QR dla Tonkeepera z danymi płatności
ton-pay-scan-tonkeeper = Albo zeskanuj ten kod Tonkeeperem lub aparatem telefonu.
ton-pay-qr-other = Kod QR dla innego portfela (np. MyTonWallet)
ton-pay-qr-tonkeeper = Kod QR dla Tonkeepera
ton-pay-address = Adres
ton-pay-comment = Komentarz
ton-pay-copy = Kopiuj
ton-pay-copied = Skopiowano
ton-pay-warn-comment = Komentarz jest wymagany: po nim rozpoznajemy twoją płatność.
ton-pay-warn-encrypt = Nie szyfruj komentarza.
ton-pay-warn-token = Płać w TON, nie tokenami.
ton-pay-warn-exchange = Płacisz z giełdy? Wpisz powyższy kod w pole memo (komentarz).
ton-pay-warn-deadline = Wyślij płatność przed { $time } ({ $zone }). Płatność wysłana później może nie zostać uwzględniona.
ton-pay-check = Zapłacono, sprawdź
ton-pay-email = Wyślij mi ten link e-mailem
ton-pay-emailed = Wysłaliśmy ten link na adres e-mail podany przy rezerwacji.
ton-pay-email-unavailable = Nie udało się wysłać linku. Nie zamykaj tej strony albo zapisz jej adres.
ton-pay-checking-now = Sprawdzamy twoją płatność. Ta strona odświeża się sama.
ton-pay-state-waiting = Czekamy na twoją płatność.
ton-pay-state-seen = Twoja płatność jest widoczna w sieci. Czekamy, aż stanie się ostateczna, co zwykle trwa kilka sekund.
ton-pay-state-partial = Otrzymaliśmy { $paid } TON. Dopłać pozostałe { $rest } TON z tym samym komentarzem.
ton-pay-state-checking = Czas na płatność minął. Jeśli płatność została już wysłana, sprawdzamy ją. Ta strona odświeża się sama.
ton-pay-state-paid = Płatność otrzymana. Potwierdzamy rezerwację…
ton-pay-state-confirmed = Opłacono. Rezerwacja jest potwierdzona, e-mail z potwierdzeniem jest w drodze.
ton-pay-state-waived = Ta rezerwacja nie wymaga płatności. Gospodarz ją potwierdza.
ton-pay-state-busy = Płatność otrzymana, ale ten termin nie jest już wolny. Skontaktujemy się z tobą.
ton-pay-state-expired = Czas na płatność minął i termin został zwolniony. Jeśli mimo to wysłano płatność, napisz do nas z adresu e-mail podanego przy rezerwacji, a zwrócimy pieniądze.
ton-pay-state-late = Twoja płatność dotarła po terminie, więc rezerwacja nie została potwierdzona. Skontaktujemy się w sprawie zwrotu. Możesz też napisać do nas z adresu e-mail podanego przy rezerwacji.
ton-pay-state-closed = Ta rezerwacja nie jest już aktywna.
ton-pay-state-revoked = Ten adres płatności nie jest już używany. Nie wysyłaj na niego niczego i zarezerwuj ponownie.
ton-pay-rebook = Zarezerwuj inny termin

# Leaving the embedded widget
ton-exit-title = Termin zarezerwowany
ton-exit-body = Termin jest dla ciebie zarezerwowany do { $time } ({ $zone }). Dokończ płatność, aby potwierdzić rezerwację.
ton-exit-go = Przejdź do płatności

# Dashboard
ton-dash-badge-awaiting = czeka na płatność · { $amount } TON · do { $time }
ton-dash-badge-partial = opłacona częściowo · { $paid } z { $amount } TON · do { $time }
ton-dash-badge-paid = opłacona · { $amount } TON
ton-dash-badge-overpaid = opłacona · nadpłata { $excess } TON
ton-dash-badge-busy = opłacona, termin zajęty
ton-dash-badge-confirm-failed = opłacona, niepotwierdzona · potwierdź ręcznie
ton-dash-badge-refund = opłacono { $amount } TON · należy się zwrot
ton-dash-badge-waived = potwierdzona bez płatności
ton-dash-badge-unpaid = nieopłacona
ton-dash-waive = Potwierdź bez płatności
ton-dash-error-unpaid = Ta rezerwacja nie jest opłacona, więc nie można jej potwierdzić ani przenieść. Użyj opcji „Potwierdź bez płatności”, aby potwierdzić ją bezpłatnie, albo ją odrzuć.
ton-dash-error-waive-failed = Nie udało się potwierdzić rezerwacji bez płatności: nie czeka już na płatność.
ton-waive-page-title = Potwierdź bez płatności
ton-waive-heading = Potwierdzić tę rezerwację bez płatności?
ton-waive-body = Ta rezerwacja NIE jest opłacona ({ $amount } TON, termin płatności do { $time }). Jeśli ją potwierdzisz, gość dostanie spotkanie bezpłatnie, a płatność, która dotrze później, trzeba będzie zwrócić ręcznie.
ton-waive-submit = Potwierdź bezpłatnie
ton-waive-back = Wróć do rezerwacji

# Guest emails
ton-email-link-subject = Dokończ rezerwację: { $event }
ton-email-link-message = Rozpoczęto rezerwację: { $event }, gospodarz: { $host }. Dokończ ją na stronie poniżej, zanim minie czas.
ton-email-link-action = Dokończ rezerwację
ton-email-link-line = Dokończ rezerwację: { $url }
ton-email-expired-subject = Bez rezerwacji: { $event }, { $date }
ton-email-expired-message = Płatność za { $event } nie dotarła na czas, więc termin został zwolniony. Możesz wybrać inny termin.
ton-email-expired-partial = Otrzymaliśmy część płatności. Skontaktujemy się w sprawie zwrotu.
ton-email-rebook-line = Wybierz inny termin: { $url }

# Owner emails
ton-email-row-amount = Kwota:
ton-email-row-paid = Otrzymano:
ton-email-row-excess = Nadpłata:
ton-email-row-transfer = Przelew:
ton-email-row-sender = Nadawca:
ton-email-row-tx = Transakcja:
ton-email-row-detail = Szczegóły:
ton-email-amount-value = { $amount } TON ≈ { $usd } $ (1 TON = { $rate } $)
ton-email-sender-wallet = { $address } (portfel)
ton-email-sender-other = { $address } (nie portfel: giełda lub kontrakt)
ton-email-dashboard-line = Rezerwacje: { $url }
ton-email-dashboard-action = Otwórz rezerwacje
ton-email-refund-rule = Zwroty robisz ręcznie ze swojego portfela: na adres nadawcy, jeśli to portfel; w przeciwnym razie tylko na adres, o który gość poprosi z adresu e-mail podanego przy rezerwacji, w wiadomości zawierającej jego link do płatności lub anulowania. Kod rezerwacji, kwota i transakcja są publiczne i niczego nie dowodzą.
ton-email-received-subject = Płatność otrzymana: { $event }, { $guest } ({ $date })
ton-email-received-heading = Płatność otrzymana
ton-email-received-message = Zapłacono { $amount } TON. Gość: { $guest }. Rezerwacja jest potwierdzona.
ton-email-alert-subject = Płatność w TON wymaga uwagi: { $what }
ton-email-alert-heading = Płatność wymaga twojej uwagi
ton-alert-partial = płatność częściowa
ton-alert-partial-body = Dotarła część należnej kwoty. Rezerwacja pozostaje niepotwierdzona, dopóki nie dotrze reszta; w przeciwnym razie zostanie zwolniona w terminie, a otrzymaną kwotę trzeba będzie zwrócić.
ton-alert-overpaid = nadpłata
ton-alert-overpaid-body = Dotarła płatność za już opłaconą rezerwację. Zwróć ją, jeśli warto.
ton-alert-late = spóźniona płatność
ton-alert-late-body = Płatność wpłynęła po terminie. Rezerwacja była już zwolniona i nie można jej już potwierdzić. Zwróć płatność albo poproś gościa o ponowną rezerwację i potwierdź nową rezerwację bez płatności.
ton-alert-unmatched = nierozpoznana płatność
ton-alert-unmatched-body = Dotarła płatność bez znanego kodu rezerwacji. Znajdź rezerwację, do której należy, albo ją zwróć.
ton-alert-failed = nieudany przelew
ton-alert-failed-body = Przelew na adres płatności nie powiódł się w sieci. Środki mogą wciąż być w twoim portfelu: sprawdź transakcję w eksploratorze.
ton-alert-not-text = przelew bez czytelnego komentarza
ton-alert-not-text-body = Dotarł przelew z zaszyfrowanym komentarzem, tokenem lub inną zawartością. Nigdy nie jest zaliczany automatycznie: sprawdź go ręcznie.
ton-alert-other = nietypowy przelew
ton-alert-other-body = Przelewu nie udało się przetworzyć automatycznie. Sprawdź go w rejestrze płatności.
ton-email-problem-subject = Problem z płatnością w TON: { $event } ({ $date })
ton-email-problem-urgent-subject = Pilne: opłacona rezerwacja wymaga decyzji: { $event } ({ $date })
ton-email-problem-heading = Opłacona rezerwacja wymaga twojej uwagi
ton-problem-host-busy = Gość zapłacił, ale termin nie jest już wolny (wydarzenie w kalendarzu lub inna rezerwacja). Termin pozostaje zarezerwowany do rozpoczęcia spotkania. Poproś gościa o nowy termin, potwierdź rezerwację ręcznie albo ją odrzuć i zwróć płatność.
ton-problem-partial = Opłacono tylko część kwoty, a potem rezerwacja wygasła, została anulowana lub odrzucona albo potwierdzona bez płatności. Zwróć otrzymaną kwotę, chyba że ustalono z gościem inaczej.
ton-problem-overpaid = Rezerwacja jest opłacona i potwierdzona, ale wpłacono więcej, niż należało. Zwróć nadwyżkę, jeśli warto.
ton-problem-refund-due = Rezerwacja została opłacona, a potem anulowana lub odrzucona. Zwróć płatność.
ton-problem-confirmed-unpaid = Nieopłacona rezerwacja okazała się potwierdzona. To nie powinno się zdarzyć: sprawdź, jak została potwierdzona.
ton-problem-address-revoked = Adres płatności tej rezerwacji został unieważniony, a rezerwacja zostanie zwolniona.
ton-email-health-subject = Płatności w TON wymagają uwagi ({ $kind })
ton-email-health-heading = Płatności w TON wymagają uwagi
ton-health-indexer-down = Monitor płatności nie może połączyć się z indekserem. Płatności nie są sprawdzane, a nowe płatne rezerwacje są odrzucane, dopóki połączenie nie wróci.
ton-health-auth-failed = Indekser odrzucił klucz API (błędny lub dla innej sieci). Sprawdź CALRS_TON_API_KEY.
ton-health-wrong-network = Indekser odpowiada dla innej sieci niż skonfigurowana. Do czasu poprawki nic nie jest zaliczane ani nie wygasa.
ton-health-wallet-inactive = Portfel odbiorczy nie jest aktywny. Płatne rezerwacje są odrzucane, dopóki nie wykona co najmniej jednego przelewu wychodzącego.
ton-health-lagging = Monitor płatności nie nadąża za przelewami na adres (być może spam). Nieopłacone rezerwacje nie wygasają, dopóki nie nadrobi zaległości.
ton-health-stuck-finality = Jeden przelew od dawna nie stał się ostateczny. Monitor czeka na niego, zanim przeczyta dalej.
ton-health-watcher-crashed = Monitor płatności kilka razy uległ awarii i wciąż się restartuje. Sprawdź dziennik serwera.
ton-health-disabled-with-open = Płatności w TON są wyłączone, a nieopłacone rezerwacje są otwarte. Ich terminy zostaną zwolnione w terminie.
ton-health-other-network-open = Są otwarte rezerwacje w innej sieci TON niż skonfigurowana. Zostaną zwolnione w terminie.
ton-health-rate-unavailable = Brak akceptowalnego kursu TON (źródła niedostępne, rozbieżne lub poza twoim przedziałem). Płatne rezerwacje są odrzucane, dopóki kurs się nie pojawi.
ton-health-capacity = Osiągnięto limit otwartych lub dziennych nieopłaconych rezerwacji. Nowe płatne rezerwacje są na razie odrzucane.
ton-health-other = Monitor płatności zgłosił problem. Sprawdź dziennik serwera.

# TON follow-ups: releases, refunds, contact, resend
ton-resend-submit = Wyślij mi linki
ton-resend-title = Sprawdź pocztę
ton-resend-done = Jeśli na ten adres e-mail jest nieopłacona rezerwacja, wysłaliśmy na niego jej linki do płatności i anulowania.
ton-pay-state-taken = Ten termin został zajęty, zanim rezerwacja została opłacona, więc rezerwację zwolniono. Nie trzeba nic płacić. Wybierz inny termin.
ton-pay-state-refund = Ta rezerwacja zakończyła się bez potwierdzenia. Zapłacone { $paid } TON zostaną zwrócone; organizator skontaktuje się z Tobą.
ton-pay-contact = Pytania? Napisz do organizatora ({ $host }): { $email }.
ton-pay-copy-address = Kopiuj adres
ton-pay-copy-comment = Kopiuj komentarz
ton-pay-copy-manual = Nie udało się skopiować. Tekst jest zaznaczony: skopiuj go ręcznie.
ton-email-row-pay-by = Zapłać do:
ton-email-taken-message = Termin wybrany na { $event } został zajęty, zanim rezerwacja została opłacona, więc rezerwację zwolniono. Nie trzeba nic płacić. Możesz wybrać inny termin.
ton-email-refund-message = Otrzymaliśmy Twoją płatność za { $event }, ale nie udało się potwierdzić terminu, więc rezerwację zwolniono. { $host } zwróci płatność i skontaktuje się z Tobą.
ton-problem-host-busy-unpaid = Termin tej nieopłaconej rezerwacji został zajęty (zatwierdzono na nim inną prośbę albo w kalendarzu pojawiło się wydarzenie), więc rezerwację zwolniono przed płatnością. Jeśli coś już wpłynęło, zwróć to.
ton-problem-confirm-failed = Gość zapłacił, ale rezerwacji nie udało się automatycznie potwierdzić po kilku próbach (to błąd, a nie zajęty organizator: zobacz dziennik serwera). Termin pozostaje zarezerwowany do początku spotkania. Potwierdź rezerwację ręcznie albo ją odrzuć i zwróć płatność.
ton-health-capacity-near = Nowa płatna rezerwacja zajęła ostatnie wolne miejsce lub ostatni dzienny limit. Kolejne będą odrzucane, dopóki rezerwacje nie zostaną opłacone lub zwolnione. Jeśli to nie jest prawdziwy popyt, ktoś może blokować terminy bez płacenia.
ton-health-payment-alerts-held = Wpłynęły kolejne przelewy do rezerwacji, o których w ciągu ostatniej godziny otrzymano już e-mail. Zapisano je bez osobnych e-maili: sprawdź rejestr płatności.
ton-waive-body-partial = Ta rezerwacja jest opłacona tylko częściowo (otrzymano { $paid } z { $amount } TON, termin do { $time }). Jeśli ją potwierdzisz, gość otrzyma spotkanie bez dopłaty pozostałych { $rest } TON. Zwróć otrzymaną kwotę, chyba że ustalono inaczej.

## Telegram (ilia fork)

tg-when = { $date }, { $time } ({ $tz })
tg-in-minutes =
    { $count ->
        [one] za { $count } minutę
        [few] za { $count } minuty
        [many] za { $count } minut
       *[other] za { $count } minuty
    }
tg-in-hours =
    { $count ->
        [one] za { $count } godzinę
        [few] za { $count } godziny
        [many] za { $count } godzin
       *[other] za { $count } godziny
    }
tg-in-days =
    { $count ->
        [one] za { $count } dzień
        [few] za { $count } dni
        [many] za { $count } dni
       *[other] za { $count } dnia
    }
tg-reason = Powód: { $reason }
tg-link-line = { $label }: { $url }
tg-guest-confirmed = Rezerwacja potwierdzona: { $event }, { $when }. Gospodarz: { $host }.
tg-guest-pending = Prośba wysłana: { $event }, { $when }. Odpowiedź gospodarza otrzymasz tutaj.
tg-guest-approved = Twoja prośba została potwierdzona: { $event }, { $when }.
tg-guest-reschedule-approved = Nowy termin potwierdzony: { $event }, { $when }.
tg-guest-declined = Twoja prośba została odrzucona: { $event }, { $when }.
tg-guest-cancelled-by-host = Gospodarz odwołał spotkanie: { $event }, { $when }.
tg-guest-cancelled-by-guest = Anulowałeś rezerwację: { $event }, { $when }.
tg-guest-rescheduled = Rezerwacja przeniesiona: { $event } odbędzie się { $when }.
tg-guest-reschedule-pending = Poproszono o nowy termin: { $event }, { $when }. Odpowiedź gospodarza otrzymasz tutaj.
tg-guest-reschedule-requested = Gospodarz prosi o wybranie nowego terminu dla { $event }. Było: { $when }.
tg-guest-reminder = Przypomnienie: { $event } zaczyna się { $in }, { $when }.
tg-guest-expired = Prośba wygasła: { $event }, { $when }. Gospodarz nie odpowiedział na czas; zarezerwuj inny termin.
tg-button-cancel = Anuluj rezerwację
tg-button-reschedule = Zmień termin
tg-button-calendar = Dodaj do kalendarza
tg-button-pick-time = Wybierz nowy termin
tg-button-rebook = Zarezerwuj inny termin
tg-button-bookings = Otwórz rezerwacje
tg-host-new-booking = Nowa rezerwacja: { $event }, { $when }. Gość: { $guest }, { $email }.
tg-host-approval-request = Prośba do zatwierdzenia: { $event }, { $when }. Gość: { $guest }, { $email }.
tg-host-reschedule-approval = Nowy termin do zatwierdzenia: { $event }, { $when }. Gość: { $guest }, { $email }.
tg-host-rescheduled = Przeniesienie: { $event }, teraz { $when }. Gość: { $guest }.
tg-host-cancelled = Anulowanie: { $event }, { $when }. Gość: { $guest }.
tg-host-guest-linked = { $guest } będzie otrzymywać powiadomienia w Telegramie o { $event }, { $when }: { $username }.
tg-host-guest-linked-differs = Podano w formularzu: { $entered }.
tg-host-no-username = bez nazwy użytkownika
tg-bot-help = Ten bot wysyła powiadomienia o rezerwacjach w { $site }. Aby je włączyć, otwórz link ze strony potwierdzenia lub z e-maila; przy płatnej rezerwacji link jest na jej stronie płatności. /stop je wyłącza. Gdy rezerwacja jest połączona z tym czatem, możesz tu też napisać do jej gospodarza.
tg-bot-linked-guest = Gotowe: powiadomienia o tej rezerwacji będą przychodzić tutaj.
tg-bot-linked-host = Gotowe: powiadomienia gospodarza dla { $name } będą przychodzić tutaj.
tg-bot-link-invalid = Ten link już nie działa: działa tylko w jednym czacie Telegrama i tylko do końca spotkania.
tg-bot-link-replaced = Ten link został zastąpiony. Otwórz ponownie stronę płatności rezerwacji i użyj znajdującego się tam linku do Telegrama.
tg-bot-rate-limited = Zbyt wiele prób. Spróbuj ponownie później.
tg-bot-stopped = Powiadomienia są wyłączone. Aby znów je włączyć dla rezerwacji, otwórz ponownie jej link z e-maila z potwierdzeniem; gospodarze tworzą nowy link w Profil i ustawienia → Telegram.
tg-bot-not-linked = Ten czat nie otrzymuje powiadomień.
tg-cmd-help = Jak działa ten bot
tg-cmd-stop = Wyłącz powiadomienia
tg-book-label = Telegram
tg-book-placeholder = @nazwa
tg-book-help-optional = Po rezerwacji możesz włączyć powiadomienia w Telegramie.
tg-book-help-required = Po rezerwacji otwórz link do Telegrama, aby otrzymywać powiadomienia.
tg-book-invalid-title = Sprawdź nazwę użytkownika Telegrama
tg-book-invalid = Podaj nazwę użytkownika Telegrama w postaci @nazwa (litery, cyfry i _, od 4 do 32 znaków).
tg-book-required = Podaj swoją nazwę użytkownika Telegrama.
tg-confirmed-title = Powiadomienia w Telegramie
tg-confirmed-body = Otwórz bota i naciśnij przycisk na dole czatu: tam będą przychodzić powiadomienia o tej rezerwacji.
tg-confirmed-button = Otwórz Telegram
tg-email-button = Powiadomienia w Telegramie
tg-email-plain = Powiadomienia w Telegramie: { $url }
tg-settings-nav = Powiadomienia w Telegramie
tg-settings-title = Telegram
tg-settings-intro = Otrzymuj powiadomienia gospodarza we własnym czacie Telegrama: nowe rezerwacje, prośby do zatwierdzenia, anulowania i zmiany terminu przez gości.
tg-settings-off = Telegram nie jest skonfigurowany na tym serwerze.
tg-settings-no-bot-name = Bot nie jest jeszcze gotowy. Spróbuj ponownie za minutę.
tg-settings-linked = Powiadomienia gospodarza trafiają do { $username }. Połączono: { $date }.
tg-settings-not-linked = Twój Telegram nie jest połączony.
tg-settings-link = Połącz mój Telegram
tg-settings-link-open = Otwórz Telegram
tg-settings-link-expires = W czacie naciśnij przycisk na dole. Link działa raz w ciągu 15 minut.
tg-settings-link-limited = Zbyt wiele linków w tej godzinie. Spróbuj ponownie później.
tg-settings-unlink = Odłącz
tg-settings-modes = Telegram gościa według typu spotkania
tg-settings-modes-help = Opcjonalnie lub wymagane: formularz pyta o Telegram gościa, a potwierdzenie podaje link do bota.
tg-settings-modes-none = Brak typów spotkań do ustawienia.
tg-settings-mode-off = Wyłączone
tg-settings-mode-optional = Opcjonalnie
tg-settings-mode-required = Wymagane
tg-settings-save = Zapisz
tg-settings-saved = Zapisano.
tg-settings-team = Zespół { $team }
tg-dashboard-label = Telegram { $username }
tg-dashboard-entered = (podano { $entered })
tg-dashboard-not-linked = (jeszcze nie połączono)
tg-bot-already-linked = Już włączone: powiadomienia o tej rezerwacji przychodzą na ten czat.
tg-bot-link-invalid-host = Ten link wygasł: linki z ustawień działają raz w ciągu 15 minut. Utwórz nowy w Profil i ustawienia → Telegram.
tg-settings-linked-no-username = Powiadomienia gospodarza trafiają na twój czat w Telegramie. Połączono: { $date }.
tg-bot-description = Ten bot wysyła powiadomienia o twoich rezerwacjach w { $site }: potwierdzenia, zmiany i przypomnienia. Naciśnij przycisk poniżej, aby włączyć je dla swojej rezerwacji. /stop je wyłącza.
tg-bot-short-description = Powiadomienia o rezerwacjach z { $site }.
tg-button-join = Dołącz

## Telegram: TON payments (ilia fork; docs/ton.md)

tg-guest-payment-due = Twoja rezerwacja czeka na płatność: { $event }, { $when }. Kwota: { $amount } TON (≈ { $usd } $). Termin płatności: { $deadline }. Jeśli płatność nie dotrze, termin zostanie zwolniony.
tg-guest-payment-partial = Dotarła część płatności za { $event }, { $when }: { $paid } z { $amount } TON. Dopłać pozostałe { $rest } TON. Termin płatności: { $deadline }. Jeśli płatność nie dotrze, termin zostanie zwolniony.
tg-guest-payment-received = Płatność otrzymana, Twoja rezerwacja jest potwierdzona: { $event }, { $when }. Gospodarz: { $host }.
tg-guest-payment-late = Płatność { $value } TON za { $event }, { $when } dotarła po terminie, więc nie potwierdziła rezerwacji. { $host } skontaktuje się z Tobą w sprawie zwrotu.
tg-guest-payment-expired = Czas na zapłatę za { $event }, { $when } minął, więc termin został zwolniony. Możesz zarezerwować inny termin.
tg-guest-payment-taken = Termin wybrany przez Ciebie na { $event }, { $when } został zajęty, zanim rezerwacja została opłacona, więc rezerwację zwolniono. Nie musisz nic płacić. Możesz zarezerwować inny termin.
tg-guest-payment-refund = Otrzymaliśmy Twoją płatność za { $event }, { $when }, ale nie udało się potwierdzić terminu, więc rezerwację zwolniono. { $host } zwróci płatność i skontaktuje się z Tobą.
tg-guest-payment-refund-part = Część płatności już dotarła: { $host } skontaktuje się z Tobą w sprawie zwrotu.
tg-guest-payment-held = Otrzymaliśmy Twoją płatność za { $event }, { $when }, ale nie udało się automatycznie potwierdzić rezerwacji. { $host } skontaktuje się z Tobą.
tg-button-pay = Zapłać
tg-host-payment-received = Opłacono: { $event }, { $when }. Gość: { $guest }, { $email }. Otrzymano { $paid } TON (≈ { $usd } $), rezerwacja jest potwierdzona.
tg-host-payment-problem = Płatność wymaga uwagi: { $event }, { $when }. Gość: { $guest }, { $email }.
tg-host-payment-transfer = Przelew: { $value } TON.
tg-host-payment-amounts = Otrzymano { $paid } z { $amount } TON.
tg-host-payment-guest-telegram = Telegram gościa: { $username }.

## ilia fork: booking flow v2 (docs/ton.md, docs/operations.md)
troubleshoot-label-busy = Zajęty
ton-etf-currency = Waluta
ton-etf-currency-ton = TON
ton-etf-currency-usd = USD (przeliczane na TON)
ton-etf-price-help-ton = W TON: od 1 do 100000, gość płaci dokładnie tę kwotę (100 TON, jeśli pole jest puste).
ton-etf-usd-no-rates = Na tym serwerze nie skonfigurowano kursów (CALRS_TON_RATE_MIN_USD i CALRS_TON_RATE_MAX_USD), więc ceny w USD nie da się zarezerwować. Podaj cenę w TON.
ton-form-error-price-ton = Podaj cenę w TON od 1 do 100000, z najwyżej dwoma miejscami po przecinku.
ton-form-error-currency = Wybierz walutę ceny: TON lub USD.
ton-form-error-telegram = Dla Telegrama gościa wybierz: wyłączone, opcjonalne lub wymagane.
ton-etf-telegram = Telegram gościa
ton-etf-telegram-help = Przy płatnej rezerwacji domyślnie wymagane: gość dostaje wtedy przypomnienia o płatności i można się z nim skontaktować przez Telegram. Możesz ustawić je jako opcjonalne lub je wyłączyć.
ton-slots-price-ton = Cena: { $ton } TON
ton-email-amount-value-ton = { $amount } TON
tg-guest-payment-due-ton = Twoja rezerwacja czeka na płatność: { $event }, { $when }. Kwota: { $amount } TON. Termin płatności: { $deadline }. Jeśli płatność nie dotrze, termin zostanie zwolniony.
tg-host-payment-received-ton = Opłacono: { $event }, { $when }. Gość: { $guest }, { $email }. Otrzymano { $paid } TON, rezerwacja jest potwierdzona.
ton-pay-renewed = Termin został przedłużony ({ $n } z { $max }).
ton-dash-badge-renewed = termin przedłużony { $n } z { $max }
ton-email-renewed-subject = Twój termin jest nadal zarezerwowany: { $event }
ton-email-renewed-message = Nie otrzymaliśmy jeszcze płatności za: { $event }, gospodarz: { $host }, więc termin pozostaje zarezerwowany do { $deadline }. Jeśli już zapłaciłeś, nic nie musisz robić: płatność zostanie zaliczona. W przeciwnym razie zapłać do tego czasu na stronie poniżej albo anuluj rezerwację, jeśli plany się zmieniły.
tg-guest-payment-renewed = Płatność za: { $event }, { $when } jeszcze nie dotarła, więc termin pozostaje dla ciebie zarezerwowany do { $deadline }. Zapłać pozostałe { $rest } TON do tego czasu, inaczej termin zostanie zwolniony. Jeśli już zapłaciłeś, nic nie musisz robić: płatność zostanie zaliczona.

## ilia fork: messages with guests (docs/operations.md)
msg-page-title = Wiadomości
msg-back = Rezerwacje
msg-guest = Gość: { $guest }
msg-sent = Wiadomość wysłana.
msg-empty = Nie ma jeszcze wiadomości.
msg-label = Wiadomość do gościa
msg-help-email = Gość otrzyma ją e-mailem z adresu systemu; jego odpowiedzi pojawią się tutaj i w Twoim Telegramie.
msg-help-no-email = Na tym serwerze nie skonfigurowano poczty e-mail.
msg-help-telegram = Gość połączył Telegram i otrzyma ją także tam.
msg-help-no-telegram = Gość nie połączył Telegrama.
msg-help-limits = Najwyżej 20 wiadomości dziennie na rezerwację, w tym 5 e-mailem.
msg-send = Wyślij
msg-link = Wiadomości
msg-link-count = Wiadomości ({ $count })
msg-from-host = Gospodarz
msg-from-guest = Gość
msg-from-attendee = Uczestnik: { $address }
msg-from-other = Inny adres, nie gościa: { $address }
msg-via-dashboard = panel
msg-via-mcp = agent
msg-via-telegram = Telegram
msg-status-email-queued = e-mail w kolejce
msg-status-email-sent = e-mail wysłany
msg-status-email-failed = e-mail niedostarczony
msg-status-telegram-queued = Telegram w kolejce
msg-status-telegram-sent = wysłano w Telegramie
msg-status-telegram-failed = niedostarczono w Telegramie
msg-error-empty = Najpierw napisz wiadomość.
msg-error-too-long = Wiadomość może mieć najwyżej 2000 znaków.
msg-error-limit = Osiągnięto dzienny limit wiadomości dla tej rezerwacji. Spróbuj jutro.
msg-error-no-channel = Nie można skontaktować się z gościem: na tym serwerze nie skonfigurowano poczty e-mail, a gość nie połączył Telegrama.
msg-error-closed = Wiadomości można wysyłać tylko do oczekujących lub potwierdzonych rezerwacji, do tygodnia po spotkaniu.
msg-email-subject = Wiadomość od { $host }: { $event }, { $date }
msg-email-intro = Wiadomość od { $host } w sprawie { $event }:
msg-email-footer = Odpowiedź na tego e-maila trafi do: { $host }.
tg-guest-host-message = Wiadomość od { $host } w sprawie { $event }, { $when }:
tg-guest-reply-hint = Aby odpowiedzieć, napisz odpowiedź tutaj.
tg-host-guest-message = Wiadomość od { $guest } ({ $email }) w sprawie { $event }, { $when }:
tg-host-reply-hint = Odpowiedz na tę wiadomość, aby odpisać gościowi.
tg-bot-message-sent = Twoja wiadomość została przekazana gospodarzowi.
tg-bot-message-pick-booking = Z tym czatem połączonych jest kilka rezerwacji. Odpowiedz na wiadomość o rezerwacji, o którą chodzi.
tg-bot-message-closed = Ta rezerwacja nie przyjmuje już wiadomości.
tg-bot-message-after-payment = Do gospodarza można napisać po opłaceniu rezerwacji.
tg-bot-message-limit = Zbyt wiele wiadomości. Spróbuj ponownie później.
tg-bot-reply-sent = Wysłano do gościa.
tg-bot-reply-not-linked = Ten gość nie połączył Telegrama, więc odpowiedź nie została wysłana. Napisz do niego z rezerwacji w panelu.
tg-bot-reply-not-allowed = Odpowiedź nie została wysłana: to nie Twoja rezerwacja albo nie przyjmuje już wiadomości.
msg-error-internal = Nie udało się zapisać wiadomości, więc nic nie zostało wysłane. Spróbuj ponownie.
msg-link-new = Wiadomości (nowe: { $count })
msg-recent-title = Ostatnie spotkania
msg-recent-help = Spotkania, które już się zaczęły lub skończyły w ostatnim tygodniu. Z ich gośćmi wciąż można wymieniać wiadomości.
tg-bot-message-sent-shortened = Twoja wiadomość została przekazana gospodarzowi, skrócona do { $max } znaków.
tg-bot-reply-too-long = Odpowiedź nie została wysłana: wiadomość może mieć najwyżej { $max } znaków.
tg-bot-reply-empty = Odpowiedź nie została wysłana: nie zawiera widocznego tekstu.
tg-bot-reply-failed = Nie udało się zapisać odpowiedzi, więc nie została wysłana. Spróbuj ponownie.
troubleshoot-not-synced = Kalendarze są odczytywane tylko od { $from } do { $until }; wydarzenia spoza tych dni nie są wczytywane, więc nie ma ich tutaj.
event-type-form-horizon-capped =
    { $days ->
        [one] Ten serwer udostępnia terminy najwyżej { $days } dzień naprzód, niezależnie od horyzontu rezerwacji.
        [few] Ten serwer udostępnia terminy najwyżej { $days } dni naprzód, niezależnie od horyzontu rezerwacji.
        [many] Ten serwer udostępnia terminy najwyżej { $days } dni naprzód, niezależnie od horyzontu rezerwacji.
       *[other] Ten serwer udostępnia terminy najwyżej { $days } dni naprzód, niezależnie od horyzontu rezerwacji.
    }

## ilia fork: the system mailbox (src/inbox)
msg-help-email-no-inbox = Gość otrzyma ją e-mailem z adresu systemu. Odpowiedzi e-mailem nie są na tym serwerze czytane.
msg-via-email = e-mail
msg-email-footer-no-reply = Tę wiadomość wysłał system rezerwacji w imieniu: { $host }; odpowiedzi na nią nie są czytane.
tg-host-guest-email = E-mail od { $guest } ({ $email }) w sprawie { $event }, { $when }:
tg-host-attendee-email = E-mail od { $sender }, dodatkowego uczestnika rezerwacji gościa { $guest } ({ $email }), w sprawie { $event }, { $when }:
tg-host-other-email = E-mail z innego adresu, nie gościa: { $sender }, w wątku rezerwacji gościa { $guest } ({ $email }) w sprawie { $event }, { $when }:
booking-blocked-body-system = Gospodarz wymaga co najmniej { $minutes } minut wyprzedzenia. Jeśli nie możesz się pojawić, napisz na <a href="{ $href }">{ $address }</a>.
booking-blocked-body-no-reply = Gospodarz wymaga co najmniej { $minutes } minut wyprzedzenia. Jeśli nie możesz się pojawić, daj znać gospodarzowi w inny sposób.

## ilia fork: passkeys and security keys (docs/passkeys-plan.md)

ilia-passkey-sign-in = Zaloguj się kluczem dostępu
ilia-mfa-login-help-passkey = Potwierdź to logowanie kluczem dostępu lub kluczem bezpieczeństwa. Logowanie wygasa po 10 minutach.
ilia-mfa-use-totp = Użyj kodu z aplikacji
ilia-mfa-use-recovery = Użyj kodu odzyskiwania
ilia-mfa-use-passkey = Użyj klucza dostępu
ilia-mfa-use-passkey-instead = Zamiast tego użyj klucza dostępu lub klucza bezpieczeństwa
ilia-mfa-totp-login-help = Wpisz sześciocyfrowy kod z aplikacji uwierzytelniającej. Logowanie wygasa po 10 minutach.
ilia-mfa-recovery-code = Kod odzyskiwania
ilia-mfa-recovery-login-help = Wpisz jeden z zapisanych kodów odzyskiwania. Każdy kod działa tylko raz. Logowanie wygasa po 10 minutach.
ilia-passkey-unsupported = Ta przeglądarka nie obsługuje kluczy dostępu ani kluczy bezpieczeństwa. Wybierz inną metodę poniżej albo otwórz tę stronę w aktualnej przeglądarce.
ilia-passkey-noscript = Klucze dostępu wymagają JavaScriptu. Włącz go dla tej witryny albo wybierz inną metodę poniżej.
ilia-passkey-cancelled = Żądanie zostało anulowane lub wygasło. Spróbuj ponownie albo wybierz inną metodę.
ilia-passkey-invalid = Nie udało się zweryfikować klucza dostępu lub klucza bezpieczeństwa albo żądanie wygasło. Spróbuj ponownie albo wybierz inną metodę.
ilia-passkey-key-type = Ta przeglądarka nie potrafi odczytać klucza tego typu, więc nie można go tu dodać. Spróbuj innej przeglądarki, na przykład aktualnej wersji Firefoksa lub Chrome, albo wybierz inną metodę.
ilia-passkey-wrong-site = Klucze dostępu działają tylko pod adresem { $origin }. Otwórz tę stronę pod tym adresem i spróbuj ponownie albo wybierz inną metodę.
ilia-mfa-enroll-help = Wybierz, czym będziesz potwierdzać każde logowanie. Najbezpieczniejszy jest klucz dostępu lub klucz bezpieczeństwa: działa tylko w tej witrynie, więc fałszywa kopia strony nie może go użyć.
ilia-passkey-add-security-key = Dodaj klucz bezpieczeństwa
ilia-passkey-add-device = Dodaj klucz dostępu na tym urządzeniu lub telefonie
ilia-passkey-kinds-help = Klucz bezpieczeństwa to klucz USB lub NFC, na przykład YubiKey. Klucz dostępu na tym urządzeniu lub telefonie korzysta z Touch ID, Face ID, Windows Hello lub blokady ekranu telefonu i może być synchronizowany z innymi twoimi urządzeniami.
ilia-mfa-use-app-instead = Zamiast tego użyj aplikacji uwierzytelniającej
ilia-passkey-name = Nazwa klucza
ilia-passkey-name-help = Opcjonalnie, na przykład „YubiKey przy kluczach” albo „MacBook”. Do 64 znaków.
ilia-passkey-default-security-key = Klucz bezpieczeństwa { $n }
ilia-passkey-default-device = Klucz dostępu { $n }
ilia-passkey-register-help = Naciśnij przycisk i postępuj zgodnie z przeglądarką: włóż klucz bezpieczeństwa i dotknij go albo potwierdź przez Touch ID, Face ID, Windows Hello lub telefon.
ilia-passkey-exists = Ten klucz jest już zarejestrowany.
ilia-mfa-recovery-help = Zapisz te kody odzyskiwania w bezpiecznym miejscu; są wyświetlane tylko raz. Każdy kod zastępuje klucz dostępu lub kod z aplikacji przy jednym logowaniu. Nowe kody zastępują wszystkie poprzednie.
ilia-mfa-status-on = Uwierzytelnianie dwuskładnikowe jest włączone.
ilia-mfa-status-off = Uwierzytelnianie dwuskładnikowe jest wyłączone.
ilia-mfa-setup-intro = Dodaj drugi krok do każdego logowania: klucz dostępu lub klucz bezpieczeństwa (YubiKey, Touch ID, Face ID, Windows Hello, telefon) albo aplikację uwierzytelniającą.
ilia-passkey-list = Klucze dostępu i klucze bezpieczeństwa
ilia-passkey-none = Nie ma jeszcze kluczy dostępu ani kluczy bezpieczeństwa.
ilia-passkey-kind-hardware = Klucz sprzętowy
ilia-passkey-kind-synced = Synchronizowany klucz dostępu
ilia-passkey-added = Dodano { $date }
ilia-passkey-last-used = Ostatnio użyty { $date }
ilia-passkey-never-used = Jeszcze nieużywany
ilia-passkey-other-site = Zarejestrowany dla { $rp_id }; nie działa pod tym adresem witryny.
ilia-passkey-rename = Zmień nazwę
ilia-passkey-delete = Usuń
ilia-passkey-add = Dodaj klucz dostępu lub klucz bezpieczeństwa
ilia-passkey-new-name = Nowa nazwa
ilia-passkey-limit = Osiągnięto maksymalną liczbę kluczy ({ $max }). Usuń jeden, aby dodać kolejny.
ilia-passkey-backup-advice = Dodaj drugi klucz i przechowuj go w bezpiecznym miejscu, aby utrata jednego klucza nie odcięła ci dostępu.
ilia-passkey-unavailable = Klucze dostępu są wyłączone: publiczny adres serwera (CALRS_BASE_URL) nie jest adresem https://. Skontaktuj się z administratorem. Dodane wcześniej klucze są na liście i można je usunąć.
ilia-mfa-totp-title = Aplikacja uwierzytelniająca
ilia-mfa-totp-on = Aplikacja uwierzytelniająca jest skonfigurowana.
ilia-mfa-totp-off = Brak skonfigurowanej aplikacji uwierzytelniającej.
ilia-mfa-totp-add = Skonfiguruj aplikację uwierzytelniającą
ilia-mfa-totp-remove = Usuń aplikację uwierzytelniającą
ilia-mfa-recovery-title = Kody odzyskiwania
ilia-mfa-recovery-left =
    { $count ->
        [one] Został { $count } kod odzyskiwania
        [few] Zostały { $count } kody odzyskiwania
        [many] Zostało { $count } kodów odzyskiwania
       *[other] Zostało { $count } kodu odzyskiwania
    }
ilia-mfa-last-factor = Uwierzytelnianie dwuskładnikowe jest obowiązkowe, więc nie można usunąć ostatniej metody logowania. Najpierw dodaj inną.
ilia-mfa-last-factor-off = To twoja ostatnia metoda: jej usunięcie wyłączy uwierzytelnianie dwuskładnikowe i usunie twoje kody odzyskiwania.
ilia-mfa-changes-help = Każda zmiana tutaj wymaga hasła oraz klucza dostępu lub kodu i wylogowuje twoje pozostałe sesje.
ilia-mfa-confirm-title = Potwierdź, że to ty
ilia-mfa-confirm-help = Wpisz hasło, a potem potwierdź kluczem dostępu lub kodem.
ilia-passkey-confirm = Potwierdź kluczem dostępu
ilia-mfa-confirm-code = Potwierdź kodem
ilia-mfa-action-passkey-add = Dodajesz klucz dostępu lub klucz bezpieczeństwa.
ilia-mfa-action-passkey-rename = Zmieniasz nazwę klucza „{ $name }”.
ilia-mfa-action-passkey-delete = Usuwasz klucz „{ $name }”.
ilia-mfa-action-totp-add = Konfigurujesz aplikację uwierzytelniającą.
ilia-mfa-action-totp-remove = Usuwasz aplikację uwierzytelniającą.
ilia-mfa-action-regenerate = Zastępujesz wszystkie kody odzyskiwania.
ilia-mfa-action-policy = Ustawiasz zasadę uwierzytelniania dwuskładnikowego dla kont lokalnych na:
ilia-mfa-policy-continue = Kontynuuj

## ilia fork: the instance name (CALRS_BRAND_NAME) and the error page (src/web/error_page.rs)
ilia-login-subtitle = Zaloguj się na swoje konto { $brand }
ilia-register-subtitle = Zarejestruj nowe konto { $brand }
ilia-error-not-found = Nie znaleziono
ilia-error-not-found-body = Pod tym adresem nic nie ma. Sprawdź link.
ilia-error-gone = Ten link jest już nieważny
ilia-error-bad-request = Nieprawidłowe żądanie
ilia-error-forbidden = Dostęp ograniczony

# ilia fork: TON price by demand (the ladder)
ton-slots-ladder-now = Teraz: { $ton } TON
ton-slots-ladder-now-free = Teraz: za darmo, gospodarz potwierdza prośbę
ton-slots-ladder-locked = Cena zostaje ustalona w chwili rezerwacji.
ton-slots-ladder-summary-free = Za darmo, dopóki kalendarz jest pusty, potem od { $from } do { $to } TON w miarę jego zapełniania.
ton-slots-ladder-summary = Od { $from } do { $to } TON, zależnie od tego, jak pełny jest kalendarz.
ton-slots-ladder-all-free = Za darmo; gospodarz potwierdza każdą prośbę.
ton-slots-ladder-fee = Opłatę sieciową płaci twój portfel.
ton-quote-title = Cena się zmieniła
ton-quote-was = Pokazano ci { $ton } TON.
ton-quote-was-free = Pokazano ci bezpłatną rezerwację.
ton-quote-now = Cena teraz: { $ton } TON.
ton-quote-now-free = Teraz jest za darmo: gospodarz potwierdza prośbę.
ton-quote-why = Od załadowania strony zmienił się kalendarz lub ceny. Nic jeszcze nie zostało zarezerwowane.
ton-quote-why-free-used = Niedawno miałeś już bezpłatną rezerwację, więc ta rezerwacja jest płatna. Nic jeszcze nie zostało zarezerwowane.
ton-quote-confirm-help = Potwierdź cenę. Nic jeszcze nie zostało zarezerwowane.
ton-quote-confirm = Zarezerwuj za { $ton } TON
ton-quote-confirm-free = Poproś za darmo
ton-bae-body-free-used = Niedawno miałeś już bezpłatną rezerwację, więc ta nie może być bezpłatna. Spróbuj ponownie później.
ton-form-error-mode = Wybierz stałą cenę albo cenę zależną od popytu.
ton-etf-mode = Cena
ton-etf-mode-fixed = Stała cena
ton-etf-mode-ladder = Zależna od popytu (drabina cen serwisu, w TON)
ton-etf-ladder-now = Obecna drabina:
ton-etf-ladder-help = Cena to stopień odpowiadający liczbie twoich nadchodzących rezerwacji w najbliższych dniach (potwierdzonych lub opłaconych, bezpłatnych próśb czekających na twoje zatwierdzenie i rezerwacji zespołowych, które prowadzisz; nieopłacone się nie liczą): najpierw za darmo (każdą bezpłatną rezerwację potwierdzasz sam), potem wyżej, w miarę zapełniania się kalendarza. Goście widzą bieżącą cenę i płacą tę, którą im pokazano. Drabinę ustala administrator.

# ilia fork: the TON settings page (/dashboard/admin/ton)
nav-admin-ton = Płatności TON
ton-etf-ladder-edit = Edytuj drabinę
## ilia-ov-ton
ton-admin-debug-tg-saved-locked = Nic nie zmieniono: nazwa użytkownika Telegram gościa jest zablokowana.
dbg-err-requote = Cena zmieniła się dwa razy podczas rezerwacji. Nic nie zarezerwowano: spróbuj ponownie.
ton-admin-page-title = Płatności TON
ton-admin-heading = Płatności TON
ton-admin-intro = Te ustawienia dotyczą płatności TON na tym serwerze. Pierwszeństwo ma wartość ze środowiska serwera, potem zapisana tutaj lub przez config apply, a na końcu domyślna.
ton-admin-saved-live = Zapisano. Nowe ustawienia obowiązują od teraz; otwarte faktury zachowują swoje kwoty i terminy.
ton-admin-saved-off = Zapisano. Płatności są na tym serwerze wyłączone (CALRS_TON_NETWORK nie jest ustawione); ustawienia zaczną obowiązywać po ich włączeniu.
ton-admin-saved-restart = Zapisano. Ustawienia zaczną obowiązywać po ponownym uruchomieniu calrs.
ton-admin-saved-invalid = Zapisano, ale ustawienia serwera jako całość są nieprawidłowe, więc płatności pozostają wyłączone. Zobacz stan poniżej.
ton-admin-saved-none = Nic się nie zmieniło.
ton-admin-errors = Nic nie zapisano: popraw pola oznaczone poniżej.
ton-admin-status = Stan
ton-admin-status-on = Płatności są włączone
ton-admin-status-off = Płatności są wyłączone
ton-admin-network = Sieć
ton-admin-address = Adres portfela
ton-admin-address-raw = Postać surowa
ton-admin-indexer = Indekser
ton-admin-watcher = Obserwator
ton-admin-watcher-ready = gotowy

## ilia-ov3-ton
ton-admin-notready-wallet = portfel odbiorczy nie jest aktywny (lub jeszcze go nie sprawdzono)
ton-admin-notready-network = indekser działa w innej sieci (lub jeszcze go nie sprawdzono)
ton-admin-notready-stale = obserwator nie wykonał żadnego kroku w ciągu ostatnich 2 minut
ton-admin-off-unset = nie ustawiono zmiennej CALRS_TON_NETWORK

ton-admin-watcher-not-ready = niegotowy: { $reason }
ton-admin-last-step = ostatni krok
ton-admin-open-invoices = Otwarte faktury
ton-admin-api-key = Klucz API indeksera
ton-admin-coingecko-key = Klucz CoinGecko
ton-admin-secret-credentials = ustawiony (poświadczenia systemd)
ton-admin-secret-env = ustawiony (środowisko)
ton-admin-not-set = nieustawione
ton-admin-env-only = Sieć, adres portfela, indekser i klucze API ustawia się tylko w środowisku serwera i w poświadczeniach systemd, nigdy na tej stronie.
ton-admin-src-env = ustawione przez środowisko ({ $var })
ton-admin-src-apply = ustawione przez config apply
ton-admin-src-panel = ustawione tutaj
ton-admin-src-default = domyślne
ton-admin-locked-env = Aby zmienić to tutaj, usuń { $var } ze środowiska serwera i uruchom ponownie calrs.
ton-admin-locked-apply = Zmień to w dokumencie config apply albo pomiń w jego sekcji ton, aby przekazać tej stronie.
ton-admin-default-hint = Zostaw puste dla wartości domyślnej: { $value }.
ton-admin-blank-default = Zostaw puste, aby nie ustawiać.
ton-admin-group-pricing = Cena zależna od popytu
ton-admin-group-timing = Terminy faktur
ton-admin-group-rate = Kurs wymiany (ceny w USD)
ton-admin-group-alerts = Powiadomienia
ton-admin-f-ladder = Drabina (TON według liczby rezerwacji)
ton-admin-f-free_per_guest_days = Jedna bezpłatna rezerwacja na gościa w ciągu (dni, 0 = bez limitu)
ton-admin-f-window_min = Czas na płatność (minuty)
ton-admin-f-grace_min = Zapas po czasie na płatność (minuty)
ton-admin-f-fuse_hours = Margines bezpieczeństwa po zapasie (godziny)
ton-admin-f-renew_min_lead_hours = Odnów nieopłacony czas, gdy do spotkania zostało co najmniej tyle (godziny)
ton-admin-f-renew_window_min = Odnowiony czas na płatność (minuty)
ton-admin-f-max_renewals = Odnowienia na fakturę
ton-admin-f-underpay_tolerance_ton = Akceptowana niedopłata (TON)
ton-admin-f-rate_min_usd = Najniższy akceptowany kurs (USD za TON)
ton-admin-f-rate_max_usd = Najwyższy akceptowany kurs (USD za TON)
ton-admin-f-rate_max_spread_pct = Największa różnica między źródłami (%)
ton-admin-f-rate_max_age_min = Najstarszy akceptowany kurs (minuty)
ton-admin-f-rate_max_jump_pct = Największy skok względem ostatniego kursu (%)
ton-admin-f-alert_email = E-mail powiadomień (puste: wszyscy administratorzy)
ton-admin-ladder-help = Szczeble oddzielone przecinkami: cena przy 0, 1, 2… potwierdzonych rezerwacjach w najbliższych dniach; ostatni szczebel to pułap. „free” tylko na początku: bezpłatna rezerwacja czeka na zgodę właściciela.
ton-admin-ladder-row = Rezerwacje: %1
ton-admin-ladder-add = Dodaj szczebel
ton-admin-ladder-remove = Usuń
ton-admin-free = bezpłatnie
ton-admin-preview = Podgląd
ton-admin-preview-button = Podgląd drabiny
ton-admin-previewing = Podgląd drabiny wpisanej powyżej. Nic jeszcze nie zapisano.
ton-admin-ladder-invalid = Obowiązująca drabina jest nieprawidłowa: { $error }
ton-admin-preview-n = Nadchodzące rezerwacje
ton-admin-preview-price = Cena
ton-admin-preview-or-more = { $n } lub więcej
ton-admin-my-demand = Twoje nadchodzące rezerwacje teraz: { $n }. Twoja cena z drabiny teraz: { $price }.
ton-admin-types = Typy wydarzeń z ceną
ton-admin-types-none = Żaden typ wydarzenia nie ma ceny.
ton-admin-type = Typ wydarzenia
ton-admin-type-owner = Właściciel
ton-admin-type-price = Cena
ton-admin-type-now = Teraz
ton-admin-mode-ladder = według popytu
ton-admin-type-off = cena wyłączona
ton-admin-type-below-ceiling = Na pułapie drabiny lub poniżej: cena „w dowolnym czasie” jest zwykle wyższa.
ton-admin-save = Zapisz
ton-admin-audit = Ostatnie zmiany
ton-admin-audit-none = Jeszcze żadnych zmian.
ton-admin-audit-at = Kiedy
ton-admin-audit-who = Kto
ton-admin-audit-key = Ustawienie
ton-admin-audit-change = Zmiana
ton-admin-audit-default = (domyślne)
ton-admin-via-apply = config apply

## ilia fork: the debug booking page (CALRS_DEBUG_BOOKING, src/web/debug_booking.rs)

nav-admin-debug-booking = Rezerwacja testowa
dbg-title = Rezerwacja testowa
dbg-intro = Zarezerwuj testowe spotkanie tak, jak zrobiłby to gość, bez wypełniania za każdym razem formularza gościa. Rezerwacja przechodzi tę samą drogę co publiczna strona rezerwacji: sprawdzenia, cenę i fakturę w TON, blokadę terminu i e-maile. Płatność nigdy nie jest pomijana ani oznaczana jako otrzymana: opłać fakturę ze swojego portfela albo anuluj rezerwację.
dbg-switch-note = Po zakończeniu debugowania wyłącz tę stronę przełącznikiem na górze strony:
dbg-cancelled = Anulowane rezerwacje testowe: { $count }.
dbg-no-types = Nie masz żadnego włączonego osobistego typu wydarzenia.
dbg-event-type = Typ wydarzenia
dbg-priced = płatny
dbg-private-short = prywatny
dbg-choose = Pokaż
dbg-price = Cena
dbg-price-now-free = teraz bezpłatnie (prośbę zatwierdzasz ty)
dbg-price-now-ton = teraz { $ton } TON
dbg-price-ladder = według popytu, ustalana przy rezerwacji
dbg-price-free-used = ten gość miał już niedawno bezpłatną rezerwację, więc obowiązuje pierwsza cena
dbg-free = bezpłatnie
dbg-private = Ten typ wydarzenia nie jest publiczny: dla każdej rezerwacji testowej tworzone jest jednorazowe zaproszenie na twój własny adres.
dbg-ton-off = Płatności TON są na tym serwerze wyłączone, więc płatnego typu wydarzenia nie można teraz zarezerwować.
dbg-captcha-on = Captcha jest włączona w formularzach rezerwacji. Ta strona nie potrafi jej rozwiązać, więc dopóki jest włączona, droga rezerwacji odrzuca rezerwacje testowe.
dbg-err-username = Twoje konto nie ma nazwy użytkownika, więc nie ma stron rezerwacji.
dbg-slot = Termin
dbg-tz = Godziny w { $tz }, strefie czasowej twojego profilu.
dbg-no-slots = Brak wolnych terminów w najbliższych 31 dniach.
dbg-name = Imię i nazwisko
dbg-email = E-mail
dbg-email-help = Tylko twoje własne adresy: e-mail logowania i e-mail do rezerwacji.
dbg-telegram-username = Nazwa użytkownika Telegram
dbg-telegram-required = wymagana dla tego typu wydarzenia
dbg-notes = Notatki
dbg-notes-default = Rezerwacja testowa ze strony debugowania.
dbg-own-chat = Telegram: dostarczaj wiadomości dla gościa do mojego połączonego czatu Telegram
dbg-own-chat-off = Telegram jest na tym serwerze wyłączony (brak tokenu bota).
dbg-own-chat-linked = Twój połączony czat: { $who }.
dbg-own-chat-help = Strona gościa rezerwacji (co zapłacić, potwierdzenie, przypomnienie, anulowanie) trafia do czatu, w którym bot już do ciebie pisze, tak jakby link gościa został tam otwarty. Bez zaznaczenia strona płatności pokazuje link gościa jak zwykle.
dbg-own-chat-not-linked = Twój czat Telegram nie jest połączony. Najpierw go połącz, potem wróć tutaj:
dbg-own-chat-no-username = bez nazwy użytkownika
dbg-test-note = To prawdziwa rezerwacja oznaczona jako test. Dopóki istnieje, blokuje termin, liczy się do ceny według popytu i wysyła zwykłe e-maile i wiadomości na twój własny adres. Anuluj ją, gdy skończysz.
dbg-submit = Zarezerwuj jako gość
dbg-list = Moje rezerwacje testowe
dbg-list-none = Na razie brak rezerwacji testowych.
dbg-col-when = Kiedy
dbg-col-type = Typ wydarzenia
dbg-col-status = Status
dbg-col-payment = Płatność
dbg-col-telegram = Telegram
dbg-open-pay = strona płatności
dbg-own-chat-yes = mój czat
dbg-cancel-all = Anuluj wszystkie moje rezerwacje testowe ({ $count })
dbg-cancel-all-help = Anuluje każdą twoją rezerwację testową, która nadal obowiązuje (oczekującą odrzuca), tak jak robi to panel rezerwacji: otwarte faktury są zamykane, a zwykłe powiadomienia wysyłane. Inne rezerwacje pozostają nietknięte.
dbg-cancel-reason = Rezerwacja testowa anulowana ze strony debugowania.
dbg-impersonating = Zakończ podszywanie się pod innego użytkownika, aby korzystać ze strony rezerwacji testowej.
dbg-err-type = Wybierz jeden ze swoich typów wydarzeń.
dbg-err-email = Użyj jednego ze swoich adresów.
dbg-err-slot = Wybierz termin z listy.
dbg-err-own-chat = Najpierw połącz swój czat Telegram albo odznacz pole Telegram.
dbg-own-chat-failed = Rezerwacja testowa została utworzona, ale wiadomości dla gościa nie udało się skierować do twojego czatu Telegram: czat został odłączony, bot zatrzymany albo link w międzyczasie wygasł. Rezerwacja obowiązuje i zajmuje termin: otwórz jej stronę płatności z listy poniżej albo ją anuluj.
dbg-st-pending = oczekuje
dbg-st-confirmed = potwierdzona
dbg-st-cancelled = anulowana
dbg-st-declined = odrzucona
dbg-inv-open = czeka na płatność
dbg-inv-paid = opłacona
dbg-inv-confirmed = opłacona i potwierdzona
dbg-inv-expired = wygasła
dbg-inv-closed = zamknięta
dbg-inv-waived = potwierdzona bez płatności
dbg-banner = DEBUG · rezerwacja testowa
dbg-badge-test = TEST
ton-admin-debug-title = Debugowanie
ton-admin-debug-intro = Rezerwacja testowa: rezerwuj własne typy wydarzeń tak jak gość (prawdziwa cena, faktura, strona płatności i Telegram), bez wypełniania formularza gościa. Tylko dla administratorów. Gdy jest wyłączona, jej strona odpowiada 404 i nic do niej nie prowadzi.
ton-admin-debug-state-on = Rezerwacja testowa: włączona
ton-admin-debug-state-off = Rezerwacja testowa: wyłączona
ton-admin-debug-turn-on = Włącz
ton-admin-debug-turn-off = Wyłącz
ton-admin-debug-open = Otwórz rezerwację testową
ton-admin-debug-locked-env = Ustawione przez środowisko serwera ({ $var }={ $value }). Aby przełączać to tutaj, usuń { $var } ze środowiska i uruchom ponownie calrs.
ton-admin-debug-locked-apply = Ustawione przez config apply (site.ton.debug_booking). Zmień to w dokumencie config apply albo pomiń klucz, aby przekazać to tej stronie.
ton-admin-debug-saved-on = Rezerwacja testowa jest włączona.
ton-admin-debug-saved-off = Rezerwacja testowa jest wyłączona.
ton-admin-debug-saved-locked = Nic nie zmieniono: przełącznik jest zablokowany.
ton-admin-f-debug_booking = Rezerwacja testowa
dbg-telegram-from-chat = Uzupełnione z twojego połączonego czatu Telegram.
dbg-telegram-from-setting = Uzupełnione z bloku Debug na stronie
dbg-telegram-none = Nie ma czego uzupełnić: ustaw nazwę w bloku Debug na stronie
dbg-timing = Terminy tej rezerwacji testowej
dbg-timing-normal = Zwykłe: ustawienia witryny (strona „Płatności TON”)
dbg-timing-fast = Szybki test: { $window } min na zapłatę; przedłużenia: { $renewals }, po { $renew } min (gość jest proszony o ponowną zapłatę); potem faktura wygasa, a rezerwacja jest anulowana automatycznie. Karencja { $grace } min, margines { $margin } min.
dbg-timing-custom = Własne, w minutach (puste pole przyjmuje wartość szybkiego testu):
dbg-timing-window = Okno płatności, min
dbg-timing-renew-window = Przedłużone okno, min
dbg-timing-renewals = Przedłużenia
dbg-timing-help = Tylko dla faktury tej rezerwacji testowej; może być krótsze, niż pozwala witryna (co najmniej { $min } min). Okno i tak kończy się przed minimalnym wyprzedzeniem spotkania. Bez zapłaty rezerwacja jest anulowana automatycznie: w tym czasie obserwator sprawdza mniej więcej co 10 sekund (rzadziej tylko wtedy, gdy indeksator TON ogranicza zapytania lub je odrzuca), a calrs zwalnia termin w ciągu minuty od wygaśnięcia faktury. Płatność nigdy nie jest oznaczana jako otrzymana.
dbg-err-timing = Wybierz terminy. Własne minuty: okno 2–240, przedłużone okno 2–1440, przedłużenia 0–5.
dbg-col-timing = Terminy
dbg-timing-site = ustawienia witryny
dbg-timing-summary = test: { $window } min, przedłużenia { $renewals } × { $renew } min
dbg-renewals = przedłużono { $n } z { $max }
dbg-cd-until = zapłać do { $expires } (przelew liczy się do { $settle })
dbg-cd-pay = na zapłatę zostało { $time }
dbg-cd-grace = okno minęło; przelew wysłany w ciągu najbliższych { $time } jeszcze się liczy
dbg-cd-wait = okno minęło: przedłużenie lub wygaśnięcie w następnym kroku obserwatora
dbg-list-live = Dopóki rezerwacja testowa oczekuje, ta lista odświeża się sama co 10 sekund.
dbg-guest-lang = Język gościa
dbg-guest-lang-help = Język stron, e-maili i wiadomości Telegram gościa dla tej rezerwacji testowej. Domyślnie język twojego interfejsu. E-maile do gospodarzy pozostają w języku każdego gospodarza.
dbg-err-lang = Wybierz język gościa z listy.
dbg-col-lang = Język
dbg-no-chat-title = Twój czat Telegram nie jest połączony
dbg-no-chat-body = „Dostarczaj wiadomości dla gościa do mojego połączonego czatu Telegram” jest niedostępne, dopóki nie połączysz czatu w Profil i ustawienia → Telegram. Połącz go tam i wróć na tę stronę.
dbg-no-chat-link = Połącz mój czat Telegram
ton-admin-debug-tg-label = Nazwa użytkownika Telegram gościa dla rezerwacji testowych
ton-admin-debug-tg-save = Zapisz
ton-admin-debug-tg-help = Uzupełniana na stronie rezerwacji testowej, gdy twój połączony czat Telegram nie ma nazwy użytkownika (dla typów wydarzeń, które o nią pytają). Bez @; puste: brak.
ton-admin-debug-tg-locked-apply = Zablokowane przez config apply (site.ton.debug_booking_telegram). Zmień to w dokumencie config apply albo pomiń klucz, aby przekazać je tej stronie.
ton-admin-debug-tg-saved = Nazwa użytkownika Telegram gościa zapisana.
ton-admin-debug-tg-invalid = Nic nie zapisano: to nie jest nazwa użytkownika Telegram (litera łacińska, potem litery, cyfry i _, od 4 do 32 znaków).
ton-admin-f-debug_booking_telegram = Rezerwacja testowa: nazwa użytkownika Telegram gościa

## ilia fork: event type translations (src/web/event_texts.rs)
etf-translations = Tłumaczenia
etf-translations-help = Goście, którzy oglądają strony rezerwacji w jednym z tych języków, widzą ten tytuł i opis zamiast powyższych, także w e-mailach i wiadomościach. Pozostałe języki pokazują te powyżej.
etf-translation-language = Język
etf-translation-description-help = Obsługuje **pogrubienie**, *kursywę* i [linki](url). Jeśli zostawisz puste, w tym języku nie będzie opisu.
etf-translation-add = Dodaj język
etf-translation-remove = Usuń
form-error-translation-language = Wybierz język dla każdego tłumaczenia.
form-error-translation-duplicate = Każdy język może mieć tylko jedno tłumaczenie.
form-error-translation-title-required = Każde tłumaczenie musi mieć tytuł.
form-error-translation-title-length = Przetłumaczony tytuł może mieć najwyżej 255 znaków.

## ilia fork: 3CX video meetings (src/web/threecx.rs)
admin-threecx = Spotkania wideo 3CX
admin-threecx-help = Każda potwierdzona rezerwacja typu wydarzenia z miejscem „Spotkanie 3CX” dostaje własne zaplanowane spotkanie w tej centrali 3CX. Goście dostają link do spotkania, gospodarze link organizatora w kalendarzu i e-mailach. Przeniesienie lub odwołanie rezerwacji przenosi lub usuwa spotkanie. Adres e-mail gościa nigdy nie trafia do 3CX, więc 3CX nic do gościa nie wysyła.
admin-threecx-base-url = Adres centrali
admin-threecx-extension = Numer wewnętrzny organizatora
admin-threecx-extension-help = Numer wewnętrzny, do którego należą spotkania, na przykład 100.
admin-threecx-api-key = Klucz API wideokonferencji
admin-threecx-api-key-help = Klucz API wideokonferencji 3CX z konsoli administracyjnej 3CX. Jest przechowywany w postaci zaszyfrowanej i nigdy więcej nie jest pokazywany.
admin-threecx-display-name-help = Co goście widzą jako miejsce, na przykład „3CX”. Puste: „Rozmowa wideo”.
admin-threecx-save = Zapisz ustawienia 3CX
admin-threecx-test = Sprawdź połączenie
admin-threecx-clear = Usuń ustawienia 3CX
admin-threecx-notice-saved = Ustawienia 3CX zapisane.
admin-threecx-notice-cleared = Ustawienia 3CX usunięte razem z kluczem.
admin-threecx-notice-ok = 3CX odpowiedziała: adres i klucz działają.
admin-threecx-notice-auth = 3CX odrzuciła klucz. Wywołania 3CX są wstrzymane na godzinę, aby centrala nie zablokowała tego serwera; zapisanie nowego klucza lub ponowne sprawdzenie kończy przerwę.
admin-threecx-notice-transport = 3CX nie odpowiedziała. Sprawdź adres i czy ten serwer może się z nią połączyć.
admin-threecx-notice-service = 3CX odpowiedziała błędem. Sprawdź, czy wideokonferencje są włączone dla numeru wewnętrznego.
admin-threecx-notice-bad-link = 3CX zwróciła link do spotkania pod innym adresem. Sprawdź adres centrali.
admin-threecx-notice-not-set = Najpierw podaj adres, numer wewnętrzny i klucz.
admin-threecx-notice-bad-base-url = Nie zapisano: adres musi zaczynać się od https:// i zawierać tylko host, bez ścieżki.
admin-threecx-notice-bad-extension = Nie zapisano: numer wewnętrzny musi mieć od 1 do 10 cyfr.
admin-threecx-notice-bad-api-key = Nie zapisano: klucz musi mieć od 16 do 256 znaków bez spacji.
admin-threecx-notice-bad-display-name = Nie zapisano: etykieta może mieć najwyżej 100 znaków.
admin-threecx-notice-bad-pair = Nie zapisano: adres i numer wewnętrzny idą w parze, a klucz wymaga obu.
admin-threecx-notice-key-env = Nie zapisano: klucz jest ustawiany przy wdrożeniu (CALRS_THREECX_API_KEY). Zmień go tam.
admin-threecx-api-key-env = Ustawiany przy wdrożeniu (CALRS_THREECX_API_KEY). Zmień go w sekretach wdrożenia i wdróż ponownie.
admin-threecx-api-key-env-bad = CALRS_THREECX_API_KEY jest ustawiony, ale nie jest kluczem (od 16 do 256 znaków bez spacji): spotkania 3CX są wyłączone, dopóki wdrożenie tego nie poprawi.
admin-threecx-api-key-env-placeholder = Ustawiany przy wdrożeniu
etf-location-threecx = Spotkanie 3CX (tworzone dla każdej rezerwacji)
etf-threecx-hint = Dla każdej potwierdzonej rezerwacji tworzone jest spotkanie 3CX, które podąża za jej przeniesieniem i odwołaniem. 3CX konfiguruje się na stronie administracyjnej.
