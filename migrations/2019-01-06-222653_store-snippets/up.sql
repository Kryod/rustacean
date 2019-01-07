CREATE TABLE IF NOT EXISTS snippet (
    id          INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user        INTEGER NOT NULL,
    code        TEXT NOT NULL,
    language    TEXT NOT NULL,
    guild       TEXT,
    run_time    TEXT NOT NULL,
    FOREIGN KEY (user) REFERENCES user (id)
);
