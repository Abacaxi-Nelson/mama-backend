table! {
    families (id) {
        id -> Uuid,
        nom -> Varchar,
        created_at -> Timestamp,
    }
}

table! {
    smss (id) {
        id -> Uuid,
        tel -> Varchar,
        code -> Varchar,
        created_at -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Uuid,
        tel -> Varchar,
        nom -> Varchar,
        email -> Varchar,
        created_at -> Timestamp,
    }
}

table! {
    users_families (id) {
        id -> Uuid,
        user_id -> Uuid,
        family_id -> Uuid,
        role -> Varchar,
        created_at -> Timestamp,
    }
}

joinable!(users_families -> families (family_id));
joinable!(users_families -> users (user_id));

allow_tables_to_appear_in_same_query!(
    families,
    smss,
    users,
    users_families,
);
