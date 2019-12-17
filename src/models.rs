/// Struct returned from the `get_sets()` method
pub struct GetSetResults {
  pub id: i32,
  pub name: String,
}

/// Struct returned from the `get_cards()` method
pub struct GetCardResults {
  pub id: i32,
  pub format_text: String,
  pub is_black: bool,
  pub parent_set_id: i32,
  pub parent_set_name: String,
  pub total_votes: i32,
  pub average_rating: Option<f32>,
}

/// Struct used to call the `add_card()` method.
/// These fields are all required (hence no default impl)
pub struct AddCard {
  pub user_id: i32,
  pub format_text: String,
  pub is_black: bool,
}

/// Struct returned from the `add_card()` method containing
/// the ID of the newly-inserted card object
pub struct AddCardResult {
  pub id: i32,
}

/// Struct used to call the `add_user_card_rating()` method.
/// These fields are all required (hence no default impl)
pub struct AddCardRating {
  pub user_id: i32,
  pub card_id: i32,
  pub rating: f32,
}

/// Struct returned from the `add_user_card_rating` method
/// containing the new total_votes and average_rating after
/// submission of the rating.
pub struct AddCardRatingResult {
  pub total_votes: i32,
  pub average_rating: f32,
}

/// Struct used to call the `add_card_combination_rating` method.
/// These fields are all required (hence no default impl)
pub struct AddCardRatingCombination {
  pub user_id: i32,
  pub white_card_id: i32,
  pub black_card_id: i32,
  pub rating: f32,
  pub ordinal: i32,
}

/// Struct used to call the `get_sets()` method.
/// The fields are `Option` to allow NULL database parameters.
pub struct GetSets {
  pub search: Option<String>,
  pub n_results: Option<i32>,
  pub cursor: Option<i32>,
}

impl GetSets {
  /// Default impl for GetSets. Creates a default limit of 100.
  pub fn default() -> GetSets {
    GetSets {
      search: None,
      n_results: Some(100),
      cursor: None,
    }
  }
}

/// Struct used to call the `get_cards()` method.
/// The fields are `Option` to allow NULL database parameters.
pub struct GetCards {
  pub search: Option<String>,
  pub filter_black: Option<bool>,
  pub previous_cursor: Option<i32>,
  pub n_cards: Option<i32>,
  pub card_sets: Option<Vec<i32>>,
  pub get_random: Option<bool>,
  pub random_seed: Option<f32>,
  pub user_submitted: Option<bool>,
}

impl GetCards {
  /// Default impl for GetCards.
  /// Creates a default limit of 100 and sets user_submitted and get_random to false.
  pub fn default() -> GetCards {
    GetCards {
      search: None,
      filter_black: None,
      previous_cursor: None,
      n_cards: Some(100),
      card_sets: None,
      get_random: Some(false),
      random_seed: None,
      user_submitted: Some(false),
    }
  }
}
