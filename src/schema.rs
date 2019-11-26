pub mod bb {
    table! {
        bb.Card (id) {
            id -> Int4,
            isblack -> Bool,
            formattext -> Text,
            isactive -> Bool,
            lastmodified -> Timestamp,
        }
    }

    table! {
        bb.DrawCard (id) {
            id -> Int4,
            cardid -> Int4,
            drawdate -> Timestamp,
            sessionkey -> Nullable<Text>,
            isactive -> Bool,
            lastmodified -> Timestamp,
        }
    }

    table! {
        bb.ParentSet (id) {
            id -> Int4,
            name -> Text,
            isactive -> Bool,
            lastmodified -> Timestamp,
        }
    }

    table! {
        bb.ParentSetCard (parentsetid, cardid) {
            parentsetid -> Int4,
            cardid -> Int4,
            isactive -> Bool,
            lastmodified -> Timestamp,
        }
    }

    table! {
        bb.User (id) {
            id -> Int4,
            username -> Text,
            isactive -> Bool,
            lastmodified -> Timestamp,
        }
    }

    joinable!(DrawCard -> Card (cardid));
    joinable!(ParentSetCard -> Card (cardid));
    joinable!(ParentSetCard -> ParentSet (parentsetid));

    allow_tables_to_appear_in_same_query!(
        Card,
        DrawCard,
        ParentSet,
        ParentSetCard,
        User,
    );
}
