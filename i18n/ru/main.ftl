# Booking confirmation page (templates/confirmed.html)

confirmed-page-title-pending = Запись ожидает подтверждения
confirmed-page-title-booked = Запись подтверждена

confirmed-heading-reschedule-requested = Запрошен перенос
confirmed-heading-rescheduled = Запись перенесена!
confirmed-heading-pending = Ожидает подтверждения
confirmed-heading-booked = Вы записаны!

confirmed-subtitle-reschedule-requested = Запрос на перенос отправлен организатору ({ $host }). Когда перенос подтвердят, на { $email } придёт письмо.
confirmed-subtitle-rescheduled = Ваша запись перенесена. Письмо с подтверждением отправлено на { $email }.
confirmed-subtitle-pending = Запрос на запись отправлен организатору ({ $host }). Когда запись подтвердят, на { $email } придёт письмо.
confirmed-subtitle-booked = Письмо с подтверждением отправлено на { $email }.

confirmed-detail-event = Встреча:
confirmed-detail-date = Дата:
confirmed-detail-time = Время:
confirmed-detail-with = Организатор:
confirmed-detail-location = Место:
confirmed-detail-notes = Комментарий:
confirmed-detail-additional-guests = Дополнительные гости:

confirmed-book-another = Записаться на другое время

confirmed-add-to-calendar = Добавить в календарь

# Slot picker (templates/slots.html)

slots-location-video = Видеозвонок
slots-location-phone = Телефонный звонок
slots-location-google-meet = Google Meet

slots-tz-label = Ваш часовой пояс
slots-time-format-label = Формат времени
tz-group-common = Популярные часовые пояса
tz-region-africa = Африка
tz-region-america = Америка
tz-region-antarctica = Антарктида
tz-region-arctic = Арктика
tz-region-asia = Азия
tz-region-atlantic = Атлантический океан
tz-region-australia = Австралия
tz-region-europe = Европа
tz-region-indian = Индийский океан
tz-region-pacific = Тихий океан
slots-offset-note-day = %1: время указано по %2 (перевод часов — %3).
slots-offset-note-range = Перевод часов — %1: с этого дня время указано по %2.

slots-view-month = Месяц
slots-view-week = Неделя
slots-view-column = Список

slots-weekday-mon = Пн
slots-weekday-tue = Вт
slots-weekday-wed = Ср
slots-weekday-thu = Чт
slots-weekday-fri = Пт
slots-weekday-sat = Сб
slots-weekday-sun = Вс

slots-weekday-mon-short = Пн
slots-weekday-tue-short = Вт
slots-weekday-wed-short = Ср
slots-weekday-thu-short = Чт
slots-weekday-fri-short = Пт
slots-weekday-sat-short = Сб
slots-weekday-sun-short = Вс

slots-select-date = Выберите дату
slots-loading-availability = Загружаем свободное время…
slots-click-highlighted = Выберите отмеченную дату, чтобы увидеть свободное время
slots-bookable-range-both = Запись открыта с { $from } по { $until }
slots-bookable-range-from = Запись открыта с { $from }
slots-bookable-range-until = Запись открыта до { $until }
slots-no-longer-bookable = Запись на эту встречу закрыта.
slots-no-times-month = В этом месяце нет свободного времени
slots-no-times-day = В этот день нет свободного времени
slots-no-availability-participants = В этом месяце нет времени, когда свободны все участники
slots-week-more = ещё

# Booking form (templates/book.html)

book-page-title = Запись: { $title }
book-back-to-times = Назад к выбору времени
book-name-label = Ваше имя
book-name-placeholder = Анна Смирнова
book-email-label = Эл. почта
book-email-placeholder = anna@example.com
book-email-invalid = Введите адрес эл. почты полностью, с доменом (например, anna@example.com).
book-notes-label = Комментарий
book-notes-optional = (необязательно)
book-notes-placeholder = Что вы хотели бы обсудить?
book-additional-guests-label = Дополнительные гости
book-additional-guests-hint = (необязательно, не более { $max })
book-add-guest-btn = + Добавить адрес гостя
book-guest-email-placeholder = colleague@example.com
book-phone-label = Номер телефона
book-phone-placeholder = 050 123 4567
book-phone-help = Можно ввести местный номер: если он начинается не с +, используется код страны { $country }.
book-phone-optional-consequence = Оставьте поле пустым, если не хотите получать SMS об этой записи.
book-phone-required = Для этой записи нужен номер телефона.
book-phone-invalid-title = Неверный номер телефона
book-phone-invalid = Введите номер, на который можно отправить SMS, или оставьте поле пустым.
book-phone-country-search = Поиск
book-phone-country-label = Выберите страну
book-phone-country-none = Страна не выбрана
book-phone-country-no-results = Ничего не найдено
captcha-label = Проверка безопасности
captcha-initial-state = Подтвердите, что вы не робот
captcha-verifying = Проверяем…
captcha-solved = Вы не робот
captcha-error = Ошибка
captcha-troubleshooting = Помощь
captcha-wasm-disabled = Включите WASM, чтобы проверка проходила намного быстрее
captcha-verify-aria = Нажмите, чтобы подтвердить, что вы не робот
captcha-verifying-aria = Идёт проверка, подождите
captcha-verified-aria = Проверка пройдена
captcha-required = Подтвердите, что вы не робот
captcha-error-aria = Что-то пошло не так, попробуйте ещё раз
book-confirm-button = Записаться

# SMS notifications (src/sms/message.rs).
#
# These are text messages, billed per 160-character segment (70 if the text
# contains any character outside the GSM-7 alphabet, which includes most
# accented letters). Keep them short and plain.

sms-confirmed = Запись подтверждена: { $event }, { $date } в { $time } ({ $tz }).
sms-cancelled = Запись отменена: { $event }, { $date } в { $time } ({ $tz }).
sms-rescheduled = Запись перенесена: { $event }, теперь { $date } в { $time } ({ $tz }).
sms-reminder = Напоминание: { $event }, { $date } в { $time } ({ $tz }).

# Shared labels used across the cancel / decline / approve / reschedule / claim flows

common-detail-guest = Гость:
common-detail-reason = Причина:
common-reason-optional = (необязательно)
common-close-page = Эту страницу можно закрыть.

# Cancel flow (booking_cancel_form.html, booking_cancelled_guest.html)

cancel-page-title = Отмена записи
cancel-heading = Отмена записи
cancel-subtitle = Вы собираетесь отменить запись.
cancel-reason-label = Причина
cancel-reason-placeholder-host = Сообщите организатору причину…
cancel-button = Отменить запись
cancelled-heading = Запись отменена
cancelled-subtitle = Ваша запись отменена, организатор получил уведомление.

# Decline flow (booking_decline_form.html, booking_declined.html)

decline-page-title = Отклонение записи
decline-heading = Отклонение записи
decline-subtitle = Вы собираетесь отклонить этот запрос на запись.
decline-reason-placeholder-guest = Сообщите гостю причину…
decline-button = Отклонить запись
declined-heading = Запись отклонена
declined-subtitle = Запись отклонена, гость получил уведомление.

# Approve flow (booking_approve_form.html, booking_approved.html)

approve-page-title = Подтверждение записи
approve-heading = Подтверждение записи
approve-subtitle = Вы собираетесь подтвердить этот запрос на запись.
approve-button = Подтвердить запись
approved-heading = Запись подтверждена
approved-subtitle = Запись подтверждена, письмо с подтверждением отправлено на { $email }.

# Claim flow (booking_claim_form.html, booking_claimed.html, booking_already_claimed.html)

claim-page-title = Взять запись
claim-heading = Взять запись
claim-subtitle = Вы собираетесь взять эту запись. Вас добавят в участники встречи.
claim-assigned-to = Ответственный:
claim-button = Взять запись
claimed-page-title = Вы взяли запись
claimed-heading = Вы взяли запись
claimed-subtitle = Вы взяли эту запись. Приглашение в календарь отправлено вам на почту.
already-claimed-page-title = Запись уже взята
already-claimed-heading = Запись уже взята
already-claimed-subtitle = Эта запись уже взята: { $name }.

# Generic error page (booking_action_error.html)

action-error-page-title = Не удалось выполнить действие с записью

# Host-initiated reschedule (booking_host_reschedule.html)

host-resched-page-title = Перенос записи — calrs
host-resched-heading = Перенос записи
host-resched-subtitle = { $guest } получит письмо с просьбой выбрать новое время.
host-resched-currently = Запланировано:
host-resched-button = Отправить запрос на перенос
host-resched-cancel-link = Отмена

# Guest reschedule confirmation (booking_reschedule_confirm.html)

resched-confirm-page-title = Подтверждение переноса
resched-confirm-heading = Подтверждение переноса
resched-confirm-subtitle = Вы собираетесь перенести запись на новое время.
resched-was = Было:
resched-new = Стало:
resched-button = Подтвердить перенос
resched-back-to-picker = Назад к выбору времени

# Base layout chrome (templates/base.html)

base-loader-checking = Проверяем свободное время
base-loader-please-wait = Подождите, загружаем актуальные данные календаря…
base-stop-impersonating = Вернуться в свой аккаунт
base-theme-toggle = Сменить тему
base-language-choose = Выберите язык
base-powered-by = Сделано на

# Profile (templates/profile.html)

profile-pick-event-type-invite = Выберите тип встречи и запишитесь на удобное время.
profile-no-event-type = Пока нет доступных типов встреч.

# Month and weekday names + per-locale date format patterns.
# Used by server-side date formatters in src/i18n.rs.

common-month-1 = январь
common-month-2 = февраль
common-month-3 = март
common-month-4 = апрель
common-month-5 = май
common-month-6 = июнь
common-month-7 = июль
common-month-8 = август
common-month-9 = сентябрь
common-month-10 = октябрь
common-month-11 = ноябрь
common-month-12 = декабрь

common-weekday-long-mon = понедельник
common-weekday-long-tue = вторник
common-weekday-long-wed = среда
common-weekday-long-thu = четверг
common-weekday-long-fri = пятница
common-weekday-long-sat = суббота
common-weekday-long-sun = воскресенье

# Format patterns are parametric per locale to handle word order. Translators
# pick where each placeholder lands. Example outputs:
#   EN: April 2026  /  Tuesday, March 12, 2026
#   FR: avril 2026  /  mardi 12 mars 2026
#   ES: abril 2026  /  martes, 12 de marzo de 2026
common-format-month-year = { $month } { $year }
common-format-long-date = { $weekday }, { $day } { $monthnum ->
    [1] января
    [2] февраля
    [3] марта
    [4] апреля
    [5] мая
    [6] июня
    [7] июля
    [8] августа
    [9] сентября
    [10] октября
    [11] ноября
   *[12] декабря
} { $year }
common-format-date =
    { $day } { $monthnum ->
    [1] января
    [2] февраля
    [3] марта
    [4] апреля
    [5] мая
    [6] июня
    [7] июля
    [8] августа
    [9] сентября
    [10] октября
    [11] ноября
    *[12] декабря
    } { $year }

# Email signatures and shared bits (src/email.rs)

email-signature = — calrs
email-action-reschedule = Перенести
email-action-cancel-booking = Отменить запись

# Email: guest booking confirmation

# Kept to "event — date": Exchange titles the guest appointment after the
# email Subject header, not the ICS SUMMARY (#157).
email-confirm-subject = { $event } — { $date }
email-confirm-greeting = Здравствуйте, { $name }!
email-confirm-headline = Ваша запись подтверждена.
email-confirm-ics-attached-plain = Приглашение в календарь во вложении.
email-confirm-ics-attached-html = К письму приложено приглашение в календарь.
email-confirm-need-to-cancel = Нужно отменить? { $url }

# Email: guest reminder

email-reminder-subject = Напоминание: { $event } в { $time }
email-reminder-headline = Встреча скоро начнётся.

# Email: guest cancellation

email-cancel-subject = Отменено: { $event } — { $date }
email-cancel-headline-by-host = Ваша запись отменена организатором ({ $host }).
email-cancel-headline-by-guest = Ваша запись отменена.
email-cancel-ics-attached-plain = Отмена для календаря во вложении.
email-cancel-ics-attached-html = К письму приложена отмена для календаря.

# Confirmation email: notice-window policy lines (src/email.rs)

email-confirm-cancel-notice = Важно: отменить запись можно не позднее чем за { $minutes } мин до начала встречи.
email-confirm-reschedule-notice = Важно: перенести запись можно не позднее чем за { $minutes } мин до начала встречи.

# Event type form: Google Meet location + cancel/reschedule minimum notice
# (templates/event_type_form.html, src/google_meet.rs)

event-type-form-location-google-meet = Google Meet (ссылка создаётся автоматически)
event-type-form-location-google-meet-hint = Уникальная ссылка Google Meet создаётся при подтверждении записи, её владелец — назначенный организатор. У каждого организатора (у вас или у каждого подходящего участника команды) должен быть подключён Google Calendar и выбран календарь для новых записей.

google-meet-prereq-no-host = Для Google Meet нужен организатор с подключённым Google Calendar.
google-meet-prereq-no-eligible = Для Google Meet нужен хотя бы один подходящий участник команды с подключённым Google Calendar.
google-meet-prereq-missing = Для Google Meet у каждого организатора должен быть подключён Google Calendar и выбран календарь для новых записей. Ещё не настроили: { $names }. Подключить календарь можно в разделе «Кабинет → Источники».
google-meet-unavailable-title = Google Meet недоступен
google-meet-dynamic-group-unavailable = Организатору нужно подключить Google Calendar и выбрать календарь для новых записей.

event-type-form-cancel-notice-label = Минимальный срок для отмены
event-type-form-reschedule-notice-label = Минимальный срок для переноса
event-type-form-notice-help = Оставьте пустым, чтобы не ограничивать.
event-type-form-resources-label = Необходимые ресурсы
event-type-form-resources-hint = Время доступно для записи, только если выбранные ресурсы свободны (с учётом режима ниже).
event-type-form-resources-mode-all = Все выбранные ресурсы должны быть свободны
event-type-form-resources-mode-round-robin = Достаточно одного свободного ресурса (он закрепляется за записью)
event-type-form-notice-unit-minutes = минуты
event-type-form-notice-unit-hours = часы
event-type-form-notice-unit-days = дни
event-type-form-booking-horizon-label = Глубина записи
event-type-form-booking-horizon-help = На сколько дней вперёд можно записаться. Оставьте пустым, чтобы не ограничивать; 0 — только на сегодня.
event-type-form-bookable-range-label = Период записи
event-type-form-bookable-from = с
event-type-form-bookable-until = по
event-type-form-bookable-range-help = Записаться можно только на эти дни. Оставьте пустым, чтобы не ограничивать. Чтобы задать другие часы для отдельных дней, используйте исключения для дат.
event-type-form-bookable-range-past-warning = Этот период уже закончился. Тип встречи больше не показывается публично.
event-type-form-bookable-range-too-far-warning = Этот период начинается больше чем через год. Записаться можно максимум на год вперёд, поэтому запись откроется с { $date }.
event-type-form-bookable-range-horizon-warning = Глубина записи заканчивается раньше, чем начинается этот период. Гости смогут записаться с { $date }.
event-type-form-bookable-range-bookings-outside =
    { $count ->
        [one] { $count } уже созданная запись не попадает в период записи.
        [few] { $count } уже созданные записи не попадают в период записи.
        [many] { $count } уже созданных записей не попадают в период записи.
       *[other] { $count } уже созданные записи не попадают в период записи.
    }
event-type-form-bookable-range-bookings-link = Показать записи
event-type-form-saved = Сохранено.

# Booking confirmation: cancel/reschedule policy notices (templates/confirmed.html)

confirmed-cancel-notice-info = Отменить запись можно не позднее чем за { $minutes } мин до начала встречи.
confirmed-reschedule-notice-info = Перенести запись можно не позднее чем за { $minutes } мин до начала встречи.

# Booking action blocked page (templates/booking_action_blocked.html)

booking-blocked-title-cancel = Эту запись уже нельзя отменить онлайн
booking-blocked-title-reschedule = Эту запись уже нельзя перенести онлайн
booking-blocked-body = Отменить или перенести запись можно не позднее чем за { $minutes } мин до начала встречи. Если вы не сможете прийти, напишите организатору: <a href="mailto:{ $host_email }">{ $host_email }</a>.

# Dashboard event types listing (templates/dashboard_event_types.html)

dashboard-event-types-copy = Скопировать
dashboard-event-types-copied = Скопировано
dashboard-event-types-copy-title = Скопировать ссылку для записи
dashboard-event-types-copy-failed = Не удалось скопировать

# Dashboard sidebar and shared chrome (templates/dashboard_base.html)

nav-section-scheduling = Планирование
nav-overview = Обзор
nav-event-types = Типы встреч
nav-bookings = Записи
nav-teams = Команды
nav-section-shared-links = Общие ссылки
nav-invite-links = Ссылки-приглашения
nav-section-calendars = Календари
nav-sources = Источники
nav-section-personal = Личное
nav-settings = Профиль и настройки
nav-troubleshoot = Диагностика
nav-section-admin = Администрирование
nav-admin-panel = Панель администратора
nav-sign-out = Выйти
nav-release-notes = Список изменений

# Timezone mismatch banner (templates/dashboard_base.html)

tz-banner-text = Часовой пояс браузера — { $detected }, а для записей у вас указан { $current }.
tz-banner-update = Обновить
tz-banner-dismiss = Скрыть

# Markdown editor toolbar (templates/dashboard_base.html)

editor-link-prompt = Введите адрес ссылки:
editor-link-default-label = текст ссылки
editor-placeholder-text = текст
editor-nothing-to-preview = Нечего показать

# Dashboard overview (templates/dashboard_overview.html)

overview-page-title = Кабинет
overview-welcome = Добро пожаловать, { $name }
overview-public-page = Публичная страница:
overview-avail-banner-title = Доступность по умолчанию
overview-avail-banner-body = Для вас установлены рабочие часы по умолчанию: пн–пт, 9:00–17:00. Они учитываются, когда другие добавляют вас во встречи по групповой ссылке.
overview-avail-banner-cta = Проверить доступность
overview-dismiss = Скрыть
overview-getting-started = С чего начать
overview-getting-started-help = Выполните эти шаги, чтобы начать принимать записи.
overview-step-connect-calendar = Подключите календарь
overview-step-first-event-type = Создайте первый тип встречи
overview-step-share-link = Поделитесь ссылкой для записи
overview-pending-approval = Ожидают подтверждения
overview-booking-with = { $title }: { $guest }
overview-badge-pending = ожидает
overview-guest-booked = Время гостя:
overview-confirm = Подтвердить
overview-decline = Отклонить
overview-stat-event-types = Типы встреч
overview-stat-upcoming = Предстоящие записи
overview-stat-pending = Ожидают подтверждения
overview-stat-sources = Источники календарей
overview-quick-actions = Новый тип встречи
overview-action-public-title = Публичная страница записи
overview-action-public-desc = Поделитесь ссылкой — любой сможет выбрать время и записаться к вам.
overview-action-team-title = Командные встречи
overview-action-team-desc = Распределяйте записи между участниками команды или находите время, когда свободны все.
overview-action-team-desc-empty = Сначала создайте команду, затем настройте общие типы встреч.
overview-action-private-title = Только по приглашению
overview-action-private-desc = Создавайте одноразовые ссылки для конкретных людей. Больше никто не сможет записаться.
overview-action-shared-title = Общие ссылки-приглашения
overview-action-shared-desc = Любой участник команды может создавать ссылки для записи и делиться ими с людьми вне команды.
overview-action-reason-calendar = Сначала подключите календарь
overview-action-reason-ask-admin = Попросите администратора создать команду
overview-action-reason-team-admin = Нужна команда — сначала создайте её
overview-action-reason-team-member = Нужна команда — обратитесь к администратору

# Dashboard bookings (templates/dashboard_bookings.html)

bookings-page-title = Записи
bookings-pending-approval = Ожидают подтверждения
bookings-available-to-claim = Можно взять
bookings-upcoming = Предстоящие записи
bookings-with = { $title }: { $guest }
bookings-guest-booked = Время гостя:
bookings-resource = Ресурс:
bookings-confirm = Подтвердить
bookings-reschedule = Перенести
bookings-decline = Отклонить
bookings-claim = Взять
bookings-badge-awaiting-reschedule = ждёт переноса
bookings-badge-writeback-failed = нет в вашем календаре
bookings-writeback-failed-detail = Не удалось добавить в календарь: { $error }
bookings-cancel = Отменить
bookings-reason-placeholder = Причина (необязательно)
bookings-confirm-cancel = Подтвердить отмену
bookings-back = Назад
bookings-empty = Предстоящих записей пока нет.<br>Поделитесь { $link }, чтобы к вам могли записаться.
bookings-empty-link-label = ссылками на типы встреч

# Dashboard teams listing (templates/dashboard_teams.html)

teams-page-title = Команды
teams-heading = Команды
teams-new = Создать
teams-badge-public = публичная
teams-badge-private = закрытая
teams-settings = Настройки
teams-view = Открыть
teams-empty = Команд пока нет.
teams-empty-admin = { $link }, чтобы работать вместе с коллегами.
teams-empty-admin-link-label = Создайте команду
teams-empty-member = Команды создают администраторы. Попросите администратора создать команду и добавить вас в неё.

# Dashboard invite links (templates/dashboard_internal.html)

invite-links-page-title = Ссылки-приглашения
invite-links-heading = Ссылки-приглашения
invite-links-new = Создать внутренний тип встречи
invite-links-help = Создавайте одноразовые ссылки для записи на внутренние типы встреч. Любой коллега с аккаунтом может создавать здесь ссылки и делиться ими.
invite-links-duration = { $minutes } мин
invite-links-hosted-by = Организатор: { $host }
invite-links-get-link = Получить ссылку
invite-links-invites = Приглашения
invite-links-empty = Внутренних типов встреч пока нет.<br>{ $link } с видимостью «Внутренний», чтобы любой коллега мог создавать ссылки для записи.
invite-links-empty-link-label = Создайте тип встречи
invite-links-js-generating = Создаём…
invite-links-js-copied = Скопировано
invite-links-js-error = Ошибка

teams-member-count =
    { $count ->
        [one] { $count } участник
        [few] { $count } участника
        [many] { $count } участников
       *[other] { $count } участника
    }

# Dashboard calendar sources (templates/dashboard_sources.html)

sources-page-title = Источники календарей
sources-heading = Источники календарей
sources-add = Добавить
sources-last-sync = Последняя синхронизация:
sources-sync-failing = Ошибка синхронизации с { $since } UTC: { $error }
sources-sync = Синхронизировать
sources-full-resync = Полная синхронизация
sources-full-resync-title = Очистить кэш и заново загрузить все события с сервера
sources-test = Проверить
sources-reconnect = Переподключить
sources-reconnect-title = Заново пройти авторизацию в Google
sources-edit = Изменить
sources-remove = Удалить
sources-remove-confirm = Удалить источник «{ $name }»? Все события, синхронизированные из него, будут удалены.
sources-no-write-calendar = Календарь для новых записей не выбран. Подтверждённые записи остаются в calrs и не попадают в этот календарь. Выберите календарь ниже, чтобы записи добавлялись в него.
sources-write-bookings-to = Добавлять записи в:
sources-write-none = Никуда (не добавлять)
sources-empty = Источники календарей не подключены. { $link }, чтобы calrs учитывал вашу занятость.
sources-empty-link-label = Добавьте источник
sources-calendars = Календари
sources-calendars-synced = Синхронизируется календарей: { $synced } из { $total }
source-calendars-page-title = Календари для синхронизации
source-calendars-heading = Календари для синхронизации
source-calendars-back = Вернуться к источникам календарей
source-calendars-help = Выберите календари источника «{ $name }», которые будет синхронизировать calrs. Неотмеченные календари не загружаются и не влияют на вашу занятость.
source-calendars-select-all = Выбрать все
source-calendars-select-none = Снять все
source-calendars-write-badge = Сюда добавляются записи
source-calendars-write-hint = Подтверждённые записи добавляются в этот календарь, даже если он не синхронизируется.
source-calendars-save = Сохранить
source-calendars-saved = Сохранено. Только что отмеченные календари загрузятся при следующей синхронизации; чтобы загрузить их сейчас, нажмите «Синхронизировать» на странице источников.
source-calendars-empty = Для этого источника календари пока не найдены. Запустите синхронизацию, чтобы их найти.

# Dashboard event types listing (templates/dashboard_event_types.html)

event-types-page-title = Типы встреч
event-types-heading = Типы встреч
event-types-new = Создать
event-types-badge-disabled = выключен
event-types-badge-internal = внутренний
event-types-badge-private = закрытый
event-types-badge-resources = ресурсы
event-types-badge-active = активен
event-types-badge-expired = истёк
event-types-badge-range-from = с { $date }
event-types-badge-range-until = до { $date }
event-types-send-invites = Отправить приглашения
event-types-duration = { $minutes } мин
event-types-mode-collective = совместная
event-types-mode-round-robin = по очереди
event-types-edit = Изменить
event-types-disable = Выключить
event-types-enable = Включить
event-types-embed = Встроить
event-types-overrides = Исключения для дат
event-types-team-settings = Настройки команды
event-types-invites = Приглашения
event-types-view-public = Открыть публичную страницу
event-types-view-page = Открыть страницу
event-types-delete = Удалить
event-types-delete-confirm = Удалить тип встречи «{ $title }»? Это действие нельзя отменить.
event-types-duplicate = Создать копию
event-types-duplicate-title = { $title } (копия)
event-types-empty = Типов встреч пока нет. { $link }, чтобы начать принимать записи.
event-types-empty-link-label = Создайте тип встречи

# Markdown editor toolbar (templates/settings.html, templates/team_form.html)

editor-bold = Жирный (Ctrl+B)
editor-italic = Курсив (Ctrl+I)
editor-strikethrough = Зачёркнутый
editor-code = Код
editor-link = Вставить ссылку (Ctrl+K)
editor-toggle-preview = Показать или скрыть предпросмотр
editor-preview = Предпросмотр

# Profile and settings (templates/settings.html)

settings-page-title = Настройки
settings-heading = Профиль и настройки
settings-public-page-label = Ваша публичная страница записи
settings-copy = Скопировать
settings-copied = Скопировано
settings-open = Открыть
settings-avatar = Фото профиля
settings-upload = Загрузить
settings-remove = Удалить
settings-display-name = Отображаемое имя
settings-display-name-placeholder = Ваше имя
settings-username = Имя пользователя
settings-username-hint = (используется в ссылке на вашу страницу записи)
settings-username-pattern-title = Только строчные латинские буквы, цифры и дефисы
settings-username-help = Ваша публичная страница записи:
settings-title = Должность
settings-title-placeholder = например, разработчик, менеджер продукта
settings-title-help = Показывается в вашем публичном профиле и на боковой панели.
settings-bio = О себе
settings-bio-placeholder = Расскажите немного о себе…
settings-bio-help = Показывается на вашей публичной странице записи. Можно использовать **жирный**, *курсив*, ~~зачёркнутый~~, `code` и [ссылки](url).
settings-booking-email = Адрес для записей
settings-booking-email-help = Этот адрес будет указан на ваших публичных страницах записи и в уведомлениях по эл. почте. Оставьте поле пустым, чтобы использовать адрес, с которым вы входите.
settings-booking-email-warning = Убедитесь, что такой адрес есть у вашего почтового сервиса, иначе уведомления не дойдут.
settings-timezone = Часовой пояс
settings-timezone-help = Ваши правила доступности и время записей рассчитываются в этом часовом поясе.
settings-language = Язык
settings-language-auto = Авто (как в браузере)
settings-language-help = Выберите язык интерфейса или оставьте «Авто», чтобы использовать настройку браузера.
settings-public-page-language = Язык публичной страницы записи
settings-public-page-language-visitor = Язык посетителя
settings-public-page-language-host = Всегда мой язык
settings-public-page-language-help = Язык вашего публичного профиля, страниц записи и писем, которые получают ваши гости. «Всегда мой язык» — язык, выбранный выше.
settings-error-public-page-language = Выберите допустимый язык публичной страницы записи.
settings-dynamic-group = Разрешить другим добавлять меня в групповые ссылки
settings-dynamic-group-help = Если включено, другие пользователи могут создавать ссылки на совместные встречи с вашим участием (например, { $example }).
settings-lend-resource = Разрешить использовать доступ к моему календарю для бронирования ресурсов
settings-lend-resource-help = Если для записи нужно забронировать общий ресурс (демолабораторию, переговорную), а ваш аккаунт календаря может добавлять в него события, calrs сможет использовать для этого ваши сохранённые учётные данные календаря.
settings-default-availability = Доступность по умолчанию
settings-default-availability-help = Ваши рабочие часы по умолчанию. Используются в групповых ссылках, когда другие добавляют вас во встречу.
settings-copy-to-all = Скопировать на все дни
settings-copy-to-all-title = Скопировать интервалы первого включённого дня на все остальные включённые дни
settings-add-window = Добавить интервал
settings-remove-window = Убрать интервал
settings-save = Сохранить настройки
settings-appearance = Внешний вид
settings-theme-system = Как в системе
settings-theme-light = Светлая
settings-theme-dark = Тёмная

# Sign in (templates/auth/login.html)

login-page-title = Вход
login-heading = Вход
login-subtitle = Войдите в свой аккаунт calrs
login-sso = Войти через SSO
login-or = или
login-email = Эл. почта
login-password = Пароль
login-submit = Войти по эл. почте
login-no-account = Нет аккаунта? { $link }
login-register-link = Зарегистрироваться

# Registration (templates/auth/register.html)

register-page-title = Регистрация
register-heading = Создание аккаунта
register-subtitle = Создайте новый аккаунт calrs
register-domains-limited = Регистрация доступна только для доменов: { $domains }
register-name = Имя
register-name-placeholder = Ваше имя
register-email = Эл. почта
register-password = Пароль
register-password-hint = (не менее 12 символов)
register-submit = Создать аккаунт
register-have-account = Уже есть аккаунт? { $link }
register-signin-link = Войти

# Authentication errors (src/auth.rs)

auth-error-rate-limited = Слишком много попыток входа. Попробуйте позже.
auth-error-invalid-credentials = Неверный адрес эл. почты или пароль
auth-error-internal = Внутренняя ошибка
auth-error-registration-disabled = Регистрация отключена.
auth-error-name-length = Имя должно содержать от 1 до 255 символов
auth-error-email-length = Адрес эл. почты должен содержать от 1 до 255 символов
auth-error-email-invalid = Введите корректный адрес эл. почты
auth-error-email-domain = Этот почтовый домен не разрешён
auth-error-password-length = Пароль должен быть не короче 12 символов
auth-error-email-taken = Этот адрес эл. почты уже зарегистрирован
auth-error-create-failed = Не удалось создать аккаунт

# Calendar source test and write-back setup (templates/source_test.html, templates/source_write_setup.html)

source-test-page-title = Источник календарей
source-test-sync-heading = Синхронизация: { $name }
source-test-heading = Проверка подключения
source-write-page-title = Настройка добавления в календарь
source-write-back = Вернуться в кабинет
source-write-heading = Куда добавлять записи?
source-write-help = Когда кто-то записывается к вам на встречу, calrs может сам создать событие в вашем календаре. Выберите календарь источника «{ $name }», в который будут добавляться записи.
source-write-save = Сохранить
source-write-skip = Не сейчас
source-write-sync-results = Результаты синхронизации

source-write-event-count =
    { $count ->
        [one] { $count } событие
        [few] { $count } события
        [many] { $count } событий
       *[other] { $count } события
    }

# Date overrides (templates/overrides.html)

overrides-page-title = Исключения для дат
overrides-heading = Исключения для дат
overrides-back-teams = Вернуться к командам
overrides-back-event-types = Вернуться к типам встреч
overrides-intro = Добавьте исключения для отдельных дат: { $title }
overrides-add-heading = Новое исключение
overrides-date = Дата
overrides-type = Тип исключения
overrides-type-blocked = Закрыть весь день
overrides-type-custom = Особые часы
overrides-start-time = Начало
overrides-end-time = Конец
overrides-add-submit = Добавить исключение
overrides-existing = Текущие исключения
overrides-badge-blocked = закрыто
overrides-badge-custom = особые часы
overrides-delete = Удалить
overrides-delete-confirm = Удалить это исключение?
overrides-empty = Исключений для дат пока нет.<br>В форме выше можно закрыть отдельные даты (праздники, выходные) или задать особые часы.

# Public team page (templates/team_profile.html)

team-profile-subtitle = Выберите тип встречи, чтобы записаться.
team-profile-empty = Доступных типов встреч пока нет.

# Availability troubleshoot (templates/troubleshoot.html, src/web/mod.rs)

troubleshoot-page-title = Диагностика
troubleshoot-empty = Типы встреч не найдены. { $link }, чтобы начать диагностику доступности.
troubleshoot-empty-link-label = Создайте тип встречи
troubleshoot-subtitle = Почему время свободно или недоступно: { $title }
troubleshoot-duration = { $minutes } мин
troubleshoot-buffer-before = перерыв { $minutes } мин до встречи
troubleshoot-buffer-after = перерыв { $minutes } мин после встречи
troubleshoot-min-notice = мин. срок: { $minutes } мин
troubleshoot-blocked-override = Закрыто исключением для даты (выходной)
troubleshoot-custom-hours-active = Действует исключение с особыми часами (вместо недельного расписания)
troubleshoot-legend-available = Свободно
troubleshoot-legend-calendar-event = Событие календаря
troubleshoot-legend-booking = Запись
troubleshoot-legend-resource = Ресурс занят
troubleshoot-legend-outside = Нерабочее время
troubleshoot-legend-buffer = Перерыв / мин. срок
troubleshoot-blocked-slots = Недоступное время
troubleshoot-none-date-blocked = Эта дата закрыта исключением (выходной). Свободного времени нет.
troubleshoot-none-custom-hours = Действует исключение с особыми часами, но подходящих интервалов нет. Проверьте настройки исключения.
troubleshoot-none-no-rules = Для этого дня недели нет правил доступности. В этот день ({ $date }) записаться на этот тип встречи нельзя.
troubleshoot-none-all-bookable = В часы доступности ничего не заблокировано: записаться можно на любое время.
troubleshoot-label-outside = Вне часов доступности
troubleshoot-label-available = Свободно
troubleshoot-label-min-notice = Мин. срок ({ $minutes } мин)
troubleshoot-label-beyond-horizon = За пределами глубины записи (дней: { $days })
troubleshoot-label-outside-date-range = Вне периода записи
troubleshoot-label-buffer = Перерыв ({ $minutes } мин)
troubleshoot-label-resource-busy = Ресурс занят: { $names }
troubleshoot-detail-around = Рядом: { $label }
troubleshoot-detail-around-booking = Рядом с записью: { $guest }
troubleshoot-reason-calendar-event = Событие календаря: { $label }
troubleshoot-reason-booking = Запись: { $label }

# Invite management (templates/invite_form.html)

invites-heading = Приглашения
invites-back-teams = Вернуться к командам
invites-back-event-types = Вернуться к типам встреч
invites-intro = Отправьте ссылки-приглашения на встречу «{ $title }»
invites-capped = <strong>Обработаны не все адреса: за один раз можно отправить не более { $max }.</strong> Остальные отправьте отдельно.
invites-failed-hint = — подробности в журнале сервера.
invites-quick-link = Быстрая ссылка
invites-quick-link-help = Создайте одноразовую ссылку — она сразу скопируется в буфер обмена.
invites-get-link = Получить ссылку
invites-or-email = Или отправьте по эл. почте
invites-recipients = Получатели
invites-recipients-hint = (по одному адресу в строке, не более { $max })
invites-message = Личное сообщение
invites-message-hint = (необязательно, его получат все адресаты)
invites-message-placeholder = С удовольствием покажу вам, как всё работает…
invites-expires-in = Срок действия
invites-expires-days = { $days } дн.
invites-expires-never = Бессрочно
invites-allow-multiple = Разрешить получателю записываться несколько раз
invites-send = Отправить приглашения
invites-sent-heading = Отправленные приглашения
invites-badge-expired = истекло
invites-badge-used = использовано
invites-badge-active = активно
invites-sent-by = Отправитель: { $name }
invites-uses = Использовано: { $used } из { $max }
invites-expires-at = Действует до { $date }
invites-copy-link = Скопировать ссылку
invites-delete = Удалить
invites-delete-confirm = Удалить это приглашение?
invites-empty = Приглашений пока нет. Отправьте ссылку для записи с помощью формы выше.
invites-js-generating = Создаём…
invites-js-copied = Скопировано
invites-js-error = Ошибка

invites-sent-count =
    { $count ->
        [one] Отправлено { $count } приглашение.
        [few] Отправлено { $count } приглашения.
        [many] Отправлено { $count } приглашений.
       *[other] Отправлено { $count } приглашения.
    }

invites-skipped-invalid =
    { $count ->
        [one] Пропущена { $count } строка с ошибкой:
        [few] Пропущены { $count } строки с ошибками:
        [many] Пропущено { $count } строк с ошибками:
       *[other] Пропущены { $count } строки с ошибками:
    }

invites-skipped-duplicate =
    { $count ->
        [one] Пропущена { $count } повторяющаяся строка:
        [few] Пропущены { $count } повторяющиеся строки:
        [many] Пропущено { $count } повторяющихся строк:
       *[other] Пропущены { $count } повторяющиеся строки:
    }

invites-failed =
    { $count ->
        [one] Не удалось отправить { $count } приглашение (БД или SMTP):
        [few] Не удалось отправить { $count } приглашения (БД или SMTP):
        [many] Не удалось отправить { $count } приглашений (БД или SMTP):
       *[other] Не удалось отправить { $count } приглашения (БД или SMTP):
    }

# Calendar source form (templates/source_form.html)

source-form-title-edit = Изменение источника календарей
source-form-title-add = Добавление календаря
source-form-heading-edit = Изменение источника календарей
source-form-heading-add = Подключение календаря
source-form-subtitle-edit = Измените параметры подключения. Чтобы оставить текущий пароль, не заполняйте это поле. Если вы изменили адрес или имя пользователя, запустите синхронизацию, чтобы обновить список календарей.
source-form-subtitle-add = Подключите сервер CalDAV или Microsoft Exchange (EWS), чтобы calrs учитывал вашу занятость, когда гости записываются на встречи.
source-form-backend = Протокол
source-form-preset = Шаблон
source-form-connect-google = Подключить через Google
source-form-google-unavailable = Google Calendar недоступен. Обратитесь к администратору.
source-form-name = Отображаемое имя
source-form-name-placeholder = Мой календарь
source-form-url-caldav = Адрес CalDAV
source-form-url-ews = Адрес EWS
source-form-username = Имя пользователя
source-form-password = Пароль
source-form-password-keep = Оставьте пустым, чтобы не менять
source-form-password-placeholder = Пароль приложения или аккаунта
source-form-skip-test = Пропустить проверку подключения
source-form-skip-test-help = Отметьте, если проверка зависает (так бывает с некоторыми серверами BlueMind и Zimbra). Проверить подключение можно позже.
source-form-save = Сохранить изменения
source-form-add = Добавить источник
source-form-help-google-configured = Нажмите кнопку ниже, чтобы разрешить calrs доступ к вашему Google Calendar.
source-form-help-google-unconfigured = Интеграция с Google Calendar ещё не настроена. Попросите администратора указать учётные данные Google OAuth2 в панели администратора.

# Calendar source form: provider help (templates/source_form.html)

source-form-help-bluemind = <strong>BlueMind</strong> — укажите DAV-адрес вашего сервера BlueMind.<br> Обычно это <code>https://mail.yourcompany.com/dav/</code><br> Имя пользователя — ваш <strong>адрес эл. почты</strong> (например, <code>alice@yourcompany.com</code>), а не только имя для входа.<br> Если проверка подключения зависает, отметьте «Пропустить проверку подключения» и сразу запустите синхронизацию.
source-form-help-nextcloud = <strong>Nextcloud</strong> — укажите корневой адрес WebDAV, а не ссылку на отдельный календарь.<br> Обычно это <code>https://cloud.example.com/remote.php/dav</code>
source-form-help-fastmail = <strong>Fastmail</strong> — укажите в пути URL свой полный адрес эл. почты.<br> Пример: <code>https://caldav.fastmail.com/dav/calendars/user/you@fastmail.com/</code><br> Используйте пароль приложения (Settings &rarr; Privacy &amp; Security &rarr; Integrations).
source-form-help-icloud = <strong>iCloud</strong> — укажите <code>https://caldav.icloud.com/</code><br> Нужен пароль приложения, его можно создать на <a href="https://appleid.apple.com" target="_blank" style="color: var(--accent);">appleid.apple.com</a> (Безопасность &rarr; Пароли приложений).
source-form-help-zimbra = <strong>Zimbra</strong> — укажите DAV-адрес вашего сервера Zimbra.<br> Обычно это <code>https://mail.example.com/dav/</code>
source-form-help-sogo = <strong>SOGo</strong> — укажите DAV-адрес SOGo.<br> Обычно это <code>https://mail.example.com/SOGo/dav/</code>
source-form-help-radicale = <strong>Radicale</strong> — укажите корневой адрес сервера.<br> Обычно это <code>https://cal.example.com/</code>
source-form-help-baikal = <strong>Baikal</strong> — укажите DAV-адрес вашего сервера Baikal.<br> Обычно это <code>https://cal.example.com/dav.php/</code>
source-form-help-exchange = <strong>Microsoft Exchange (EWS)</strong> — укажите адрес SOAP:<br> <code>https://mail.example.com/EWS/Exchange.asmx</code><br> Имя пользователя — адрес почтового ящика; пароль должен работать с HTTP Basic через TLS (если в вашей организации Basic отключён, включите его для служебного почтового ящика).<br> Не забудьте выбрать <strong>Microsoft Exchange (EWS)</strong> в списке «Протокол» выше.
source-form-help-google = <strong>Google Calendar</strong> — подключение через OAuth2, пароль не нужен.<br>
source-form-help-other = Укажите <strong>корневой DAV-адрес</strong> вашего сервера CalDAV, а не адрес отдельного календаря или публичную ссылку.<br> calrs сам найдёт ваши календари через PROPFIND (RFC 4791).

# Markdown editor toolbar, short labels (templates/team_form.html, templates/team_settings.html)

editor-bold-short = Жирный
editor-italic-short = Курсив
editor-link-short = Вставить ссылку

# Team creation (templates/team_form.html)

team-form-heading = Новая команда
team-form-name = Название команды
team-form-name-placeholder = Разработка
team-form-slug = Короткий адрес
team-form-slug-hint = (часть ссылки: латиница, цифры, дефис)
team-form-slug-pattern-title = Только строчные латинские буквы, цифры и дефисы
team-form-description = Описание
team-form-optional = (необязательно)
team-form-description-placeholder = Чем занимается эта команда…
team-form-description-help = Показывается на странице команды. Можно использовать **жирный**, *курсив* и [ссылки](url).
team-form-visibility = Видимость
team-form-public = Публичная
team-form-private = Закрытая
team-form-visibility-help = Для закрытой команды создаётся ссылка-приглашение, которой можно поделиться. Публичные команды видны на странице профиля команды.
team-form-members = Участники
team-form-members-help = Вы автоматически станете администратором команды. Добавьте пользователей или привяжите группы OIDC.
team-form-search-placeholder = Найти пользователей или группы…
team-form-search-users = Пользователи
team-form-search-groups = Группы OIDC
team-form-you = (вы)
team-form-submit = Создать команду

# Team settings (templates/team_settings.html)

team-settings-page-title = Настройки
team-settings-subtitle = Настройки команды: их могут менять администраторы команды.
team-settings-public-url = Публичный адрес
team-settings-public-url-help = Записаться по этой ссылке может любой.
team-settings-invite-link = Ссылка-приглашение
team-settings-invite-link-help = Поделитесь этой ссылкой, чтобы открыть доступ к странице записи этой закрытой команды.
team-settings-avatar = Изображение команды
team-settings-profile = Профиль
team-settings-description-placeholder = Расскажите об этой команде…
team-settings-description-help = Показывается на публичной странице записи команды. Можно использовать **жирный**, *курсив* и [ссылки](url).
team-settings-visibility-help = Публичные команды видны на странице профиля команды. Чтобы открыть закрытую команду, нужна ссылка-приглашение.
team-settings-members-help = Управляйте составом команды. Добавьте пользователей или привяжите группы OIDC для автоматической синхронизации.
team-settings-role-member = Участник
team-settings-role-admin = Администратор
team-settings-oidc-group = Группа OIDC
team-settings-remove = Убрать
team-settings-save = Сохранить изменения
team-settings-danger-zone = Опасная зона
team-settings-danger-help = Команда будет удалена навсегда. Типы встреч отвяжутся от неё, но не удалятся. Это действие нельзя отменить.
team-settings-delete = Удалить эту команду
team-settings-delete-confirm = Удалить команду «{ $name }»? Это действие нельзя отменить.

# Event type form (templates/event_type_form.html)

etf-heading-edit = Изменение типа встречи
etf-heading-new = Новый тип встречи
etf-disabled-banner = Этот тип встречи выключен, поэтому записаться на него пока нельзя.
etf-team = Команда
etf-team-hint = (необязательно — оставьте пустым, если тип встречи личный)
etf-team-personal = Личный
etf-scheduling-mode = Режим распределения
etf-mode-round-robin = По очереди — встреча назначается одному свободному участнику
etf-mode-collective = Совместная — нужны все участники
etf-scheduling-mode-help = В режиме «По очереди» встреча назначается одному свободному участнику (сначала наименее занятому). В режиме «Совместная» все участники должны быть свободны в одно и то же время.
etf-title = Название
etf-title-placeholder = Знакомство, 30 минут
etf-slug = Короткий адрес
etf-slug-placeholder = создаётся автоматически из названия
etf-description-placeholder = Короткий звонок, чтобы познакомиться и обсудить…
etf-description-help = Показывается на странице записи. Можно использовать **жирный**, *курсив* и [ссылки](url).
etf-location = Место
etf-location-link = Видеозвонок (постоянная ссылка)
etf-location-jitsi = Jitsi (комната создаётся автоматически)
etf-location-webhook = Вебхук (свой сервис)
etf-location-phone = Телефон
etf-location-in-person = Лично
etf-location-custom = Другое
etf-location-details = Подробности
etf-location-details-placeholder = https://meet.example.com/my-room
etf-pattern-placeholder = Оставьте пустым, чтобы использовать шаблон организации по умолчанию
etf-duration = Длительность (мин)
etf-slot-interval = Шаг времени (мин)
etf-slot-interval-placeholder = Равен длительности
etf-slot-interval-help = С каким шагом предлагать время. Если не заполнено, шаг равен длительности.
etf-required-members = Обязательные участники
etf-required-members-help = Время предлагается, только если все отмеченные участники свободны. Снимите отметку с тех, кого нужно исключить (их доступность не будет учитываться).
etf-member-priority = Приоритет участников
etf-member-priority-help = Новые записи в первую очередь получают свободные участники с более высоким приоритетом. При равном приоритете запись получает тот, у кого меньше недавних записей.
etf-member-timezone-title = Часовой пояс участника: в нём задаются его личные рабочие часы.
etf-priority-high = Высокий
etf-priority-medium = Средний
etf-priority-low = Низкий
etf-section-availability = Доступность
etf-timezone-help = Часы ниже указаны в этом часовом поясе. Для командных типов встреч выберите рабочий часовой пояс команды (он может не совпадать с вашим).
etf-reset-default = Вернуть мою доступность по умолчанию
etf-reset-default-title = Заменить эти часы доступностью по умолчанию из вашего профиля
etf-availability-prefilled = Часы взяты из вашей { $link }. Здесь их можно изменить для этого типа встречи.
etf-availability-prefilled-link = доступности по умолчанию
etf-section-buffers = Перерывы и сроки
etf-buffer-before = Перерыв до встречи (мин)
etf-buffer-after = Перерыв после встречи (мин)
etf-min-notice = Минимальный срок записи
etf-min-notice-help = За сколько времени до начала можно записаться.
etf-section-limits = Ограничения записи
etf-first-slot-only = Одно время в день
etf-first-slot-only-help = Показывать только самое раннее свободное время каждого дня.
etf-freq-limit = Ограничить частоту записи
etf-freq-limit-help = Сколько раз можно записаться на этот тип встречи за период.
etf-add-limit = Добавить ограничение
etf-section-options = Параметры записи
etf-requires-confirmation = Нужно подтверждение
etf-requires-confirmation-help = Новые записи будут ждать, пока вы не подтвердите их в кабинете.
etf-sms = SMS-уведомления
etf-sms-off = Выключены, номер не запрашивается
etf-sms-optional = Номер необязателен, гость может его оставить
etf-sms-required = Номер обязателен, гость должен его оставить
etf-sms-help = Помимо писем, гость получает SMS, когда запись подтверждена, перенесена или отменена, а также перед началом встречи. Если гость не укажет номер, SMS не будет. Нужен SMS-шлюз, настроенный в { $link }.
etf-admin-panel-link = панели администратора
etf-additional-guests = Дополнительные гости
etf-guests-none = Гости не могут приглашать других
etf-additional-guests-help = Гость сможет пригласить дополнительных участников встречи — они тоже получат приглашение в календарь.
etf-default-view = Вид календаря по умолчанию
etf-view-month = Месяц — сетка календаря со списком свободного времени
etf-view-week = Неделя — 7 колонок по дням со свободным временем
etf-view-column = Колонка — список дней со свободным временем
etf-view-week-short = неделя
etf-view-column-short = колонка
etf-default-view-help = Вид, который гости видят первым. Переключиться можно в любой момент.
etf-conflict-calendars = Календари для проверки занятости
etf-conflict-calendars-help = Выберите календари для проверки занятости. Если ничего не выбрано, проверяются все календари.
etf-no-resources = Общих ресурсов пока нет. Добавьте ресурс (демолабораторию, переговорную) в { $link }, чтобы привязать его к этому типу встречи.
etf-section-access = Доступ и уведомления
etf-visibility-public = Публичный — виден в вашем профиле
etf-visibility-internal = Внутренний — любой коллега может создавать ссылки-приглашения
etf-visibility-private = Закрытый — только по ссылке-приглашению
etf-visibility-help = Определяет, кто может видеть этот тип встречи и записываться на него.
etf-vis-internal = Внутренний
etf-reminder = Напоминание о встрече
etf-reminder-none = Без напоминания
etf-reminder-help = Перед встречей вам и гостю придёт письмо-напоминание.
etf-dynamic-group = Групповая ссылка
etf-dynamic-group-help = Создайте ссылку для разовой встречи с проверкой доступности — вашей и других пользователей.
etf-dynamic-group-search = Найти пользователя…
etf-dynamic-group-note = Показаны только пользователи, которые разрешили добавлять себя в групповые ссылки.
etf-dynamic-group-url = Адрес групповой ссылки
etf-watcher-teams = Команды-наблюдатели
etf-watcher-teams-help = Выбранные команды получают уведомления о новых записях. Их участники могут взять запись и присоединиться к встрече.
etf-save = Сохранить изменения
etf-create = Создать тип встречи
etf-js-loading = Загрузка…
etf-js-no-default = Доступность по умолчанию не задана
etf-js-reset-done = Сброшено
etf-js-error = Ошибка
etf-js-remove-limit = Убрать ограничение
etf-period-day = В день
etf-period-week = В неделю
etf-period-month = В месяц
etf-period-year = В год

# Event type form: runtime summary hints (templates/event_type_form.html)

# %1 and %2 are substituted client-side; the values are only known once a field is edited.

etf-hint-no-days = Дни не выбраны
etf-hint-every-day = Каждый день
etf-fmt-day-one = %1 день
etf-fmt-day-other = %1 дн.
etf-fmt-hours = %1 ч
etf-fmt-minutes = %1 мин
etf-hint-buffer-both = перерыв %1 мин до и %2 мин после
etf-hint-buffer-before = перерыв %1 мин до встречи
etf-hint-buffer-after = перерыв %1 мин после встречи
etf-hint-notice = мин. срок: %1
etf-hint-no-buffers = Без перерывов, записаться можно в любой момент
etf-hint-max = Не более %1
etf-hint-period-day = /день
etf-hint-period-week = /нед.
etf-hint-period-month = /мес.
etf-hint-period-year = /год
etf-hint-no-limits = Без ограничений
etf-hint-confirmation-required = Нужно подтверждение
etf-hint-auto-confirmed = Подтверждается автоматически
etf-hint-extra-guests-one = до %1 доп. гостя
etf-hint-extra-guests-other = доп. гостей: до %1
etf-hint-view = вид: %1
etf-hint-reminder = напоминание за %1
etf-hint-no-reminder = без напоминания

etf-guests-up-to =
    { $count ->
        [one] До { $count } дополнительного гостя
        [few] До { $count } дополнительных гостей
        [many] До { $count } дополнительных гостей
       *[other] До { $count } дополнительных гостей
    }

etf-reminder-hours =
    { $count ->
        [one] За { $count } час
        [few] За { $count } часа
        [many] За { $count } часов
       *[other] За { $count } часа
    }

etf-reminder-days =
    { $count ->
        [one] За { $count } день
        [few] За { $count } дня
        [many] За { $count } дней
       *[other] За { $count } дня
    }

# Event type form: preset banners and meeting-pattern help (templates/event_type_form.html)
# Literal braces are escaped as {"{"} because Fluent reads a bare { as a placeable.

etf-preset-public = Создаётся <strong>публичный</strong> тип встречи &mdash; записаться может любой, у кого есть ссылка.
etf-preset-private = Создаётся <strong>закрытый</strong> тип встречи &mdash; записаться могут только те, кого вы пригласили.
etf-preset-internal = Создаётся <strong>внутренний</strong> тип встречи &mdash; ссылкой для записи может поделиться любой коллега.
etf-preset-team = Создаётся <strong>командный</strong> тип встречи &mdash; записи распределяются между участниками команды.
etf-pattern-hint = Необязательный шаблон вместо стандартного. Переменные: <code>{"{"}username{"}"}</code>, <code>{"{"}event{"}"}</code>, <code>{"{"}date{"}"}</code>, <code>{"{"}random{"}"}</code>. Оставьте пустым, чтобы использовать шаблон организации, заданный администратором.
etf-pattern-random-warning = В этом шаблоне нет переменной <code>{"{"}random{"}"}</code>. Две записи этого типа встречи в один день попадут в одну и ту же комнату, и второй гость сможет зайти на встречу первого. Используйте постоянные комнаты, только если вам нужно именно это.
etf-webhook-hint = Ссылка на встречу для каждой записи запрашивается у вебхука, который администратор настроил в разделе «Панель администратора &rarr; Вебхук для встреч». Указывать адрес здесь не нужно.

# Admin panel (templates/admin.html)

admin-page-title = Панель администратора
admin-heading = Панель администратора
admin-action-refused = Действие не выполнено:
admin-error-last-admin = Это последний активный администратор. Сначала сделайте администратором другого пользователя или разблокируйте другого администратора.
admin-logo = Логотип компании
admin-logo-help = Показывается на публичных страницах записи. PNG, JPEG, GIF или WebP, не более 2 МБ.
admin-logo-refused = Логотип не сохранён. Загрузите изображение PNG, JPEG, GIF или WebP размером не более 2 МБ.
admin-favicon = Значок сайта
admin-favicon-help = Показывается во вкладках браузера и в закладках. Рекомендуется квадратный PNG размером от 32×32 пикселей и не более 512 КБ. Если значка нет, показывается краб calrs.
admin-favicon-refused = Значок сайта не сохранён. Загрузите изображение PNG, JPEG, GIF или WebP размером не более 512 КБ.
admin-upload-save-failed = Не удалось сохранить файл в каталоге данных сервера. Подробности — в журнале сервера.

# User and team avatar uploads (#307)
avatar-upload-refused = Изображение не сохранено. Загрузите файл PNG, JPEG, GIF или WebP размером не более 2 МБ.
avatar-upload-save-failed = Не удалось сохранить изображение на сервере. Попробуйте позже или попросите администратора проверить журнал сервера.
admin-company-link = Сайт компании
admin-company-link-help = Логотип на публичных страницах записи ведёт на этот адрес. Оставьте пустым, чтобы логотип не был ссылкой.
admin-theme = Тема
admin-theme-help = Выберите цветовую тему для всех страниц. Светлый и тёмный режим переключаются отдельно: каждая тема поддерживает оба режима.
admin-theme-default = По умолчанию
admin-theme-default-desc = Чистый синий
admin-theme-nord-desc = Арктический иней
admin-theme-dracula-desc = Тёмный фиолетовый
admin-theme-gruvbox-desc = Тёплое ретро
admin-theme-solarized-desc = Классика от Итана
admin-theme-tokyo-desc = Неоновый город
admin-theme-custom = Своя
admin-theme-custom-desc = Ваши цвета
admin-custom-colors = Свои цвета
admin-color-accent = Акцентный цвет
admin-color-accent-hover = Акцентный цвет при наведении
admin-color-bg = Фон
admin-color-surface = Фон карточек
admin-color-text = Текст
admin-save-theme = Сохранить тему
admin-users = Пользователи ({ $count })
admin-user-filter = Фильтр по имени или эл. почте…
admin-badge-admin = администратор
admin-badge-disabled = заблокирован
admin-impersonate = Войти от имени
admin-demote = Снять права администратора
admin-promote = Сделать администратором
admin-disable = Заблокировать
admin-enable = Разблокировать
admin-delete = Удалить
admin-no-users-match = Пользователи по фильтру не найдены.
admin-no-users = Пользователей пока нет.
admin-groups = Группы ({ $count })
admin-group-filter = Фильтр по названию группы…
admin-group-name = Название группы
admin-weight = вес:
admin-no-groups-match = Группы по фильтру не найдены.
admin-no-groups = Группы пока не синхронизированы. Они загружаются автоматически из вашего провайдера OIDC.
admin-auth-settings = Настройки входа
admin-registration-enabled = Разрешить регистрацию
admin-allowed-domains = Разрешённые домены эл. почты
admin-allowed-domains-hint = (через запятую; оставьте пустым, чтобы разрешить любые)
admin-save-auth = Сохранить настройки входа
admin-system-settings = Системные настройки
admin-base-url = Публичный адрес
admin-base-url-help = Публичный адрес этого сервера. Используется для перенаправлений OIDC и ссылок в письмах (подтверждение и отклонение, отмена, напоминания).
admin-private-hosts = Разрешённые внутренние хосты
admin-private-hosts-help = Имена хостов через запятую, которым разрешено указывать на частные или зарезервированные IP-адреса в источниках CalDAV/EWS (исключение из защиты от SSRF). Добавляйте только хосты, которыми управляете сами (например, сервер календарей в той же сети Docker). Оставьте пустым, чтобы защита действовала для всех хостов.
admin-german-formal = Обращение на немецком
admin-german-formal-du = Неформальное (du)
admin-german-formal-sie = Формальное (Sie)
admin-german-formal-help = Как страницы и письма обращаются к посетителям, чей браузер запрашивает немецкий язык. У пользователей, выбравших язык в разделе «Профиль и настройки», остаётся их выбор.
admin-unset-env = Чтобы изменить это значение здесь, удалите переменную окружения.
admin-save-system = Сохранить системные настройки
admin-status = Статус:
admin-status-enabled = включено
admin-status-disabled = выключено
admin-status-disabled-paren = (выключено)
admin-status-configured = настроено
admin-status-not-configured = не настроено
admin-via-environment = (задано в переменных окружения)
admin-issuer = Издатель:
admin-client-id = ID клиента:
admin-instance = Сервер:
admin-oidc-settings = Настройки OIDC
admin-oidc-enabled = Включить OIDC
admin-issuer-url = Адрес издателя (Issuer URL)
admin-client-id-label = ID клиента
admin-client-secret = Секретный ключ клиента
admin-keep-current-hint = (оставьте пустым, чтобы не менять)
admin-keep-current-set-hint = (оставьте пустым, чтобы не менять — сейчас задан)
admin-keep-unchanged = Оставьте пустым, чтобы не менять
admin-oidc-auto-register = Автоматически регистрировать новых пользователей при входе через OIDC
admin-save-oidc = Сохранить настройки OIDC
admin-google = Google Calendar (OAuth2)
admin-save-google = Сохранить настройки Google OAuth2
admin-captcha = Капча
admin-instance-url = Адрес сервера
admin-site-key = Ключ сайта
admin-secret = Секретный ключ
admin-widget-url = Адрес скрипта виджета
admin-widget-url-help = Укажите свой адрес, если CDN заблокирован. Изменения вступают в силу сразу после сохранения.
admin-wasm-url = Адрес WASM виджета
admin-wasm-url-help = Адрес, с которого виджет загружает решатель. Укажите WASM-файл, который отдаёт ваш сервер Cap, чтобы страницы записи не обращались к cdn.jsdelivr.net. Оставьте пустым для значения по умолчанию.
admin-captcha-wasm-url-invalid = Адрес WASM виджета должен быть полным, начинаться с http:// или https://.
admin-captcha-disable-help = Оставьте адрес сервера, ключ сайта и секретный ключ пустыми, чтобы выключить капчу на страницах записи.
admin-save-captcha = Сохранить настройки капчи
admin-resources = Ресурсы
admin-resources-help = Общие ресурсы для бронирования (демолаборатория, переговорные), занятость которых берётся из календаря по ICS-ссылке. Если ресурс привязан к типу встречи и занят, на это время записаться нельзя.
admin-resource-stats = Событий в кэше: { $events } &middot; Привязан к типам встреч: { $attached }
admin-never = никогда
admin-resource-sync-failed = (последняя попытка не удалась: { $error })
admin-writeback-enabled = Добавление в календарь: включено ({ $via })
admin-writeback-readonly = Добавление в календарь: нет, только для чтения
admin-teams-allowed = Доступен командам:
admin-teams-allowed-none = нет (только глобальные администраторы)
admin-sync-now = Синхронизировать
admin-test-write = Проверить добавление
admin-delete-resource-confirm = Удалить этот ресурс? Типы встреч, к которым он привязан, перестанут проверять его занятость.
admin-name = Название
admin-name-help = Оставьте пустым, чтобы взять название из ICS-ссылки.
admin-feed-url = ICS-ссылка (адрес публикации)
admin-feed-url-help = BlueMind: публичный или закрытый адрес календаря ресурса.
admin-caldav-url = Адрес коллекции CalDAV (для добавления в календарь)
admin-caldav-url-help = Необязательно. Для BlueMind определяется автоматически по ICS-ссылке.
admin-caldav-username = Имя пользователя CalDAV
admin-caldav-password = Пароль CalDAV
admin-resource-teams = Команды, которым доступен ресурс
admin-resource-teams-help = Администраторы этих команд могут привязывать ресурс к типам встреч своей команды. Пусто — только глобальные администраторы.
admin-no-teams = Команд пока нет.
admin-save-resource = Сохранить ресурс
admin-add-resource = Добавить ресурс
admin-jitsi = Jitsi (ссылки на встречи создаются автоматически)
admin-jitsi-help = Если в типе встречи выбрано место «Jitsi (комната создаётся автоматически)», calrs создаёт для каждой записи новую комнату: добавляет шаблон ниже к базовому адресу Jitsi. Обращаться к внешнему API не нужно.
admin-display-name = Отображаемое название
admin-jitsi-display-name-placeholder = например, Meet DYB
admin-jitsi-display-name-help = Показывается гостям при выборе времени и в форме записи. Если не указано, показывается «Видеозвонок».
admin-room-pattern = Шаблон названия комнаты
admin-jitsi-disable-help = Оставьте базовый адрес пустым, чтобы выключить автоматическое создание комнат Jitsi.
admin-save-jitsi = Сохранить настройки Jitsi
admin-meeting-webhook = Вебхук для встреч (свой сервис видеосвязи)
admin-webhook-url = Адрес вебхука
admin-webhook-display-name-placeholder = например, Zoom, Whereby, Custom Meet
admin-webhook-display-name-help = Показывается гостям вместо общей метки «Видеозвонок».
admin-authentication = Аутентификация
admin-auth-none = Нет
admin-auth-hmac = HMAC-SHA256 (заголовок X-Calrs-Signature)
admin-shared-secret = Общий секретный ключ
admin-webhook-disable-help = Оставьте адрес пустым, чтобы выключить вебхук для встреч.
admin-save-webhook = Сохранить настройки вебхука
admin-smtp = Настройки SMTP
admin-smtp-test-sent = Тестовое письмо отправлено.
admin-smtp-test-failed = Не удалось отправить тестовое письмо. Проверьте журнал сервера и настройки SMTP.
admin-smtp-env-error = Ошибка в настройках SMTP из переменных окружения:
admin-smtp-host = Сервер:
admin-smtp-from = Отправитель:
admin-smtp-enabled = SMTP включён
admin-host = Сервер
admin-port = Порт
admin-tls-mode = Режим TLS
admin-tls-starttls = STARTTLS (порт 587)
admin-tls-implicit = Неявный TLS (порт 465)
admin-tls-none = Без шифрования (только для локального MTA)
admin-smtp-username-hint = (оставьте пустым, если сервер не требует аутентификации)
admin-from-email = Адрес отправителя
admin-from-name = Имя отправителя
admin-save-smtp = Сохранить настройки SMTP
admin-send-test-email = Отправить тестовое письмо на адрес
admin-send-test-email-hint = (по умолчанию — адрес эл. почты вашего аккаунта)
admin-send-test-email-btn = Отправить тестовое письмо
admin-smtp-clear-confirm = Удалить настройки SMTP из базы данных?
admin-clear-db-config = Удалить настройки из базы данных
admin-sms = Настройки SMS
admin-sms-help = Необязательно. SMS отправляются только по записям на типы встреч, где включены «SMS-уведомления», и только если гость указал номер телефона.
admin-sms-test-sent = Тестовое сообщение отправлено.
admin-sms-test-checked = Учётные данные приняты.
admin-sms-test-error = SMS-шлюз отклонил запрос.
admin-sms-captcha-warning = Форма записи публичная, а номер получателя указывает гость, поэтому без капчи кто угодно сможет рассылать SMS за ваш счёт. Настройте капчу выше и ограничьте страны получателей в настройках самого шлюза.
admin-sms-sent-today = Отправлено сегодня:
admin-sms-of-cap = из { $cap }
admin-sms-config-error = Ошибка в настройках SMS:
admin-sms-gateway = Шлюз:
admin-sms-account = Аккаунт:
admin-sms-sender = Отправитель:
admin-sms-enabled = SMS включены
admin-sms-gateway-label = Шлюз
admin-required-on-switch = Обязательно при смене шлюза
admin-sms-docs = Документация API { $provider }
admin-sms-country = Код страны по умолчанию
admin-sms-country-hint = (используется, когда гость вводит местный номер телефона)
admin-sms-daily-cap = Дневной лимит
admin-sms-daily-cap-hint = (сообщений в день на весь сервер, 0 — без ограничений)
admin-sms-daily-cap-help = Когда лимит исчерпан, calrs перестаёт отправлять SMS, но письма уходят как обычно, поэтому записи не сорвутся, даже если бюджет на SMS закончился.
admin-save-sms = Сохранить настройки SMS
admin-send-test-sms = Отправить тестовое сообщение на номер
admin-send-test-sms-hint-check = (оставьте пустым, чтобы только проверить учётные данные)
admin-send-test-sms-hint-e164 = (в формате E.164)
admin-test-gateway = Проверить шлюз
admin-sms-clear-confirm = Удалить настройки SMS из базы данных?
admin-sms-allow-all = Разрешить всем пользователям включать SMS в своих типах встреч
admin-sms-allow-all-help = По умолчанию выключено: SMS расходуют баланс аккаунта, настроенного здесь, поэтому включать SMS в типе встречи могут только администраторы.
admin-save-policy = Сохранить правило
admin-page-of = Страница %1 из %2
admin-show-more-js = Показать ещё %1
admin-show-fewer = Свернуть

# Admin panel: strings carrying markup or literal braces (templates/admin.html)

admin-delete-user-confirm = Удалить пользователя { $email } навсегда?{"\u000A"}{"\u000A"}Будут удалены его профиль, аккаунт календаря, источники календарей, типы встреч и все данные, которые принадлежат только ему. Прошлые записи удаляются вместе с типами встреч.{"\u000A"}{"\u000A"}Для пользователей OIDC/SSO: если включена автоматическая регистрация, пользователь будет создан заново при следующем входе.{"\u000A"}{"\u000A"}Это действие нельзя отменить.
admin-system-settings-help = Публичный адрес и настройки сетевой безопасности. Их также можно задать переменными окружения <code>CALRS_BASE_URL</code> и <code>CALRS_ALLOW_PRIVATE_HOSTS</code>. Если переменная окружения задана, она <strong>имеет приоритет</strong> над значением ниже.
admin-set-by-env = — задано переменной окружения ({ $var }) вместо сохранённого значения
admin-google-help = Чтобы подключить Google Calendar, создайте учётные данные OAuth2 в <a href="https://console.cloud.google.com/apis/credentials" target="_blank" style="color: var(--accent);">Google Cloud Console</a>. Включите <strong>CalDAV API</strong> (проверка занятости и добавление записей в календарь) и <strong>Google Calendar API</strong> (ссылки Google Meet), затем добавьте { $redirect_uri } в разрешённые URI переадресации.
admin-room-pattern-help = Доступные подстановки: <code>{"{"}username{"}"}</code> (организатор), <code>{"{"}event{"}"}</code> (короткий адрес типа встречи), <code>{"{"}date{"}"}</code> (ГГГГММДД), <code>{"{"}random{"}"}</code> (8 символов). По умолчанию: { $default }.
admin-room-pattern-warning = Без <code>{"{"}random{"}"}</code> название комнаты предсказуемо: два гостя, записавшиеся на один тип встречи в один день, попадут в одну комнату и смогут видеть встречи друг друга. Постоянные комнаты допустимы (например, одна личная комната на организатора), но включайте этот режим, только если понимаете последствия.
admin-meeting-webhook-help = Если для типа встречи выбрано место «Вебхук (свой сервис)», при подтверждении записи calrs отправляет её данные POST-запросом на этот адрес и ожидает в ответ JSON <code>{"{"}"url": "https://..."{"}"}</code>.
admin-auth-hmac-help = С HMAC calrs подписывает исходное тело запроса и передаёт подпись в заголовке <code>X-Calrs-Signature: sha256=&lt;hex&gt;</code>.
admin-tls-none-warning = Выбирайте <strong>Без шифрования</strong>, только если почтовый сервер работает на этой же машине и не поддерживает STARTTLS или использует самоподписанный сертификат. Письма и учётные данные передаются по сети открытым текстом.
admin-smtp-env-error-help = Исправьте переменные окружения <code>CALRS_SMTP_*</code> или удалите их, чтобы управлять SMTP здесь, через базу данных.
admin-smtp-env-managed = Управляется через <strong>переменные окружения</strong> (они имеют приоритет над базой данных). Чтобы изменить настройки, отредактируйте переменные <code>CALRS_SMTP_*</code>. Чтобы управлять SMTP здесь, удалите их.
admin-smtp-env-help = Настройки можно задать и переменными окружения (они имеют приоритет над этой формой): <code>CALRS_SMTP_HOST</code>, <code>CALRS_SMTP_PORT</code>, <code>CALRS_SMTP_TLS_MODE</code> (<code>starttls</code>, <code>tls</code> или <code>none</code>), <code>CALRS_SMTP_USERNAME</code>, <code>CALRS_SMTP_PASSWORD</code>, <code>CALRS_SMTP_FROM_EMAIL</code>, <code>CALRS_SMTP_FROM_NAME</code>. Обязательны только <code>CALRS_SMTP_HOST</code> и <code>CALRS_SMTP_FROM_EMAIL</code>. Чтобы отправлять письма через локальный MTA без аутентификации, не указывайте имя пользователя и пароль.
admin-sms-env-error-help = Исправьте переменные окружения <code>CALRS_SMS_*</code> или удалите их, чтобы управлять SMS здесь, через базу данных.
admin-sms-env-managed = Управляется через <strong>переменные окружения</strong> (они имеют приоритет над базой данных). Чтобы изменить настройки, отредактируйте переменные <code>CALRS_SMS_*</code>. Чтобы управлять SMS здесь, удалите их.
admin-sms-env-help = Настройки можно задать и переменными окружения (они имеют приоритет над этой формой): <code>CALRS_SMS_PROVIDER</code>, <code>CALRS_SMS_API_KEY</code>, <code>CALRS_SMS_API_SECRET</code>, <code>CALRS_SMS_SENDER</code>, <code>CALRS_SMS_BASE_URL</code>, <code>CALRS_SMS_DAILY_CAP</code>, <code>CALRS_SMS_DEFAULT_COUNTRY_CODE</code>.
admin-sms-trial-warning = <strong>Включён пробный режим Twilio</strong> (<code>CALRS_SMS_TWILIO_TRIAL</code>). Гости получают стандартный шаблон Twilio <code>sms_appointment_reminders</code>, а не настоящее сообщение, и доставка возможна только на номера, подтверждённые в вашей консоли Twilio. Этот режим нужен только для проверки на пробном аккаунте. Удалите переменную, прежде чем принимать записи.

admin-show-more =
    { $count ->
        [one] Показать ещё { $count }
        [few] Показать ещё { $count }
        [many] Показать ещё { $count }
       *[other] Показать ещё { $count }
    }

# Calendar source form: backend picker (templates/source_form.html)

source-form-backend-help = Выберите протокол, который поддерживает ваш сервер. EWS — для Exchange 2019/2016/2013 на собственных серверах.

admin-sms-going-live = <strong>Перед запуском:</strong> ограничьте страны назначения в настройках шлюза (в Twilio это Geo Permissions), используйте предоплаченный счёт без автопополнения и не выключайте капчу. Вместе эти три меры ограничивают, во сколько вам может обойтись атака с накруткой SMS-трафика.

troubleshoot-heading = Диагностика доступности

# Host-side form validation errors (src/web/mod.rs)

form-error-team-name-slug-required = Укажите название и короткий адрес.
form-error-team-name-length = Название должно быть не длиннее 255 символов.
form-error-team-description-length = Описание должно быть не длиннее 5000 символов.
form-error-slug-charset = Короткий адрес может содержать только строчные латинские буквы, цифры и дефисы.
form-error-slug-reserved = Этот короткий адрес зарезервирован. Выберите другой.
form-error-team-slug-taken = Команда с таким коротким адресом уже существует.
form-error-title-required = Укажите название, чтобы создать короткий адрес.
form-error-event-type-slug-taken = Тип встречи с таким коротким адресом уже существует.
form-error-event-type-slug-taken-team = В этой команде уже есть тип встречи с таким коротким адресом.
form-error-event-type-title-required = Укажите название.
form-error-event-type-title-length = Название должно быть не длиннее 255 символов.
form-error-event-type-duration = Длительность должна быть целым числом минут от 1 до 1440 (24 часа).
form-error-event-type-number = Перерывы, минимальный срок, напоминание и число дополнительных гостей должны быть целыми числами не меньше 0.
form-error-event-type-horizon = Глубина записи должна быть целым числом дней не меньше 0. Оставьте поле пустым, чтобы не ограничивать её.
form-error-location-required = Укажите место (например, ссылку на видеозвонок, номер телефона или адрес).
form-error-bookable-range-order = Дата окончания не может быть раньше даты начала.
form-error-bookable-range-invalid = Введите даты в формате ГГГГ-ММ-ДД.
form-error-not-team-admin = Вы не администратор этой команды.
form-error-no-account = Аккаунт календаря не найден. Обратитесь к администратору.
form-error-all-fields-required = Заполните все поля.
form-error-encryption = Ошибка шифрования.
form-error-connection-failed = Не удалось подключиться: { $error }. Проверьте адрес и учётные данные или отметьте «Пропустить проверку подключения», чтобы всё равно сохранить.

# Settings page flash (src/web/mod.rs)

settings-saved = Настройки сохранены.

# Profile settings validation and flash messages (src/web/mod.rs)

settings-error-name-length = Имя должно содержать от 1 до 255 символов.
settings-error-username-length = Имя пользователя должно содержать не менее 2 символов.
settings-error-username-taken = Это имя пользователя уже занято.
settings-error-booking-email = Введите корректный адрес для записей.
settings-error-save-failed = Не удалось сохранить настройки.

# Host-facing error responses (src/web/mod.rs)

error-team-not-found-or-not-admin = Команда не найдена, или вы не администратор этой команды.
error-team-not-found = Команда не найдена.
error-event-type-not-found = Тип встречи не найден.
error-decrypt-failed = Не удалось расшифровать сохранённые учётные данные.
error-source-not-found = Источник не найден.
error-source-no-password = Для источника не сохранён пароль.
error-oauth-invalid-state = Неверный параметр state. Попробуйте ещё раз.
error-oauth-no-code = Код авторизации не получен.
error-oauth-not-configured = Google OAuth2 не настроен.
error-no-scheduling-account = Аккаунт календаря не найден.
error-private-event-type-not-found = Закрытый тип встречи не найден.
error-access-denied = Доступ запрещён.

# Guest booking-flow errors (src/web/mod.rs)

error-slot-unavailable = Это время уже занято.
error-slot-too-soon = Это время уже недоступно: до начала осталось слишком мало времени.
error-slot-beyond-horizon = Это время за пределами периода записи.
error-slot-not-yet-bookable = Запись на это время ещё не открыта.
error-invite-required = Для этого типа встречи нужна ссылка-приглашение.
error-invite-invalid = Недействительная ссылка-приглашение.
error-invite-expired = Срок действия этой ссылки-приглашения истёк.
error-invite-used = Эта ссылка-приглашение уже использована.
error-invalid-date = Неверная дата.
error-invalid-time = Неверное время.
error-invalid-date-format = Неверный формат даты.
error-invalid-time-format = Неверный формат времени.
error-too-many-bookings = Слишком много попыток записи. Попробуйте ещё раз через несколько минут.
error-too-many-requests = Слишком много запросов. Попробуйте позже.
error-no-members-available = На это время нет свободных участников команды.
error-dynamic-group-public-only = Групповые ссылки доступны только для публичных типов встреч.
error-user-not-found = Пользователь не найден.

# Booking action error page: titles (templates/booking_action_error.html)

bae-title-captcha = Проверка капчи не пройдена
bae-title-invalid-booking = Неверные данные записи
bae-title-unavailable = Запись сейчас недоступна
bae-title-cannot-approve = Не удалось подтвердить запись
bae-title-invalid-link = Недействительная ссылка
bae-title-invalid-or-expired = Ссылка недействительна или устарела
bae-title-booking-not-found = Запись не найдена
bae-title-already-approved = Запись уже подтверждена
bae-title-already-declined = Запись уже отклонена
bae-title-already-cancelled = Запись уже отменена
bae-title-booking-cancelled = Запись отменена
bae-title-booking-declined = Запись отклонена

# Booking action error page: bodies

bae-body-go-back = Вернитесь на предыдущую страницу и попробуйте ещё раз.
bae-body-unavailable = Организатор больше не принимает записи на эту дату. Выберите другую дату или зайдите позже.
bae-body-resource-gone = Нужный ресурс на это время уже недоступен. Попросите гостя выбрать другое время.
bae-body-no-claim-token = Ссылка неполная: в ней нет токена.
bae-body-claim-invalid = Эта ссылка больше не действует: взять по ней запись нельзя.
bae-body-booking-gone = Этой записи больше нет.
bae-body-decline-link-invalid = Ссылка для отклонения недействительна, устарела или запись уже обработана.
bae-body-cancel-link-invalid = Ссылка для отмены недействительна, устарела или запись уже отменена.
bae-body-cancel-link-invalid-short = Ссылка для отмены недействительна или устарела.
bae-body-reschedule-link-invalid = Ссылка для переноса недействительна, устарела или запись уже обработана.
bae-body-approval-link-invalid = Ссылка для подтверждения недействительна или устарела.
bae-body-already-approved = Эта запись уже подтверждена.
bae-body-already-declined = Эта запись уже отклонена.
bae-body-already-cancelled = Эта запись уже отменена.
bae-body-was-cancelled = Эта запись была отменена.
bae-body-declined-by-host = Запись отклонена организатором.

# Booking form validation (src/web/mod.rs)

validate-name-length = Имя должно содержать от 1 до 255 символов.
validate-email-length = Адрес эл. почты должен содержать от 1 до 255 символов.
validate-email-invalid = Введите корректный адрес эл. почты.
validate-notes-length = Комментарий должен быть не длиннее 5000 символов.
validate-date-too-far = Записаться можно не более чем на год вперёд.

# Additional guests and dynamic group links (src/web/mod.rs)

guests-not-allowed = Для этого типа встречи нельзя добавлять дополнительных гостей.
guests-too-many =
    { $max ->
        [one] Можно добавить не более { $max } дополнительного гостя.
        [few] Можно добавить не более { $max } дополнительных гостей.
        [many] Можно добавить не более { $max } дополнительных гостей.
       *[other] Можно добавить не более { $max } дополнительных гостей.
    }
guests-invalid-email = Неверный адрес дополнительного гостя: { $email }
dynamic-group-min-usernames = Для групповой ссылки нужны как минимум два имени пользователя.
dynamic-group-user-not-found = Пользователь «{ $username }» не найден.
dynamic-group-user-opted-out = У пользователя «{ $username }» выключены групповые ссылки.

error-slot-unavailable-member = Это время больше недоступно (занят участник: { $username }).

# Pending requests: slot hold, expiry and approval re-check (#234)

sms-expired = Запрос истёк: { $event }, { $date } в { $time } ({ $tz }). Организатор не ответил, выберите другое время.
email-pending-expired-status = Срок запроса истёк
email-pending-expired-headline = Срок вашего запроса на запись истёк: организатор не ответил вовремя. Выберите другое время.
email-pending-expired-action = Выбрать другое время
bae-body-host-busy = Организатор (или обязательный участник команды) в это время уже занят. Попросите гостя выбрать другое время.
bookings-error-generic = Что-то пошло не так.
bookings-error-no-bookable-dates = У этого типа встречи не осталось дат для записи, поэтому гость не сможет выбрать новое время. Сначала продлите период записи.
troubleshoot-detail-pending-request = { $title } (запрос ожидает подтверждения)
etf-pending-blocks-slot = Запросы, ожидающие ответа, занимают время
etf-pending-blocks-slot-help = Пока запрос ждёт вашего ответа, никто другой не может записаться на это время — ни на этот тип встречи, ни на другие ваши.
etf-pending-expiry = Срок ожидания запроса
etf-pending-expiry-help = Если вы не ответите на запрос до этого срока, он будет отклонён автоматически, а гостю предложат выбрать другое время. Срок истекает раньше, если до встречи остаётся так мало времени, что на неё уже нельзя записаться. В середине срока вы получите одно напоминание.
etf-pending-expiry-never = Без срока

etf-pending-expiry-hours =
    { $hours ->
        [one] Через { $hours } час
        [few] Через { $hours } часа
        [many] Через { $hours } часов
       *[other] Через { $hours } часа
    }

# Host, watcher and remaining guest emails (src/email.rs, #209)

email-footer-sent-by = Отправлено через
# Sender display name of guest emails: the host, then the instance name (#221).
email-sender-via = { $host } через { $instance }
email-date-at-time = { $date } в { $time }
email-detail-phone = Телефон:
email-detail-expires = Действует до:
email-detail-new-date = Новая дата:
email-detail-new-time = Новое время:
email-detail-previous = Прежнее время:
email-detail-requested = Запрошенное время:
email-detail-originally = Исходное время:
email-detail-organizer = Организатор:
email-detail-booked-by = Автор записи:
email-detail-invited-by = Приглашение от:
email-detail-message = Сообщение:
email-action-approve = Подтвердить
email-action-decline = Отклонить
email-action-pick-new-time = Выбрать новое время
email-host-approve-line = Подтвердить: { $url }
email-host-decline-line = Отклонить: { $url }
email-host-approval-dashboard = Войдите в кабинет, чтобы подтвердить или отклонить эту запись.
email-host-new-subject = Новая запись: { $event }, { $guest } ({ $date })
email-host-new-heading = Новая запись
email-host-new-message = К вам записался новый гость: { $guest }.
email-host-confirmed-subject = Подтверждено: { $event }, { $guest } ({ $date })
email-host-confirmed-heading = Запись подтверждена
email-host-confirmed-message = Вы подтвердили запись. Гость: { $guest }.
email-host-confirmed-footer = Событие добавлено в ваш календарь.
email-host-sync-failure-subject = Требуется действие: календарь не обновлён — { $event } ({ $date })
email-host-sync-failure-heading = Ваш календарь не обновлён
email-host-sync-failure-intro = Эта запись перенесена, но calrs не удалось обновить событие в Google Calendar. У гостя уже новое время, а в вашем календаре по-прежнему старое.
email-host-sync-failure-action = Откройте событие в Google Calendar и перенесите его на время, указанное выше. Именно перенесите существующее событие, а не создавайте новое, иначе ссылка Google Meet, которая уже есть у гостя, перестанет работать.
email-writeback-failure-subject = Требуется действие: запись не добавлена в календарь — { $event } ({ $date })
email-writeback-failure-heading = Запись не попала в ваш календарь
email-writeback-failure-intro = calrs не удалось добавить эту запись в ваш календарь даже после нескольких попыток. Запись подтверждена, приглашение у гостя есть.
email-writeback-failure-action = Добавьте событие в календарь вручную, чтобы не пропустить встречу, и проверьте источник календарей в calrs. calrs ещё несколько часов будет повторять попытки; больше писем об этой записи не придёт.
email-detail-error = Ошибка:
email-source-failure-subject = Требуется действие: источник календарей «{ $source }» не синхронизируется
email-source-failure-heading = Источник календарей перестал синхронизироваться
email-source-failure-intro = calrs уже больше часа не удаётся синхронизировать этот источник календарей. Страницы записи по-прежнему предлагают свободное время, но по данным последней успешной синхронизации, поэтому события, добавленные или перенесённые с тех пор, не учитываются.
email-source-failure-action = Проверьте источник на странице «Источники календарей»: обычно причина в истёкшем пароле или отозванном доступе. Следующее письмо об этом источнике придёт, только если он восстановится, а потом снова перестанет синхронизироваться.
email-detail-source = Источник календарей:
email-detail-failing-since = Перестал синхронизироваться:
email-host-reminder-subject = Напоминание: { $event }, { $guest } ({ $date })
email-host-reminder-heading = Предстоящая встреча
email-host-reminder-message = Напоминание: скоро встреча. Гость: { $guest }.
email-host-cancel-subject = Отменено: { $event }, { $guest } ({ $date })
email-host-cancel-message-by-host = Вы отменили эту запись.
email-host-cancel-message-by-guest = Запись отменена гостем: { $guest }.
email-host-approval-subject = Нужно ваше решение: { $event }, { $guest } ({ $date })
email-host-approval-heading = Нужно ваше решение
email-host-approval-message = { $guest } хочет записаться к вам.
email-host-approval-reminder-subject = Напоминание: { $guest } ждёт вашего ответа ({ $event }, { $date })
email-host-approval-reminder-heading = Напоминание: нужно ваше решение
email-host-approval-reminder-message = { $guest } всё ещё ждёт вашего ответа. Если вы не ответите до { $deadline }, запрос будет автоматически отклонён.
email-host-approval-footer = Ответить на запрос можно и в кабинете.
email-host-expired-subject = Срок запроса истёк: { $event }, { $guest } ({ $date })
email-host-expired-heading = Срок запроса истёк
email-host-expired-message = Гость: { $guest }. На запрос никто не ответил вовремя, поэтому он автоматически отклонён, а гостю предложено выбрать другое время.
email-host-expired-footer = Срок ожидания запросов можно изменить в настройках типа встречи.
email-host-reschedule-request-subject = Запрос на перенос: { $event }, { $guest } ({ $email })
email-host-reschedule-request-message = { $guest } хочет перенести запись.
email-watcher-claim-subject = Можно взять: { $event }, { $guest } ({ $date })
email-watcher-claim-message = Появилась новая запись, которую можно взять. Нажмите кнопку ниже, чтобы стать участником встречи.
email-watcher-claim-link = Взять запись: { $url }
email-watcher-claim-footer = Взять запись можно и в кабинете.
email-claim-confirmed-subject = Вы взяли запись: { $event }, { $guest } ({ $date })
email-claim-confirmed-footer = Вас добавят в участники этой встречи.
email-attendee-greeting = Здравствуйте!
email-attendee-message = Вас добавили в участники встречи.
email-attendee-rescheduled-message = Встреча, в которой вы участвуете, перенесена на другое время.
email-attendee-cancelled-message = Встреча, в которой вы участвуете, отменена.
email-attendee-cancelled-subject = Отменено: { $event }
email-original-booking-cancelled = Исходная запись тоже отменена.
email-pick-new-time-subject = Перенос: { $event }, выберите новое время
email-pick-new-time-message = { $host } просит перенести вашу запись. Выберите новое время.
email-pick-new-time-link = Выбрать новое время: { $url }
email-rescheduled-message = Организатор ({ $host }) перенёс вашу запись.
email-reschedule-approved-message = Организатор ({ $host }) подтвердил новое время вашей записи.
email-rescheduled-ics-attached-plain = Обновлённое приглашение в календарь во вложении.
email-rescheduled-ics-attached-html = К письму приложено обновлённое приглашение в календарь.
email-need-to-reschedule = Нужно перенести? { $url }
email-invite-subject = { $host } приглашает вас записаться: { $event }
email-invite-message = { $host } приглашает вас записаться: { $event }
email-invite-link = Чтобы выбрать время, перейдите по ссылке ниже:
email-invite-action = Выбрать время
email-cancel-reason-calendar-deleted = Организатор удалил событие из календаря.

# Embed snippet generator (templates/embed.html)
embed-page-title = Встраивание — { $title }
embed-heading = Встроить на сайт: { $title }
embed-subtitle = Создайте код для вставки, чтобы разместить этот тип встречи на своём сайте.
embed-warn-internal = Обратите внимание: этот тип встречи <strong>внутренний</strong>. Чтобы записаться через встроенную форму, посетителям нужна действующая ссылка-приглашение, а код для вставки её не содержит. Чтобы посетители могли записаться без приглашения, сделайте тип встречи публичным.
embed-warn-private = Обратите внимание: этот тип встречи <strong>закрытый</strong>. Чтобы записаться через встроенную форму, посетителям нужна действующая ссылка-приглашение, а код для вставки её не содержит. Чтобы посетители могли записаться без приглашения, сделайте тип встречи публичным.
embed-warn-private-team = Обратите внимание: эта команда <strong>закрытая</strong>. Без ссылки-приглашения команды посетители не смогут записаться через встроенную форму, а код для вставки её не содержит. Чтобы посетители могли записаться без приглашения, сделайте команду публичной.
embed-need-username = Чтобы встроить этот тип встречи, сначала задайте <strong>имя пользователя</strong>: ссылка для встраивания — { $path }. { $link } и вернитесь сюда.
embed-need-username-link = Задайте имя пользователя в настройках
embed-username-placeholder = имя_пользователя
embed-tab-inline = Блок на странице
embed-tab-floating = Плавающая кнопка
embed-tab-element = По нажатию на элемент
embed-layout = Вид
embed-theme = Тема
embed-theme-auto = Авто (как у посетителя)
embed-brand-color = Фирменный цвет
embed-use-org-color = Использовать цвет организации
embed-button-text = Текст кнопки
embed-button-default = Записаться на встречу
embed-button-position = Положение кнопки
embed-pos-bottom-right = Справа внизу
embed-pos-bottom-left = Слева внизу
embed-pos-top-right = Справа вверху
embed-pos-top-left = Слева вверху
embed-button-colors = Цвета кнопки
embed-button-fg-placeholder = текст
embed-button-colors-hint = Фон / текст
embed-show-icon = Показывать значок календаря
embed-element-help = Вставьте код на сайт один раз, затем добавьте показанные data-атрибуты к любому элементу (кнопке, ссылке, div), по нажатию на который должно открываться окно записи.
embed-snippet = Код для вставки
embed-loading = Загрузка…
embed-preview = Предпросмотр
embed-preview-back = назад к предпросмотру
embed-preview-floating-hint = Нажмите на кнопку, чтобы открыть окно записи
embed-preview-element-hint = Примеры элементов. На своём сайте добавьте data-атрибуты из кода к любому элементу. Нажмите на любой из них, чтобы открыть окно записи:
embed-preview-text-link = или нажмите на эту текстовую ссылку
embed-snippet-attr-comment = Добавьте эти атрибуты к любому элементу страницы, на который можно нажать:

# Leftovers in partly translated pages (host reschedule, impersonation banner, slots, team settings)
host-resched-when = { $date }, { $start }–{ $end }
base-impersonating = Вы вошли от имени пользователя: { $name }
slots-rescheduling-banner = Перенос: { $title } (сейчас: { $date } в { $time })
slots-invite-required-title = Для записи нужна ссылка-приглашение
team-settings-slug-url = Используется в адресе команды: { $url }
team-event-type-no-teams = Команд пока нет.
team-event-type-not-team-admin = Вы не администратор ни одной команды.

# Google Calendar connection errors (src/web/sources.rs)
google-error-not-configured = Интеграция с Google Calendar не настроена. Попросите администратора указать учётные данные Google OAuth2 в панели администратора.
google-error-no-base-url = Публичный адрес сервера не настроен. Прежде чем подключаться через OAuth2, укажите его в переменной окружения CALRS_BASE_URL или в панели администратора, в разделе «Системные настройки».
google-error-authorization = Не удалось авторизоваться в Google: { $error }
google-error-token-exchange = Не удалось получить токен: { $error }
google-error-account-email = Не удалось получить адрес эл. почты аккаунта Google: { $error }

# Admin panel notices (src/web/admin.rs)
admin-resource-feed-invalid = Неверная ICS-ссылка: { $error }
admin-resource-feed-check-failed = Не удалось проверить ICS-ссылку: { $error }
admin-resource-caldav-invalid = Неверный адрес CalDAV: { $error }
admin-resource-credentials-failed = Не удалось сохранить учётные данные: { $error }
admin-resource-added = Ресурс «{ $name }» добавлен. Событий в кэше: { $count }.
admin-resource-updated = Ресурс обновлён. Событий в кэше: { $count }.
admin-resource-deleted = Ресурс удалён.
admin-resource-not-found = Ресурс не найден.
admin-resource-synced = Синхронизация завершена. Событий в кэше: { $count }.
admin-resource-sync-error = Не удалось синхронизировать: { $error }
admin-resource-no-caldav-url = Адрес CalDAV не указан, и его не удалось определить по ICS-ссылке.
admin-resource-no-write-credentials = Нет учётных данных для добавления событий: служебной учётной записи нет, и ни у одного участника, разрешившего доступ, нет подходящего источника календарей.
admin-resource-write-passed = Проверка добавления событий пройдена: { $user } (создание: { $created }, удаление: { $cleaned }).
admin-resource-write-verified = проверено
admin-resource-write-unverified = не проверено
admin-resource-write-check-calendar = проверьте календарь ресурса
admin-resource-write-failed = Проверка добавления событий не пройдена. Последняя ошибка: { $error }
admin-base-url-invalid = Публичный адрес должен быть абсолютным URL с http(s), например https://cal.example.com
admin-company-link-invalid = Ссылка на компанию должна начинаться с http:// или https://
admin-smtp-host-required = Укажите сервер SMTP.
admin-smtp-from-invalid = Укажите корректный адрес эл. почты отправителя.
admin-smtp-port-invalid = Порт SMTP должен быть числом от 1 до 65535.
admin-smtp-tls-invalid = Режим TLS должен быть «starttls», «tls» или «none».
admin-smtp-password-required = При первой настройке SMTP нужно указать пароль.
admin-sms-switch-needs-secret = Чтобы сменить SMS-шлюз, введите учётные данные этого шлюза.
admin-sms-settings-invalid = Настройки SMS: { $error }.
admin-sms-sender-format = Числовой отправитель SMS указывается в международном формате, например +971501234567.
admin-sms-not-configured = SMS не настроены.
admin-sms-load-failed = Не удалось загрузить настройки SMS.
admin-sms-check-accepted = Шлюз принял учётные данные.
admin-sms-bad-number = Этот номер телефона не подходит.
admin-sms-test-body = Тестовое сообщение calrs. SMS-шлюз работает.
admin-sms-test-accepted = Шлюз принял тестовое сообщение.

## Local multi-factor authentication

mfa-title = Двухфакторная аутентификация
mfa-invalid = Проверка не пройдена. Проверьте пароль или код. Каждый код действует только один раз; при необходимости дождитесь нового. После слишком большого числа попыток подождите 15 минут. Если время настройки истекло, начните заново.
mfa-restart = Вернуться ко входу
mfa-manage = Настройка двухфакторной аутентификации
mfa-recovery-help = Сохраните эти резервные коды в надёжном месте. Они показываются только один раз. Каждый код можно использовать один раз вместо кода из приложения-аутентификатора. Новые резервные коды заменяют все предыдущие.
mfa-continue = Перейти в кабинет
mfa-enabled = Двухфакторная аутентификация включена.
mfa-disabled = Двухфакторная аутентификация выключена.
mfa-required-help = Администратор сделал двухфакторную аутентификацию обязательной для локальных аккаунтов. Выключить её нельзя.
mfa-code = Код из приложения-аутентификатора или резервный код
mfa-regenerate = Создать новые резервные коды
mfa-disable = Выключить двухфакторную аутентификацию
mfa-enable = Включить двухфакторную аутентификацию
mfa-changes-help = Для изменений нужен пароль. Чтобы выключить двухфакторную аутентификацию или заменить резервные коды, нужен ещё и код; при этом завершатся все остальные ваши сеансы.
mfa-back = Вернуться в настройки
mfa-setup-help = Отсканируйте этот QR-код приложением-аутентификатором или введите ключ ниже вручную. Чтобы завершить настройку, введите шестизначный код. На настройку даётся 10 минут.
mfa-qr-alt = QR-код для настройки приложения-аутентификатора
mfa-login-help = Введите код из приложения-аутентификатора или один из сохранённых резервных кодов. На вход даётся 10 минут.
mfa-totp-code = Шестизначный код из приложения-аутентификатора
mfa-verify = Подтвердить
mfa-policy-help = Эта политика действует для локальных аккаунтов. При входе через SSO используется двухфакторная аутентификация поставщика удостоверений. Прежде чем менять политику, локальные администраторы должны настроить двухфакторную аутентификацию у себя. Если сделать её обязательной, локальные сеансы без второго фактора будут завершены, а при следующем входе её нужно будет настроить.
mfa-policy = Двухфакторная аутентификация для локальных аккаунтов
mfa-optional = Необязательна
mfa-required = Обязательна
mfa-save-policy = Сохранить политику

# First day of the week (#330)
week-start-label = Первый день недели
week-start-inherit-site = По умолчанию для сервера
week-start-inherit-owner = Как у владельца

## Booking directory and personal favorites

directory-title = Каталог встреч
directory-help = Найдите ссылку на встречу коллеги или команды. Отмечайте звёздочкой встречи, которыми часто делитесь, чтобы они были под рукой в кабинете.
directory-search = Поиск по имени, команде или названию встречи
directory-find = Найти
directory-favorites-only = Только избранное
directory-favorites = Избранные встречи
directory-browse = Открыть каталог
directory-generate = Создать ссылку-приглашение
directory-invite-help = Одноразовое приглашение, действует 7 дней.
directory-add = Добавить «{ $title }» в избранное
directory-remove = Убрать «{ $title }» из избранного
directory-added = Добавлено в избранное.
directory-removed = Убрано из избранного.
directory-error = Не удалось выполнить действие. Обновите страницу и попробуйте ещё раз.
directory-manual-copy = Скопируйте ссылку вручную:
directory-empty = Встречи не найдены. Попробуйте изменить запрос.
directory-favorites-empty = Избранных встреч нет. Отметьте встречи звёздочкой в каталоге, и они появятся здесь.
directory-with-hosts = Организаторы: { $names }

## ilia fork: TON payments (docs/ton.md)

# Price on the slot and booking pages
ton-slots-price = Стоимость: ${ $usd }
ton-slots-price-approx = (≈ { $ton } TON)
ton-slots-price-note = оплата в TON после выбора времени

# Event type form
ton-etf-section = Оплата в TON
ton-etf-off-note = Оплата в TON на этом сервере не настроена. Записаться на платный тип встречи нельзя, пока её не настроят.
ton-etf-enabled = Платная запись
ton-etf-enabled-help = Гость оплачивает встречу в TON сразу после выбора времени. Пока идёт оплата, время закреплено за гостем; неоплаченная запись отменяется автоматически.
ton-etf-price = Стоимость
ton-etf-price-help = В USD: от $1 до $10000; сумма в TON рассчитывается по курсу в момент записи.
ton-etf-window = Время на оплату (мин)
ton-etf-window-help = От 10 до 240. Оставьте пустым, чтобы использовать значение сервера ({ $minutes } мин).
ton-form-error-price = Укажите стоимость в долларах США от 1 до 10000, не более двух знаков после точки.
ton-form-error-window = Время на оплату — от 10 до 240 минут.
ton-form-error-team = Командные типы встреч не могут быть платными.

# Booking error pages
ton-bae-title-unavailable = На это время записаться нельзя
ton-bae-body-unavailable = Оплата временно недоступна, поэтому записаться на это время сейчас нельзя. Попробуйте ещё раз через несколько минут.
ton-bae-body-too-late = До начала встречи слишком мало времени, чтобы успеть её оплатить. Выберите время позже.
ton-bae-body-already-open = У вас уже есть неоплаченная запись на этот адрес эл. почты. Мы можем прислать её ссылки: по ним запись можно оплатить или отменить, чтобы освободить время, а затем записаться снова.
ton-bae-body-personal-only = На эту встречу можно записаться только на личной странице организатора.
ton-bae-title-unpaid = Запись не оплачена
ton-bae-body-unpaid = Эта запись не оплачена, поэтому подтвердить её нельзя. Чтобы подтвердить её бесплатно, нажмите «Подтвердить без оплаты» на странице записей в кабинете.
ton-bae-body-reschedule-unpaid = Эта запись не оплачена, поэтому перенести её нельзя. Оплатите её или отмените и запишитесь на другое время.
ton-bae-body-pay-link-invalid = Эта ссылка на оплату недействительна.

# Payment page
ton-pay-page-title = Оплата: { $title }
ton-pay-held-until = Время за вами до { $time } ({ $zone }).
ton-pay-left = осталось %1
ton-pay-amount-label = К оплате
ton-pay-rate = ≈ ${ $usd } · 1 TON = ${ $rate } ({ $sources }, { $time } UTC)
ton-pay-open-wallet = Открыть кошелёк
ton-pay-open-tonkeeper = Открыть в Tonkeeper
ton-pay-qr-alt = QR-код с данными для оплаты
ton-pay-scan = Или отсканируйте этот код в приложении кошелька.
ton-pay-address = Адрес
ton-pay-comment = Комментарий
ton-pay-copy = Скопировать
ton-pay-copied = Скопировано
ton-pay-warn-comment = Комментарий к переводу обязателен: по нему мы узнаём ваш платёж.
ton-pay-warn-encrypt = Не шифруйте комментарий.
ton-pay-warn-token = Платите в TON, а не токенами.
ton-pay-warn-exchange = Платите с биржи? Укажите код выше в поле «Memo» (комментарий).
ton-pay-warn-deadline = Отправьте платёж до { $time }. Платёж, отправленный позже, может не засчитаться.
ton-pay-check = Платёж отправлен, проверить
ton-pay-email = Прислать ссылку на эл. почту
ton-pay-emailed = Мы отправили эту ссылку на адрес эл. почты, указанный при записи.
ton-pay-email-unavailable = Не удалось отправить ссылку. Не закрывайте эту страницу или сохраните её адрес.
ton-pay-checking-now = Проверяем ваш платёж. Страница обновится сама.
ton-pay-state-waiting = Ждём вашу оплату.
ton-pay-state-seen = Ваш платёж виден в сети. Ждём его окончательного подтверждения — обычно это несколько секунд.
ton-pay-state-partial = Получено { $paid } TON. Доплатите оставшиеся { $rest } TON с тем же комментарием.
ton-pay-state-checking = Время на оплату вышло. Если вы уже отправили платёж, мы его проверяем. Страница обновится сама.
ton-pay-state-paid = Оплата получена. Подтверждаем запись…
ton-pay-state-confirmed = Оплачено. Запись подтверждена, письмо с подтверждением уже в пути.
ton-pay-state-waived = Оплата для этой записи не нужна. Организатор подтверждает её.
ton-pay-state-busy = Оплата получена, но это время уже занято. Мы с вами свяжемся.
ton-pay-state-expired = Срок оплаты истёк, и это время освобождено. Если вы всё же отправили платёж, напишите нам с адреса эл. почты, указанного при записи, и мы вернём деньги.
ton-pay-state-late = Ваш платёж пришёл после срока, поэтому запись не подтверждена. Мы свяжемся с вами насчёт возврата. Вы также можете написать нам с адреса эл. почты, указанного при записи.
ton-pay-state-closed = Эта запись больше не действует.
ton-pay-state-revoked = Этот адрес для оплаты больше не используется. Ничего на него не отправляйте и запишитесь заново.
ton-pay-rebook = Записаться на другое время

# Leaving the embedded widget
ton-exit-title = Время за вами
ton-exit-body = Время за вами до { $time } ({ $zone }). Оплатите встречу, чтобы подтвердить запись.
ton-exit-go = Перейти к оплате

# Dashboard
ton-dash-badge-awaiting = ждёт оплаты · { $amount } TON · до { $time }
ton-dash-badge-partial = оплачена частично · { $paid } из { $amount } TON · до { $time }
ton-dash-badge-paid = оплачена · { $amount } TON
ton-dash-badge-overpaid = оплачена · переплата { $excess } TON
ton-dash-badge-busy = оплачена, время занято
ton-dash-badge-confirm-failed = оплачена, не подтверждена · подтвердите вручную
ton-dash-badge-refund = оплачено { $amount } TON · нужен возврат
ton-dash-badge-waived = подтверждена без оплаты
ton-dash-badge-unpaid = не оплачена
ton-dash-waive = Подтвердить без оплаты
ton-dash-error-unpaid = Эта запись не оплачена, поэтому её нельзя подтвердить или перенести. Нажмите «Подтвердить без оплаты», чтобы подтвердить её бесплатно, или отклоните её.
ton-dash-error-waive-failed = Не удалось подтвердить запись без оплаты: она больше не ждёт оплаты.
ton-waive-page-title = Подтверждение без оплаты
ton-waive-heading = Подтвердить запись без оплаты?
ton-waive-body = Эта запись НЕ оплачена ({ $amount } TON, срок — до { $time }). Если вы её подтвердите, гость получит встречу бесплатно, а платёж, который придёт позже, придётся вернуть вручную.
ton-waive-submit = Подтвердить бесплатно
ton-waive-back = Вернуться к записям

# Guest emails
ton-email-link-subject = Завершите запись: { $event }
ton-email-link-message = Вы начали запись на встречу «{ $event }» (организатор: { $host }). Завершите запись на странице по ссылке ниже, пока не истекло время.
ton-email-link-action = Завершить запись
ton-email-link-line = Завершить запись: { $url }
ton-email-expired-subject = Запись не состоялась: { $event }, { $date }
ton-email-expired-message = Оплата встречи «{ $event }» не поступила вовремя, поэтому время освобождено. Вы можете выбрать другое время.
ton-email-expired-partial = Мы получили часть оплаты и свяжемся с вами насчёт возврата.
ton-email-rebook-line = Выбрать другое время: { $url }

# Owner emails
ton-email-row-amount = Сумма:
ton-email-row-paid = Получено:
ton-email-row-excess = Переплата:
ton-email-row-transfer = Перевод:
ton-email-row-sender = Отправитель:
ton-email-row-tx = Транзакция:
ton-email-row-detail = Подробности:
ton-email-amount-value = { $amount } TON ≈ ${ $usd } (1 TON = ${ $rate })
ton-email-sender-wallet = { $address } (кошелёк)
ton-email-sender-other = { $address } (не кошелёк: биржа или контракт)
ton-email-dashboard-line = Записи: { $url }
ton-email-dashboard-action = Открыть записи
ton-email-refund-rule = Возврат делается вручную из вашего кошелька: на адрес отправителя, если это кошелёк; иначе — только на адрес, который гость попросит в письме с адреса эл. почты, указанного при записи, и только если в письме есть его ссылка на оплату или на отмену. Код записи, сумма и транзакция публичны и ничего не доказывают.
ton-email-received-subject = Оплата получена: { $event }, { $guest } ({ $date })
ton-email-received-heading = Оплата получена
ton-email-received-message = Оплачено { $amount } TON. Гость: { $guest }. Запись подтверждена.
ton-email-alert-subject = Платёж в TON требует внимания: { $what }
ton-email-alert-heading = Платёж требует вашего внимания
ton-alert-partial = частичная оплата
ton-alert-partial-body = Пришла часть суммы. Запись не подтверждается, пока не придёт остаток; если он не придёт, запись отменится в срок, а полученную сумму нужно будет вернуть.
ton-alert-overpaid = переплата
ton-alert-overpaid-body = Пришёл платёж за уже оплаченную запись. Верните его, если сумма того стоит.
ton-alert-late = поздний платёж
ton-alert-late-body = Платёж пришёл после срока. Запись уже снята, подтвердить её больше нельзя. Верните платёж или попросите гостя записаться заново и подтвердите новую запись без оплаты.
ton-alert-unmatched = неопознанный платёж
ton-alert-unmatched-body = Пришёл платёж без известного кода записи. Найдите запись, к которой он относится, или верните его.
ton-alert-failed = неудавшийся перевод
ton-alert-failed-body = Перевод на адрес для оплаты завершился в сети ошибкой. Деньги могли остаться на вашем кошельке: проверьте транзакцию в обозревателе.
ton-alert-not-text = перевод без читаемого комментария
ton-alert-not-text-body = Пришёл перевод с зашифрованным комментарием, токеном или другими данными. Такие переводы никогда не засчитываются автоматически: проверьте его вручную.
ton-alert-other = необычный перевод
ton-alert-other-body = Перевод не удалось обработать автоматически. Проверьте его в журнале платежей.
ton-email-problem-subject = Проблема с оплатой в TON: { $event } ({ $date })
ton-email-problem-urgent-subject = Срочно: нужно решение по оплаченной записи: { $event } ({ $date })
ton-email-problem-heading = Оплаченная запись требует вашего внимания
ton-problem-host-busy = Гость оплатил встречу, но это время уже занято (событие в календаре или другая запись). Время удерживается до начала встречи. Попросите гостя выбрать новое время, подтвердите запись вручную или отклоните её и верните деньги.
ton-problem-partial = Оплачена только часть суммы, после чего запись истекла, была отменена или отклонена либо подтверждена без оплаты. Верните полученную сумму, если вы не договорились с гостем иначе.
ton-problem-overpaid = Запись оплачена и подтверждена, но пришло больше, чем нужно. Верните излишек, если сумма того стоит.
ton-problem-refund-due = Запись была оплачена, а затем отменена или отклонена. Верните платёж.
ton-problem-confirmed-unpaid = Неоплаченная запись оказалась подтверждённой. Такого быть не должно: проверьте, как её подтвердили.
ton-problem-address-revoked = Адрес для оплаты этой записи отозван, запись будет отменена.
ton-email-health-subject = Оплата в TON требует внимания ({ $kind })
ton-email-health-heading = Оплата в TON требует внимания
ton-health-indexer-down = Фоновая проверка платежей не может связаться с индексатором. Платежи не проверяются, а новые платные записи не принимаются, пока связь не восстановится.
ton-health-auth-failed = Индексатор отклонил API-ключ (он неверен или от другой сети). Проверьте CALRS_TON_API_KEY.
ton-health-wrong-network = Индексатор отвечает за другую сеть, а не за настроенную. Пока это не исправлено, платежи не засчитываются и записи не истекают.
ton-health-wallet-inactive = Кошелёк для приёма оплаты не активен. Платные записи не принимаются, пока с него не будет сделан хотя бы один исходящий перевод.
ton-health-lagging = Проверка платежей не успевает за переводами на адрес (возможно, спам). Неоплаченные записи не истекают, пока она не догонит.
ton-health-stuck-finality = Один перевод долго не получает окончательного подтверждения. Проверка ждёт его, прежде чем читать дальше.
ton-health-watcher-crashed = Фоновая проверка платежей несколько раз аварийно завершилась и перезапускается. Посмотрите журнал сервера.
ton-health-disabled-with-open = Оплата в TON выключена, а неоплаченные записи открыты. Их время освободится в срок.
ton-health-other-network-open = Есть открытые записи в другой сети TON, а не в настроенной. Они отменятся в срок.
ton-health-rate-unavailable = Нет подходящего курса TON (источники недоступны, расходятся или курс вне вашего коридора). Платные записи не принимаются, пока курс не появится.
ton-health-capacity = Достигнут предел открытых или дневных неоплаченных записей. Новые платные записи пока не принимаются.
ton-health-other = Фоновая проверка платежей сообщила о проблеме. Посмотрите журнал сервера.

# TON follow-ups: releases, refunds, contact, resend
ton-resend-submit = Прислать ссылки на эл. почту
ton-resend-title = Проверьте почту
ton-resend-done = Если на этот адрес эл. почты есть неоплаченная запись, мы отправили на него ссылки на оплату и отмену.
ton-pay-state-taken = Это время заняли до того, как запись была оплачена, поэтому запись отменена. Платить ничего не нужно. Выберите, пожалуйста, другое время.
ton-pay-state-refund = Запись завершилась без подтверждения. Оплаченные { $paid } TON будут возвращены, организатор свяжется с вами.
ton-pay-contact = Вопросы можно задать организатору: { $host }, { $email }.
ton-pay-copy-address = Скопировать адрес
ton-pay-copy-comment = Скопировать комментарий
ton-pay-copy-manual = Скопировать не удалось. Текст выделен — скопируйте его вручную.
ton-email-row-pay-by = Оплатить до:
ton-email-taken-message = Выбранное вами время встречи «{ $event }» заняли до того, как запись была оплачена, поэтому запись отменена. Платить ничего не нужно. Вы можете выбрать другое время.
ton-email-refund-message = Мы получили оплату встречи «{ $event }», но подтвердить это время не удалось, поэтому запись отменена. { $host } вернёт деньги и свяжется с вами.
ton-problem-host-busy-unpaid = Время этой неоплаченной записи оказалось занято (поверх неё подтвердили другую заявку или в календаре появилось событие), поэтому запись отменена до оплаты. Если что-то уже поступило, верните это.
ton-problem-confirm-failed = Гость оплатил встречу, но подтвердить запись автоматически не удалось после нескольких попыток (это ошибка, а не занятость организатора; подробности в журнале сервера). Время удерживается до начала встречи. Подтвердите запись вручную или отклоните её и верните деньги.
ton-health-capacity-near = Новая платная запись заняла последнее место среди открытых счетов или исчерпала дневной лимит. Следующие записи не будут приниматься, пока открытые не оплатят или не снимут. Если это не настоящий спрос, возможно, кто-то занимает время, не оплачивая его.
ton-health-payment-alerts-held = Пришли ещё переводы по записям, о которых вы уже получили письмо в течение часа. Они сохранены без отдельных писем — проверьте записи о платежах.
ton-waive-body-partial = Эта запись оплачена лишь частично (получено { $paid } из { $amount } TON, срок до { $time }). Если вы её подтвердите, гость получит встречу без доплаты оставшихся { $rest } TON. Верните полученную сумму, если вы не договорились иначе.

## Telegram (ilia fork)

tg-when = { $date }, { $time } ({ $tz })
tg-in-minutes =
    { $count ->
        [one] через { $count } минуту
        [few] через { $count } минуты
        [many] через { $count } минут
       *[other] через { $count } минуты
    }
tg-in-hours =
    { $count ->
        [one] через { $count } час
        [few] через { $count } часа
        [many] через { $count } часов
       *[other] через { $count } часа
    }
tg-in-days =
    { $count ->
        [one] через { $count } день
        [few] через { $count } дня
        [many] через { $count } дней
       *[other] через { $count } дня
    }
tg-reason = Причина: { $reason }
tg-link-line = { $label }: { $url }
tg-guest-confirmed = Запись подтверждена: { $event }, { $when }. Организатор: { $host }.
tg-guest-pending = Запрос отправлен: { $event }, { $when }. Ответ организатора придёт сюда.
tg-guest-approved = Ваш запрос подтверждён: { $event }, { $when }.
tg-guest-reschedule-approved = Новое время подтверждено: { $event }, { $when }.
tg-guest-declined = Запрос отклонён: { $event }, { $when }.
tg-guest-cancelled-by-host = Встреча отменена организатором: { $event }, { $when }.
tg-guest-cancelled-by-guest = Вы отменили запись: { $event }, { $when }.
tg-guest-rescheduled = Запись перенесена: { $event }, теперь { $when }.
tg-guest-reschedule-pending = Запрошено новое время: { $event }, { $when }. Ответ организатора придёт сюда.
tg-guest-reschedule-requested = Организатор просит выбрать другое время для встречи «{ $event }». Было: { $when }.
tg-guest-reminder = Напоминание: { $event } начнётся { $in }, { $when }.
tg-guest-expired = Запрос истёк: { $event }, { $when }. Организатор не успел ответить, выберите, пожалуйста, другое время.
tg-button-cancel = Отменить запись
tg-button-reschedule = Перенести
tg-button-calendar = Добавить в календарь
tg-button-pick-time = Выбрать новое время
tg-button-rebook = Записаться снова
tg-button-bookings = Открыть записи
tg-host-new-booking = Новая запись: { $event }, { $when }. Гость: { $guest }, { $email }.
tg-host-approval-request = Нужно подтвердить запись: { $event }, { $when }. Гость: { $guest }, { $email }.
tg-host-reschedule-approval = Нужно подтвердить новое время: { $event }, { $when }. Гость: { $guest }, { $email }.
tg-host-rescheduled = Перенос записи: { $event }, теперь { $when }. Гость: { $guest }.
tg-host-cancelled = Отмена записи: { $event }, { $when }. Гость: { $guest }.
tg-host-guest-linked = { $guest } будет получать уведомления о встрече «{ $event }», { $when }, в Telegram: { $username }.
tg-host-guest-linked-differs = В форме указано: { $entered }.
tg-host-no-username = без имени пользователя
tg-bot-help = Этот бот присылает уведомления о записях на { $site }. Чтобы включить их, откройте ссылку со страницы подтверждения или из письма; у платной записи ссылка — на странице оплаты. Отключить: /stop. Когда запись подключена к этому чату, здесь можно также написать её организатору.
tg-bot-linked-guest = Готово: уведомления об этой записи будут приходить сюда.
tg-bot-linked-host = Готово: сюда будут приходить уведомления организатора для { $name }.
tg-bot-link-invalid = Ссылка больше не действует: она работает только в одном чате Telegram и только до конца встречи.
tg-bot-link-replaced = Эта ссылка заменена. Откройте страницу оплаты записи ещё раз и воспользуйтесь ссылкой на Telegram оттуда.
tg-bot-rate-limited = Слишком много попыток. Попробуйте позже.
tg-bot-stopped = Уведомления отключены. Чтобы снова включить их для записи, ещё раз откройте её ссылку из письма с подтверждением; организатор создаёт новую ссылку в разделе «Профиль и настройки» → Telegram.
tg-bot-not-linked = Этот чат не получает уведомлений.
tg-cmd-help = Как работает бот
tg-cmd-stop = Отключить уведомления
tg-book-label = Telegram
tg-book-placeholder = @username
tg-book-help-optional = После записи можно включить уведомления в Telegram.
tg-book-help-required = После записи откройте ссылку на Telegram, чтобы получать уведомления.
tg-book-invalid-title = Проверьте имя пользователя Telegram
tg-book-invalid = Укажите имя пользователя Telegram в виде @name: латиница, цифры и _, от 4 до 32 символов.
tg-book-required = Укажите имя пользователя Telegram.
tg-confirmed-title = Уведомления в Telegram
tg-confirmed-body = Откройте бота и нажмите «Запустить» внизу чата: уведомления об этой записи будут приходить туда.
tg-confirmed-button = Открыть Telegram
tg-email-button = Уведомления в Telegram
tg-email-plain = Уведомления в Telegram: { $url }
tg-settings-nav = Уведомления в Telegram
tg-settings-title = Telegram
tg-settings-intro = Уведомления организатора в вашем чате Telegram: новые записи, запросы на подтверждение, отмены и переносы гостями.
tg-settings-off = Telegram на этом сервере не настроен.
tg-settings-no-bot-name = Бот ещё не готов. Попробуйте через минуту.
tg-settings-linked = Уведомления организатора приходят в { $username }. Подключено: { $date }.
tg-settings-not-linked = Ваш Telegram не подключён.
tg-settings-link = Подключить мой Telegram
tg-settings-link-open = Открыть Telegram
tg-settings-link-expires = В чате нажмите «Запустить» внизу. Ссылка срабатывает один раз в течение 15 минут.
tg-settings-link-limited = Слишком много ссылок за час. Попробуйте позже.
tg-settings-unlink = Отключить
tg-settings-modes = Telegram гостя по типам встреч
tg-settings-modes-help = «По желанию» или «обязательно»: форма записи спрашивает Telegram гостя, а подтверждение предлагает ссылку на бота.
tg-settings-modes-none = Нет типов встреч для настройки.
tg-settings-mode-off = Выключено
tg-settings-mode-optional = По желанию
tg-settings-mode-required = Обязательно
tg-settings-save = Сохранить
tg-settings-saved = Сохранено.
tg-settings-team = Команда { $team }
tg-dashboard-label = Telegram { $username }
tg-dashboard-entered = (указано { $entered })
tg-dashboard-not-linked = (ещё не подключён)
tg-bot-already-linked = Уже включено: уведомления об этой записи приходят в этот чат.
tg-bot-link-invalid-host = Срок действия ссылки истёк: ссылка из настроек срабатывает один раз в течение 15 минут. Создайте новую в разделе «Профиль и настройки» → Telegram.
tg-settings-linked-no-username = Уведомления организатора приходят в ваш чат Telegram. Подключено: { $date }.
tg-bot-description = Этот бот присылает уведомления о ваших записях на { $site }: подтверждения, изменения и напоминания. Нажмите кнопку ниже, чтобы включить их для своей записи. Отключить: /stop.
tg-bot-short-description = Уведомления о записях на { $site }.
tg-button-join = Подключиться

## Telegram: TON payments (ilia fork; docs/ton.md)

tg-guest-payment-due = Запись ждёт оплаты: { $event }, { $when }. К оплате: { $amount } TON (≈ ${ $usd }). Срок оплаты — { $deadline }; если оплата не поступит, время освободится.
tg-guest-payment-partial = Часть оплаты за встречу «{ $event }», { $when }, получена: { $paid } из { $amount } TON. Доплатите оставшиеся { $rest } TON. Срок оплаты — { $deadline }; если оплата не поступит, время освободится.
tg-guest-payment-received = Оплата получена, запись подтверждена: { $event }, { $when }. Организатор: { $host }.
tg-guest-payment-late = Платёж { $value } TON за встречу «{ $event }», { $when }, пришёл после срока, поэтому запись не подтверждена. { $host } свяжется с вами насчёт возврата.
tg-guest-payment-expired = Срок оплаты встречи «{ $event }», { $when }, истёк, и это время освобождено. Вы можете выбрать другое время.
tg-guest-payment-taken = Выбранное вами время встречи «{ $event }», { $when }, заняли до того, как запись была оплачена, поэтому запись отменена. Платить ничего не нужно. Вы можете выбрать другое время.
tg-guest-payment-refund = Мы получили оплату встречи «{ $event }», { $when }, но подтвердить это время не удалось, поэтому запись отменена. { $host } вернёт деньги и свяжется с вами.
tg-guest-payment-refund-part = Часть оплаты уже поступила: { $host } свяжется с вами насчёт возврата.
tg-guest-payment-held = Мы получили оплату встречи «{ $event }», { $when }, но подтвердить запись автоматически не удалось. { $host } свяжется с вами.
tg-button-pay = Оплатить
tg-host-payment-received = Оплачено: { $event }, { $when }. Гость: { $guest }, { $email }. Получено { $paid } TON (≈ ${ $usd }), запись подтверждена.
tg-host-payment-problem = Оплата требует внимания: { $event }, { $when }. Гость: { $guest }, { $email }.
tg-host-payment-transfer = Перевод: { $value } TON.
tg-host-payment-amounts = Получено { $paid } из { $amount } TON.
tg-host-payment-guest-telegram = Telegram гостя: { $username }.

## ilia fork: booking flow v2 (docs/ton.md, docs/operations.md)
troubleshoot-label-busy = Занято
ton-etf-currency = Валюта
ton-etf-currency-ton = TON
ton-etf-currency-usd = USD (пересчёт в TON)
ton-etf-price-help-ton = В TON: от 1 до 100000, гость оплачивает ровно эту сумму (если поле пустое — 100 TON).
ton-etf-usd-no-rates = На этом сервере не настроен курс (CALRS_TON_RATE_MIN_USD и CALRS_TON_RATE_MAX_USD), поэтому записаться по цене в USD нельзя. Укажите цену в TON.
ton-form-error-price-ton = Укажите стоимость в TON от 1 до 100000, не более двух знаков после точки.
ton-form-error-currency = Выберите валюту стоимости: TON или USD.
ton-form-error-telegram = Для Telegram гостя выберите: выключено, по желанию или обязательно.
ton-etf-telegram = Telegram гостя
ton-etf-telegram-help = Для платной записи по умолчанию обязательно: тогда гость получает напоминания об оплате, и с ним можно связаться в Telegram. Можно сделать поле необязательным или отключить его.
ton-slots-price-ton = Стоимость: { $ton } TON
ton-email-amount-value-ton = { $amount } TON
tg-guest-payment-due-ton = Запись ждёт оплаты: { $event }, { $when }. К оплате: { $amount } TON. Срок оплаты — { $deadline }; если оплата не поступит, время освободится.
tg-host-payment-received-ton = Оплачено: { $event }, { $when }. Гость: { $guest }, { $email }. Получено { $paid } TON, запись подтверждена.
ton-pay-renewed = Срок оплаты продлён ({ $n } из { $max }).
ton-dash-badge-renewed = срок продлён { $n } из { $max }
ton-email-renewed-subject = Время всё ещё за вами: { $event }
ton-email-renewed-message = Оплата записи на встречу «{ $event }» (организатор: { $host }) пока не поступила, поэтому время закреплено за вами до { $deadline }. Если вы уже оплатили, ничего делать не нужно: платёж будет засчитан. Если нет, оплатите запись до этого срока на странице по ссылке ниже или отмените её, если ваши планы изменились.
tg-guest-payment-renewed = Оплата пока не поступила: { $event }, { $when }. Время закреплено за вами до { $deadline }: оплатите оставшиеся { $rest } TON до этого срока, иначе время освободится. Если вы уже оплатили, ничего делать не нужно: платёж будет засчитан.

## ilia fork: messages with guests (docs/operations.md)
msg-page-title = Сообщения
msg-back = Записи
msg-guest = Гость: { $guest }
msg-sent = Сообщение отправлено.
msg-empty = Сообщений пока нет.
msg-label = Сообщение гостю
msg-help-email = Гость получит сообщение по почте от имени системы; ответы гостя появятся здесь и придут вам в Telegram.
msg-help-no-email = На этом сервере не настроена электронная почта.
msg-help-telegram = Гость подключил Telegram и получит сообщение и там.
msg-help-no-telegram = Гость не подключил Telegram.
msg-help-limits = Не более 20 сообщений по одной записи в сутки, из них 5 — по почте.
msg-send = Отправить
msg-link = Сообщения
msg-link-count = Сообщения ({ $count })
msg-from-host = Организатор
msg-from-guest = Гость
msg-from-attendee = Участник: { $address }
msg-from-other = Другой адрес, не гостя: { $address }
msg-via-dashboard = веб-интерфейс
msg-via-mcp = агент
msg-via-telegram = Telegram
msg-status-email-queued = письмо в очереди
msg-status-email-sent = письмо отправлено
msg-status-email-failed = письмо не доставлено
msg-status-telegram-queued = Telegram: в очереди
msg-status-telegram-sent = Telegram: отправлено
msg-status-telegram-failed = Telegram: не доставлено
msg-error-empty = Сначала напишите сообщение.
msg-error-too-long = Сообщение может содержать не более 2000 символов.
msg-error-limit = Достигнут суточный лимит сообщений по этой записи. Попробуйте завтра.
msg-error-no-channel = С гостем нельзя связаться: на этом сервере не настроена электронная почта, а гость не подключил Telegram.
msg-error-closed = Сообщения можно отправлять только по ожидающим или подтверждённым записям и не позднее чем через неделю после встречи.
msg-email-subject = Сообщение от { $host }: { $event }, { $date }
msg-email-intro = Сообщение от { $host } по вашей записи «{ $event }»:
msg-email-footer = Чтобы ответить организатору ({ $host }), просто ответьте на это письмо.
tg-guest-host-message = Сообщение от { $host } по записи: { $event }, { $when }.
tg-guest-reply-hint = Чтобы ответить, напишите ответ здесь.
tg-host-guest-message = Сообщение от гостя { $guest } ({ $email }) по записи: { $event }, { $when }.
tg-host-reply-hint = Чтобы ответить гостю, ответьте на это сообщение.
tg-bot-message-sent = Ваше сообщение передано организатору.
tg-bot-message-pick-booking = К этому чату подключено несколько записей. Ответьте на сообщение о нужной записи.
tg-bot-message-closed = Эта запись больше не принимает сообщений.
tg-bot-message-after-payment = Написать организатору можно будет после оплаты записи.
tg-bot-message-limit = Слишком много сообщений. Попробуйте позже.
tg-bot-reply-sent = Отправлено гостю.
tg-bot-reply-not-linked = Гость не подключил Telegram, поэтому ответ не отправлен. Напишите ему из записи в разделе «Записи».
tg-bot-reply-not-allowed = Ответ не отправлен: это не ваша запись или она больше не принимает сообщений.
msg-error-internal = Не удалось сохранить сообщение, поэтому оно не отправлено. Попробуйте ещё раз.
msg-link-new = Сообщения (новых: { $count })
msg-recent-title = Недавние встречи
msg-recent-help = Встречи, которые уже начались или закончились за последнюю неделю. С их гостями ещё можно переписываться.
tg-bot-message-sent-shortened = Ваше сообщение передано организатору; оно сокращено до { $max } символов.
tg-bot-reply-too-long = Ответ не отправлен: в сообщении может быть не больше { $max } символов.
tg-bot-reply-empty = Ответ не отправлен: в нём нет видимого текста.
tg-bot-reply-failed = Не удалось сохранить ответ, поэтому он не отправлен. Попробуйте ещё раз.
troubleshoot-not-synced = Календари загружаются только с { $from } до { $until }: события за пределами этих дней не загружаются, поэтому здесь их нет.
event-type-form-horizon-capped =
    { $days ->
        [one] На этом сервере записаться можно не более чем на { $days } день вперёд, какой бы ни была глубина записи.
        [few] На этом сервере записаться можно не более чем на { $days } дня вперёд, какой бы ни была глубина записи.
        [many] На этом сервере записаться можно не более чем на { $days } дней вперёд, какой бы ни была глубина записи.
       *[other] На этом сервере записаться можно не более чем на { $days } дня вперёд, какой бы ни была глубина записи.
    }

## ilia fork: the system mailbox (src/inbox)
msg-help-email-no-inbox = Гость получит сообщение по почте от имени системы. Ответы на такие письма на этом сервере не читаются.
msg-via-email = почта
msg-email-footer-no-reply = Это письмо отправлено системой записи от имени организатора ({ $host }); ответы на него не читаются.
tg-host-guest-email = Письмо от гостя { $guest } ({ $email }) по записи: { $event }, { $when }.
tg-host-attendee-email = Письмо от участника встречи { $sender } по записи гостя { $guest } ({ $email }): { $event }, { $when }.
tg-host-other-email = Письмо с другого адреса, не гостя: { $sender }. Переписка по записи гостя { $guest } ({ $email }): { $event }, { $when }.
booking-blocked-body-system = Отменить или перенести запись можно не позднее чем за { $minutes } мин до начала встречи. Если вы не сможете прийти, напишите нам: <a href="{ $href }">{ $address }</a>.
booking-blocked-body-no-reply = Отменить или перенести запись можно не позднее чем за { $minutes } мин до начала встречи. Если вы не сможете прийти, пожалуйста, сообщите об этом организатору другим способом.

## ilia fork: passkeys and security keys (docs/passkeys-plan.md)

ilia-passkey-sign-in = Войти с ключом доступа
ilia-mfa-login-help-passkey = Подтвердите вход ключом доступа или аппаратным ключом. Вход нужно завершить за 10 минут.
ilia-mfa-use-totp = Ввести код из приложения
ilia-mfa-use-recovery = Ввести резервный код
ilia-mfa-use-passkey = Использовать ключ доступа
ilia-mfa-use-passkey-instead = Использовать вместо этого ключ доступа или аппаратный ключ
ilia-mfa-totp-login-help = Введите шестизначный код из приложения-аутентификатора. Вход нужно завершить за 10 минут.
ilia-mfa-recovery-code = Резервный код
ilia-mfa-recovery-login-help = Введите один из сохранённых резервных кодов. Каждый код действует только один раз. Вход нужно завершить за 10 минут.
ilia-passkey-unsupported = Этот браузер не поддерживает ключи доступа и аппаратные ключи. Выберите другой способ ниже или откройте страницу в свежей версии браузера.
ilia-passkey-noscript = Для ключей доступа нужен JavaScript. Включите его для этого сайта или выберите другой способ ниже.
ilia-passkey-cancelled = Запрос отменён, или истекло время ожидания. Попробуйте ещё раз или выберите другой способ.
ilia-passkey-invalid = Не удалось проверить ключ доступа или аппаратный ключ, либо истёк срок действия запроса. Попробуйте ещё раз или выберите другой способ.
ilia-passkey-key-type = Этот браузер не может прочитать ключ такого типа, поэтому добавить его здесь нельзя. Попробуйте другой браузер, например свежую версию Firefox или Chrome, или выберите другой способ.
ilia-passkey-wrong-site = Ключи доступа работают только по адресу { $origin }. Откройте эту страницу там и попробуйте ещё раз или выберите другой способ.
ilia-mfa-enroll-help = Выберите, чем вы будете подтверждать каждый вход. Надёжнее всего ключ доступа или аппаратный ключ: он работает только на этом сайте, и поддельная копия страницы не сможет им воспользоваться.
ilia-passkey-add-security-key = Добавить аппаратный ключ
ilia-passkey-add-device = Добавить ключ доступа на этом устройстве или телефоне
ilia-passkey-kinds-help = Аппаратный ключ — это USB- или NFC-ключ, например YubiKey. Ключ доступа на этом устройстве или телефоне подтверждается через Touch ID, Face ID, Windows Hello или блокировку экрана телефона и может синхронизироваться с другими вашими устройствами.
ilia-mfa-use-app-instead = Использовать вместо этого приложение-аутентификатор
ilia-passkey-name = Название ключа
ilia-passkey-name-help = Необязательно, например «YubiKey на связке» или «MacBook». До 64 символов.
ilia-passkey-default-security-key = Аппаратный ключ { $n }
ilia-passkey-default-device = Ключ доступа { $n }
ilia-passkey-register-help = Нажмите кнопку и следуйте подсказкам браузера: вставьте аппаратный ключ и коснитесь его или подтвердите через Touch ID, Face ID, Windows Hello или телефон.
ilia-passkey-exists = Этот ключ уже зарегистрирован.
ilia-mfa-recovery-help = Сохраните эти резервные коды в надёжном месте: они показываются только один раз. Каждый код заменяет ключ доступа или код из приложения при одном входе. Новые коды заменяют все прежние.
ilia-mfa-status-on = Двухфакторная аутентификация включена.
ilia-mfa-status-off = Двухфакторная аутентификация выключена.
ilia-mfa-setup-intro = Добавьте второй шаг к каждому входу: ключ доступа или аппаратный ключ (YubiKey, Touch ID, Face ID, Windows Hello, телефон) либо приложение-аутентификатор.
ilia-passkey-list = Ключи доступа и аппаратные ключи
ilia-passkey-none = Ключей пока нет.
ilia-passkey-kind-hardware = Аппаратный ключ
ilia-passkey-kind-synced = Синхронизируемый ключ доступа
ilia-passkey-added = Добавлен { $date }
ilia-passkey-last-used = Последний вход: { $date }
ilia-passkey-never-used = Ещё не использовался
ilia-passkey-other-site = Зарегистрирован для { $rp_id } и не работает по текущему адресу сайта.
ilia-passkey-rename = Переименовать
ilia-passkey-delete = Удалить
ilia-passkey-add = Добавить ключ доступа или аппаратный ключ
ilia-passkey-new-name = Новое название
ilia-passkey-limit = Достигнуто максимальное число ключей ({ $max }). Удалите один, чтобы добавить новый.
ilia-passkey-backup-advice = Добавьте второй ключ и храните его в надёжном месте, чтобы потеря одного ключа не лишила вас доступа.
ilia-passkey-unavailable = Ключи доступа отключены: публичный адрес сервера (CALRS_BASE_URL) не начинается с https://. Обратитесь к администратору. Уже добавленные ключи показаны в списке, их можно удалить.
ilia-mfa-totp-title = Приложение-аутентификатор
ilia-mfa-totp-on = Приложение-аутентификатор подключено.
ilia-mfa-totp-off = Приложение-аутентификатор не подключено.
ilia-mfa-totp-add = Подключить приложение-аутентификатор
ilia-mfa-totp-remove = Отключить приложение-аутентификатор
ilia-mfa-recovery-title = Резервные коды
ilia-mfa-recovery-left =
    { $count ->
        [one] Остался { $count } резервный код
        [few] Осталось { $count } резервных кода
        [many] Осталось { $count } резервных кодов
       *[other] Осталось { $count } резервного кода
    }
ilia-mfa-last-factor = Двухфакторная аутентификация обязательна, поэтому последний способ подтверждения удалить нельзя. Сначала добавьте другой.
ilia-mfa-last-factor-off = Это ваш последний способ: если его удалить, двухфакторная аутентификация выключится, а резервные коды будут удалены.
ilia-mfa-changes-help = Любое изменение здесь требует пароля и ключа или кода и завершает остальные ваши сеансы.
ilia-mfa-confirm-title = Подтвердите, что это вы
ilia-mfa-confirm-help = Введите пароль, затем подтвердите ключом доступа или кодом.
ilia-passkey-confirm = Подтвердить ключом
ilia-mfa-confirm-code = Подтвердить кодом
ilia-mfa-action-passkey-add = Вы добавляете ключ доступа или аппаратный ключ.
ilia-mfa-action-passkey-rename = Вы переименовываете ключ «{ $name }».
ilia-mfa-action-passkey-delete = Вы удаляете ключ «{ $name }».
ilia-mfa-action-totp-add = Вы подключаете приложение-аутентификатор.
ilia-mfa-action-totp-remove = Вы отключаете приложение-аутентификатор.
ilia-mfa-action-regenerate = Вы заменяете все резервные коды.
ilia-mfa-action-policy = Вы меняете правило двухфакторной аутентификации для локальных аккаунтов на:
ilia-mfa-policy-continue = Продолжить

## ilia fork: the instance name (CALRS_BRAND_NAME) and the error page (src/web/error_page.rs)
ilia-login-subtitle = Войдите в свой аккаунт { $brand }
ilia-register-subtitle = Создайте новый аккаунт { $brand }
ilia-error-not-found = Не найдено
ilia-error-not-found-body = По этому адресу ничего нет. Проверьте ссылку.
ilia-error-gone = Ссылка больше не действует
ilia-error-bad-request = Неверный запрос
ilia-error-forbidden = Доступ ограничен
