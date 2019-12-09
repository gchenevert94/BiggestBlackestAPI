pub mod bb {
    table! {
        use diesel_full_text_search::TsVector;
        use diesel::sql_types::{
            Int4, Bool, Text, Timestamp, Nullable, Float4
        };
        bb.card (id) {
            id -> Int4,
            is_black -> Bool,
            format_text -> Text,
            is_active -> Bool,
            last_modified -> Timestamp,
            text_searchable_format_text -> TsVector,
            average_rating -> Nullable<Float4>,
            total_votes -> Int4,
        }
    }

    table! {
        bb.draw_card (id) {
            id -> Int4,
            cardid -> Int4,
            drawdate -> Timestamp,
            sessionkey -> Nullable<Text>,
            isactive -> Bool,
            lastmodified -> Timestamp,
        }
    }

    table! {
        use diesel_full_text_search::TsVector;
        use diesel::sql_types::{
            Int4, Text, Bool, Timestamp
        };
        bb.parent_set (id) {
            id -> Int4,
            name -> Text,
            is_active -> Bool,
            last_modified -> Timestamp,
            text_searchable_name -> TsVector,
        }
    }

    table! {
        bb.parent_set_card (parent_set_id, card_id) {
            parent_set_id -> Int4,
            card_id -> Int4,
            is_active -> Bool,
            last_modified -> Timestamp,
        }
    }

    table! {
        bb.user (id) {
            id -> Int4,
            username -> Text,
            is_active -> Bool,
            last_modified -> Timestamp,
        }
    }

    table! {
        bb.user_card_rating (user_id, card_id) {
            user_id -> Int4,
            card_id -> Int4,
            rating -> Float4,
            created_date -> Timestamp,
            is_active -> Bool,
            last_modified -> Timestamp,
        }
    }

    joinable!(draw_card -> card (cardid));
    joinable!(parent_set_card -> card (card_id));
    joinable!(parent_set_card -> parent_set (parent_set_id));
    joinable!(user_card_rating -> card (card_id));
    joinable!(user_card_rating -> user (user_id));

    allow_tables_to_appear_in_same_query!(
        card,
        draw_card,
        parent_set,
        parent_set_card,
        user,
        user_card_rating,
    );
}
