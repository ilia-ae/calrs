-- ilia fork: a user's public profile (name, title, bio) per language (owner's
-- report 2026-11-13: the Russian widget on ilia.ae showed the host's header as
-- "Ilia Arestov" / "Fractional CTO"; CLAUDE.md "Profile translations (ilia
-- fork)", src/web/user_texts.rs).
-- `lang` is a shipped locale code (`i18n::supported_code`: en, fr, es, pl, de,
-- de-formal, it, et, pt, nl, ru), checked by the settings form and
-- `config apply`. Each field is optional and falls back on its own: a guest
-- reading `lang` sees this row's value, else the value of the row of its base
-- language (`de-formal` -> `de`), else the user's own `name`, `title`, `bio`.
-- A row has at least one of the three. No row, no change anywhere.
CREATE TABLE user_texts (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    lang    TEXT NOT NULL,
    name    TEXT,
    title   TEXT,
    bio     TEXT,
    PRIMARY KEY (user_id, lang)
);
