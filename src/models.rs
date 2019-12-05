use chrono::{NaiveDateTime};

#[derive(Clone, Queryable)]
pub struct Card {
    pub id: i32,
    pub is_black: bool,
    pub format_text: String,
}

#[derive(Clone, Queryable)]
pub struct CardFromSet {
    pub id: i32,
    pub is_black: bool,
    pub format_text: String,
    pub set_id: i32
}

#[derive(Clone, Queryable)]
pub struct ParentSet {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable)]
pub struct ParentSetCard {
    parentsetid: i32,
    card_id: i32,
    is_active: bool,
    last_modified: NaiveDateTime
}
