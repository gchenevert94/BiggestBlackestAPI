pub mod bb {
    table! {
        bb.card (id) {
            id -> Int4,
            isblack -> Bool,
            formattext -> Text,
            isactive -> Bool,
            lastmodified -> Timestamp,
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
        bb.parent_set (id) {
            id -> Int4,
            name -> Text,
            isactive -> Bool,
            lastmodified -> Timestamp,
        }
    }

    table! {
        bb.parent_set_card (parentsetid, cardid) {
            parentsetid -> Int4,
            cardid -> Int4,
            isactive -> Bool,
            lastmodified -> Timestamp,
        }
    }

    table! {
        bb.user (id) {
            id -> Int4,
            username -> Text,
            isactive -> Bool,
            lastmodified -> Timestamp,
        }
    }

    joinable!(draw_card -> card (cardid));
    joinable!(parent_set_card -> card (cardid));
    joinable!(parent_set_card -> parent_set (parentsetid));

    allow_tables_to_appear_in_same_query!(
        card,
        draw_card,
        parent_set,
        parent_set_card,
        user,
    );
}
