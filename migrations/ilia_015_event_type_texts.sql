-- ilia fork: an event type's title and description per language (owner's
-- request 2026-11-13: the widget on the Russian page shows the event type in
-- Russian; CLAUDE.md "Event type translations (ilia fork)", src/web/event_texts.rs).
-- `lang` is a shipped locale code (`i18n::supported_code`: en, fr, es, pl, de,
-- de-formal, it, et, pt, nl, ru), checked by the form and `config apply`.
-- A guest reading `lang` sees this row's title and description (NULL: none),
-- else the row of its base language (`de-formal` -> `de`), else the event
-- type's own `title` and `description`. No row, no change anywhere.
CREATE TABLE event_type_texts (
    event_type_id TEXT NOT NULL REFERENCES event_types(id) ON DELETE CASCADE,
    lang          TEXT NOT NULL,
    title         TEXT NOT NULL,
    description   TEXT,
    PRIMARY KEY (event_type_id, lang)
);
