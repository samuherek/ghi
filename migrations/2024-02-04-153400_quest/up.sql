-- Your SQL goes here
CREATE TABLE quests (
    id INTEGER PRIMARY KEY NOT NULL,
    cmd VARCHAR NOT NULL,
    pattern VARCHAR NOT NULL,
    is_pattern_literal BOOLEAN NOT NULL CHECK (is_pattern_literal IN (0, 1)),
    quest VARCHAR NOT NULL,
    notes VARCHAR,
    mock_output VARCHAR,
    display_count INTEGER NOT NULL DEFAULT 0,
    ok_count INTEGER NOT NULL DEFAULT 0,
    miss_count INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    lesson_id INTEGER NOT NULL,
    FOREIGN KEY (lesson_id) REFERENCES lessons(id)
);

CREATE TABLE lessons (
    id INTEGER PRIMARY KEY NOT NULL,
    name VARCHAR NOT NULL,
    cmd VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    remote BOOLEAN NOT NULL CHECK (remote IN (0, 1)),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

