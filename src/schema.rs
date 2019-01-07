table! {
    ban (id) {
        id -> Integer,
        user -> Integer,
        guild -> Nullable<Text>,
        end_epoch -> Nullable<Text>,
    }
}

table! {
    lang_stat (id) {
        id -> Integer,
        lang_name -> Text,
        snippets_executed -> Integer,
    }
}

table! {
    snippet (id) {
        id -> Integer,
        user -> Integer,
        code -> Text,
        language -> Text,
        guild -> Nullable<Text>,
        run_time -> Text,
    }
}

table! {
    user (id) {
        id -> Integer,
        discord_id -> Text,
    }
}

joinable!(ban -> user (user));
joinable!(snippet -> user (user));

allow_tables_to_appear_in_same_query!(
    ban,
    lang_stat,
    snippet,
    user,
);
