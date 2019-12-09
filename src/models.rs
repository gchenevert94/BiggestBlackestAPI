#[derive(Clone, Queryable)]
pub struct Card {
    pub id: i32,
    pub is_black: bool,
    pub format_text: String,
    pub total_votes: i32,
    pub average_rating: Option<f32>,
}

#[derive(Clone, Queryable)]
pub struct CardFromSet {
    pub id: i32,
    pub is_black: bool,
    pub format_text: String,
    pub total_votes: i32,
    pub average_rating: Option<f32>,
    pub set_id: i32,
}

#[derive(Clone, Queryable)]
pub struct ParentSet {
    pub id: i32,
    pub name: String,
}

pub struct GetCards {
    pub id: i32,
    pub format_text: String,
    pub is_black: bool,
    pub parent_set_id: i32,
    pub total_votes: i32,
    pub average_rating: f32
}
