// @generated automatically by Diesel CLI.

diesel::table! {
    courses (id) {
        id -> Integer,
        name -> Text,
        description -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

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
        course_id -> Integer,
    }
}

diesel::joinable!(quests -> courses (course_id));

diesel::allow_tables_to_appear_in_same_query!(
    courses,
    quests,
);
