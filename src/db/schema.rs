// @generated automatically by Diesel CLI.

diesel::table! {
    bucket (id) {
        id -> Integer,
        value -> Text,
        notes -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    lessons (id) {
        id -> Integer,
        name -> Text,
        cmd -> Text,
        description -> Text,
        remote -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    quests (id) {
        id -> Integer,
        cmd -> Text,
        pattern -> Text,
        is_pattern_literal -> Bool,
        quest -> Text,
        notes -> Nullable<Text>,
        mock_output -> Nullable<Text>,
        display_count -> Integer,
        ok_count -> Integer,
        miss_count -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        lesson_id -> Integer,
    }
}

diesel::joinable!(quests -> lessons (lesson_id));

diesel::allow_tables_to_appear_in_same_query!(
    bucket,
    lessons,
    quests,
);
