pub struct GetSetResults {
  pub id: i32,
  pub name: String,
}

pub struct GetCardResults {
  pub id: i32,
  pub format_text: String,
  pub is_black: bool,
  pub parent_set_id: i32,
  pub parent_set_name: String,
  pub total_votes: i32,
  pub average_rating: Option<f32>,
}
