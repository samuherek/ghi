// @generated automatically by Diesel CLI.

diesel::table! {
    quests (id) {
        id -> Integer,
        cmd_name -> Text,
        cmd_pattern -> Text,
        cmd_quest -> Text,
        notes -> Nullable<Text>,
        mock_output -> Nullable<Text>,
        display_count -> Integer,
        ok_count -> Integer,
        miss_count -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
