CREATE TABLE IF NOT EXISTS user (
    id          INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    discord_id  TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS ban (
    id          INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user        INTEGER NOT NULL,
    guild       TEXT,
    end_epoch   TEXT,
    FOREIGN KEY (user) REFERENCES user (id)
);

CREATE TABLE IF NOT EXISTS lang_stat (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    lang_name           TEXT    NOT NULL,
    snippets_executed   INTEGER NOT NULL
);
