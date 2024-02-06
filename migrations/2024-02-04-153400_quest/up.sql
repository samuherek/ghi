-- Your SQL goes here
CREATE TABLE quests (
    id INTEGER PRIMARY KEY NOT NULL,
    cmd_name VARCHAR NOT NULL,
    cmd_pattern VARCHAR NOT NULL,
    cmd_quest VARCHAR NOT NULL,
    notes VARCHAR,
    mock_output VARCHAR,
    display_count INTEGER NOT NULL DEFAULT 0,
    ok_count INTEGER NOT NULL DEFAULT 0,
    miss_count INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
