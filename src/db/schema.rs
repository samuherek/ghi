// @generated automatically by Diesel CLI.

diesel::table! {
    quests (id) {
        id -> Integer,
        cmd_name -> Text,
        pattern -> Text,
        query -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
