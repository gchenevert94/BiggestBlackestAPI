use crate::{
  db::{self, Pool},
  models::{AddCard, AddCardRating, AddCardRatingCombination, GetCards, GetSets},
  Context,
};
use actix_web::{
  web::{self, Data, Json, ServiceConfig},
  Error as AWError, HttpResponse,
};
use base64::{decode, encode, DecodeError};
use juniper::{
  http::{playground::playground_source, GraphQLRequest},
  Context as JContext, Executor, FieldError, IntoFieldError, ID,
};
use juniper_from_schema::graphql_schema_from_file;
use rand::random;
use serde_json::{error::Error as SError, to_string};
use std::{num::ParseIntError, panic, sync::Arc};
use url::{ParseError, Url};

impl JContext for Context {}

graphql_schema_from_file!("schema.graphql", error_type: GqlError);

/// Helper trait for encoding a value into a JuniperID
trait ToEncodedJuniperID {
  fn to_encoded_id(&self) -> ID;
  fn from_encoded_id(id: ID) -> Result<Self, DecodeError>
  where
    Self: Sized;
}

impl ToEncodedJuniperID for f32 {
  fn to_encoded_id(&self) -> ID {
    let encoding = self.to_be_bytes();
    ID::new(encode(&encoding))
  }

  fn from_encoded_id(id: ID) -> Result<f32, DecodeError> {
    let decoded = decode(&id.to_string())?;
    let decoded = decoded
      .iter()
      .take(4)
      .fold((0, [0; 4]), |(i, mut acc), &x| {
        acc[i] = x;
        (i + 1, acc)
      });
    Ok(f32::from_be_bytes(decoded.1))
  }
}

impl ToEncodedJuniperID for i32 {
  fn to_encoded_id(&self) -> ID {
    let encoding = self.to_be_bytes();
    ID::new(encode(&encoding))
  }

  fn from_encoded_id(id: ID) -> Result<i32, DecodeError> {
    let decoded_v = decode(&id.to_string())?;
    Ok(decoded_v.iter().fold(0, |acc, &x| (acc << 8) + x as i32))
  }
}

pub struct Card {
  id: i32,
  format_text: String,
  color: CardColor,
  average_rating: Option<f32>,
  total_votes: i32,
  set: SetInfo,
}

impl CardFields for Card {
  fn field_id(&self, _: &Executor<'_, Context>) -> Result<ID, GqlError> {
    Ok(ID::from(self.id.to_string()))
  }

  /// Format text includes basic HTML markdown for
  /// text-decorations. Cards are formatted with a
  /// `<prompt/>` tag in place of '_', or the like
  /// to reduce confusion
  fn field_format_text(&self, _: &Executor<'_, Context>) -> Result<&String, GqlError> {
    Ok(&self.format_text)
  }

  /// Cards should be partitioned by clients for greater
  /// flexibility
  fn field_color(&self, _: &Executor<'_, Context>) -> Result<CardColor, GqlError> {
    Ok(self.color)
  }

  fn field_set(
    &self,
    _: &Executor<'_, Context>,
    _: &QueryTrail<'_, SetInfo, Walked>,
  ) -> Result<&SetInfo, GqlError> {
    Ok(&self.set)
  }

  fn field_total_votes(&self, _: &Executor<'_, Context>) -> Result<i32, GqlError> {
    Ok(self.total_votes)
  }

  fn field_average_rating(&self, _: &Executor<'_, Context>) -> Result<Option<f64>, GqlError> {
    Ok(self.average_rating.map(|v| v.into()))
  }
}

pub struct CardOperation {
  id: i32,
  format_text: String,
  color: CardColor,
}

impl CardOperationFields for CardOperation {
  fn field_id(&self, _: &Executor<'_, Context>) -> Result<ID, GqlError> {
    Ok(ID::from(self.id.to_string()))
  }

  fn field_format_text(&self, _: &Executor<'_, Context>) -> Result<&String, GqlError> {
    Ok(&self.format_text)
  }

  fn field_color(&self, _: &Executor<'_, Context>) -> Result<CardColor, GqlError> {
    Ok(self.color)
  }
}

pub struct CardResult {
  results: Vec<Card>,
  last_cursor: Option<i32>,
  has_next_page: bool,
  random_seed: Option<f32>,
}

impl CardResultFields for CardResult {
  fn field_results(
    &self,
    _: &Executor<'_, Context>,
    _: &QueryTrail<'_, Card, Walked>,
  ) -> Result<&Vec<Card>, GqlError> {
    Ok(&self.results)
  }

  fn field_last_cursor(&self, _: &Executor<'_, Context>) -> Result<Option<ID>, GqlError> {
    Ok(self.last_cursor.map(|v| v.to_encoded_id()))
  }

  fn field_has_next_page(&self, _: &Executor<'_, Context>) -> Result<bool, GqlError> {
    Ok(self.has_next_page)
  }

  fn field_random_seed(&self, _: &Executor<'_, Context>) -> Result<Option<ID>, GqlError> {
    Ok(self.random_seed.map(|c| c.to_encoded_id()))
  }
}

pub struct Set {
  id: i32,
  name: String,
}

impl SetFields for Set {
  fn field_id(&self, _: &Executor<'_, Context>) -> Result<ID, GqlError> {
    Ok(ID::from(self.id.to_string()))
  }

  fn field_name(&self, _: &Executor<'_, Context>) -> Result<&String, GqlError> {
    Ok(&self.name)
  }

  /// Most useful method. Can be used for "actual" gameplay. To *shuffle* the cards,
  /// pass `randomized: true`, and use the resulting `randomSeed` to keep the
  /// same card shuffle in subsequent results.
  ///
  /// Search field is a full-text-search implementation
  fn field_cards(
    &self,
    executor: &Executor<'_, Context>,
    _: &QueryTrail<'_, CardResult, Walked>,
    search: Option<String>,
    card_color: Option<CardColor>,
    pagination: Pagination,
    randomized: Option<bool>,
  ) -> Result<Option<CardResult>, GqlError> {
    if pagination.page_size > 1000 || pagination.page_size < 0 {
      return Err(GqlError::LimitOutOfBounds);
    }

    let mut get_cards = GetCards::default();
    get_cards.n_cards = Some(pagination.page_size + 1);
    get_cards.search = search;
    get_cards.card_sets = Some(vec![self.id]);

    match pagination.cursor.map(|v| i32::from_encoded_id(v)) {
      Some(Ok(v)) => get_cards.previous_cursor = Some(v),
      Some(Err(e)) => {
        return Err(e.into());
      }
      _ => get_cards.previous_cursor = None,
    }

    get_cards.filter_black = match card_color {
      Some(CardColor::Black) => Some(true),
      Some(CardColor::White) => Some(false),
      None => None,
    };

    if let Some(r) = randomized {
      get_cards.get_random = Some(r);

      match pagination.random_seed.map(|v| f32::from_encoded_id(v)) {
        Some(Ok(v)) => get_cards.random_seed = Some(v),
        Some(Err(e)) => {
          return Err(e.into());
        }
        None => get_cards.random_seed = Some(random::<f32>()),
      }
    }

    let db_cards = db::get_cards(&executor.context().db, &get_cards)?;

    let has_more = db_cards.iter().len() as i32 > pagination.page_size;
    let last_cursor = match get_cards.get_random {
      Some(true) => Some(get_cards.previous_cursor.unwrap_or(0) + pagination.page_size),
      _ => db_cards
        .iter()
        .nth(pagination.page_size as usize - 1)
        .map(|r| r.id),
    };

    let db_cards = db_cards
      .iter()
      .take(pagination.page_size as usize)
      .map(|c| Card {
        id: c.id,
        color: match c.is_black {
          true => CardColor::Black,
          false => CardColor::White,
        },
        format_text: c.format_text.to_owned(),
        set: SetInfo {
          id: c.parent_set_id,
          name: c.parent_set_name.to_owned(),
        },
        total_votes: c.total_votes,
        average_rating: c.average_rating,
      })
      .collect::<Vec<_>>();

    Ok(Some(CardResult {
      results: db_cards,
      has_next_page: has_more,
      last_cursor: last_cursor,
      random_seed: get_cards.random_seed,
    }))
  }
}

pub struct SetInfo {
  id: i32,
  name: String,
}

impl SetInfoFields for SetInfo {
  fn field_id(&self, _: &Executor<'_, Context>) -> Result<ID, GqlError> {
    Ok(ID::from(self.id.to_string()))
  }

  fn field_name(&self, _: &Executor<'_, Context>) -> Result<&String, GqlError> {
    Ok(&self.name)
  }
}

pub struct SetResult {
  results: Vec<SetInfo>,
  last_cursor: Option<i32>,
  has_next_page: bool,
}

impl SetResultFields for SetResult {
  fn field_results(
    &self,
    _: &Executor<'_, Context>,
    _: &QueryTrail<'_, SetInfo, Walked>,
  ) -> Result<&Vec<SetInfo>, GqlError> {
    Ok(&self.results)
  }

  fn field_last_cursor(&self, _: &Executor<'_, Context>) -> Result<Option<ID>, GqlError> {
    Ok(self.last_cursor.map(|c| c.to_encoded_id()))
  }

  fn field_has_next_page(&self, _: &Executor<'_, Context>) -> Result<bool, GqlError> {
    Ok(self.has_next_page)
  }
}

/// Biggest Blackest API Schema documentation
/// Primary (only) access to cards in the database
pub struct Query {}

impl QueryFields for Query {
  fn field_cards(
    &self,
    executor: &Executor<'_, Context>,
    _: &QueryTrail<'_, CardResult, Walked>,
    search: Option<String>,
    color: Option<CardColor>,
    pagination: Pagination,
    set_ids: Option<Vec<juniper::ID>>,
    randomized: Option<bool>,
    card_source: CardSource,
  ) -> Result<CardResult, GqlError> {
    //AWError handling
    let limit = pagination.page_size;

    if limit > 1000 || limit < 0 {
      return Err(GqlError::LimitOutOfBounds);
    }

    let mut get_cards = GetCards::default();

    get_cards.n_cards = Some(limit + 1);

    if let Some(v) = set_ids {
      get_cards.card_sets = Some(
        v.iter()
          .map(|i| i.parse().expect("Values must be numerical"))
          .collect(),
      );
    }

    match pagination.cursor.map(|v| i32::from_encoded_id(v)) {
      Some(Ok(v)) => get_cards.previous_cursor = Some(v),
      Some(Err(e)) => {
        return Err(e.into());
      }
      _ => get_cards.previous_cursor = None,
    }
    get_cards.search = search;

    get_cards.user_submitted = match card_source {
      CardSource::All => None,
      CardSource::User => Some(true),
      CardSource::Official => Some(false),
    };

    match color {
      Some(CardColor::Black) => {
        get_cards.filter_black = Some(true);
      }
      Some(CardColor::White) => {
        get_cards.filter_black = Some(false);
      }
      _ => {}
    }

    if let Some(r) = randomized {
      get_cards.get_random = Some(r);

      match pagination.random_seed.map(|v| f32::from_encoded_id(v)) {
        Some(Ok(v)) => get_cards.random_seed = Some(v),
        Some(Err(e)) => {
          return Err(e.into());
        }
        None => get_cards.random_seed = Some(random::<f32>()),
      }
    }

    let con = &executor.context().db;
    let db_cards = db::get_cards(con, &get_cards)?;

    let has_more = db_cards.iter().len() as i32 > limit;
    let last_cursor = match get_cards.get_random {
      Some(true) => Some(get_cards.previous_cursor.unwrap_or(0) + limit),
      _ => db_cards.iter().nth(limit as usize - 1).map(|r| r.id),
    };

    let db_cards = db_cards
      .iter()
      .take(limit as usize)
      .map(|c| Card {
        id: c.id,
        color: match c.is_black {
          true => CardColor::Black,
          false => CardColor::White,
        },
        format_text: c.format_text.to_owned(),
        set: SetInfo {
          id: c.parent_set_id,
          name: c.parent_set_name.to_owned(),
        },
        total_votes: c.total_votes,
        average_rating: c.average_rating,
      })
      .collect::<Vec<_>>();

    Ok(CardResult {
      results: db_cards,
      has_next_page: has_more,
      last_cursor: last_cursor,
      random_seed: get_cards.random_seed,
    })
  }

  /// To get cards belonging to a specific set
  /// The return types on this are different than the subsequent
  /// `sets` field to reduce nested query results, and nested pagination
  ///
  /// This allows finer control of where the cards are coming from,
  /// as well as allowing cards to be queried in a *shuffled* order
  fn field_set(
    &self,
    executor: &Executor<'_, Context>,
    _: &QueryTrail<'_, Set, Walked>,
    id: ID,
  ) -> Result<Set, GqlError> {
    let set = db::get_set_by_id(
      &executor.context().db,
      id.parse().expect("Id was not valid"),
    )?;
    Ok(Set {
      id: set.id,
      name: set.name,
    })
  }

  /// This returns all of the card sets within the database,
  /// or the matched sets when using the `search` parameter.
  /// `search` allows for a full-text-search of the set name
  fn field_sets(
    &self,
    executor: &Executor<'_, Context>,
    _: &QueryTrail<'_, SetResult, Walked>,
    search: Option<String>,
    pagination: Pagination,
  ) -> Result<SetResult, GqlError> {
    let limit = pagination.page_size;

    if limit > 1000 || limit < 0 {
      return Err(GqlError::LimitOutOfBounds);
    }

    let mut get_sets = GetSets::default();
    get_sets.n_results = Some(limit);
    get_sets.search = search;

    match pagination.cursor.map(|v| i32::from_encoded_id(v)) {
      Some(Ok(v)) => get_sets.cursor = Some(v),
      Some(Err(e)) => {
        return Err(e.into());
      }
      _ => get_sets.cursor = None,
    }

    let con = &executor.context().db;
    let db_sets = db::get_sets(con, &get_sets)?;

    let has_more = db_sets.iter().len() as i32 > limit;
    let last_cursor = db_sets.iter().nth(limit as usize - 1).map(|r| r.id);

    let db_sets = db_sets
      .iter()
      .take(limit as usize)
      .map(|s| SetInfo {
        id: s.id,
        name: s.name.to_owned(),
      })
      .collect::<Vec<_>>();

    Ok(SetResult {
      results: db_sets,
      has_next_page: has_more,
      last_cursor: last_cursor,
    })
  }

  fn field_license(&self, _: &Executor<'_, Context>) -> Result<Url, GqlError> {
    Ok(Url::parse(
      "https://creativecommons.org/licenses/by-nc-sa/2.0/legalcode",
    )?)
  }

  fn field_api_version(&self, _: &Executor<'_, Context>) -> Result<String, GqlError> {
    Ok(String::from("0.1.0"))
  }

  fn field_authors(&self, _: &Executor<'_, Context>) -> Result<Vec<String>, GqlError> {
    Ok(vec![
      String::from("Nicholas Dolan"),
      String::from("Cameron Otts"),
      String::from("Grace Chenevert"),
      String::from("Aaron Dentro"),
      String::from("Patrick Dolan"),
    ])
  }

  fn field_cards_against_humanity(
    &self,
    _: &Executor<'_, Context>,
    _: &QueryTrail<'_, CardsAgainstHumanity, Walked>,
  ) -> Result<CardsAgainstHumanity, GqlError> {
    Ok(CardsAgainstHumanity {})
  }
}

pub struct CardRatingResult {
  id: i32,
  rating: f32,
  total_votes: i32,
  average_rating: f32,
}

impl CardRatingResultFields for CardRatingResult {
  fn field_id(&self, _: &Executor<'_, Context>) -> Result<ID, GqlError> {
    Ok(ID::from(self.id.to_string()))
  }

  fn field_rating(&self, _: &Executor<'_, Context>) -> Result<f64, GqlError> {
    Ok(self.rating.into())
  }

  fn field_total_votes(&self, _: &Executor<'_, Context>) -> Result<i32, GqlError> {
    Ok(self.total_votes)
  }

  fn field_average_rating(&self, _: &Executor<'_, Context>) -> Result<f64, GqlError> {
    Ok(self.average_rating.into())
  }
}

/// Not currently implemented
pub struct Mutation {}

impl MutationFields for Mutation {
  fn field_add_card(
    &self,
    executor: &Executor<'_, Context>,
    _: &QueryTrail<'_, CardOperation, Walked>,
    card: CreateCard,
  ) -> Result<CardOperation, GqlError> {
    if card.format_text == "" {
      return Err(GqlError::EmptyFormatText);
    }

    let con = &executor.context().db;
    let card_create_result = db::add_card(
      con,
      &AddCard {
        user_id: executor.context().authenticated_user_id,
        format_text: card.format_text.clone(),
        is_black: match card.color {
          CardColor::Black => true,
          CardColor::White => false,
        },
      },
    )?;

    Ok(CardOperation {
      id: card_create_result.id,
      format_text: card.format_text,
      color: card.color,
    })
  }

  fn field_rate_card(
    &self,
    executor: &Executor<'_, Context>,
    _: &QueryTrail<'_, CardRatingResult, Walked>,
    rating: CardRating,
  ) -> Result<CardRatingResult, GqlError> {
    if rating.rating < 0_f64 || rating.rating > 1_f64 {
      return Err(GqlError::RatingOutOfBounds);
    }

    let user_id = executor.context().authenticated_user_id;
    let card_id = rating.id.parse()?;
    let rating = rating.rating as f32;

    let add_card_rating = AddCardRating {
      user_id,
      card_id,
      rating,
    };

    let rating_result = db::add_user_rating_to_card(&executor.context().db, &add_card_rating)?;

    Ok(CardRatingResult {
      id: card_id,
      rating: rating,
      total_votes: rating_result.total_votes,
      average_rating: rating_result.average_rating,
    })
  }

  fn field_rate_card_combo(
    &self,
    executor: &Executor<'_, Context>,
    card_rating: CardComboRating,
  ) -> Result<OperationResult, GqlError> {
    if card_rating.rating < 0_f64 || card_rating.rating > 1_f64 {
      return Err(GqlError::RatingOutOfBounds);
    }

    if card_rating.ordinal < 0_i32 {
      return Err(GqlError::NegativeOrdinal);
    }

    let add_card_rating_combination = AddCardRatingCombination {
      user_id: executor.context().authenticated_user_id,
      white_card_id: card_rating.white_card.parse().expect("Must be a valid ID"),
      black_card_id: card_rating.black_card.parse().expect("Must be a valid ID"),
      rating: card_rating.rating as f32,
      ordinal: card_rating.ordinal,
    };
    db::add_user_rate_card_combination(&executor.context().db, &add_card_rating_combination)?;
    Ok(OperationResult::Ok)
  }
}

fn playground() -> HttpResponse {
  let html = playground_source("");
  HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(html)
}

async fn graphql(
  schema: Data<Arc<Schema>>,
  data: Json<GraphQLRequest>,
  db_pool: Data<Pool>,
) -> Result<HttpResponse, AWError> {
  let ctx = Context {
    db: db_pool,
    authenticated_user_id: 1,
  };

  let res = web::block(move || {
    let res = data.execute(&schema, &ctx);
    Ok::<_, SError>(to_string(&res)?)
  })
  .await?;
  Ok(
    HttpResponse::Ok()
      .content_type("application/json")
      .body(res),
  )
}

pub fn register(config: &mut ServiceConfig) {
  let schema = Arc::new(Schema::new(Query {}, Mutation {}));

  config
    .data(schema)
    .route("/", web::post().to(graphql))
    .route("/", web::get().to(playground));
}

pub struct CardsAgainstHumanity {}

impl CardsAgainstHumanityFields for CardsAgainstHumanity {
  fn field_url(&self, _: &Executor<'_, Context>) -> Result<Url, GqlError> {
    Ok(Url::parse("https://cardsagainsthumanity.com/")?)
  }

  fn field_license(&self, _: &Executor<'_, Context>) -> Result<Url, GqlError> {
    Ok(Url::parse(
      "https://creativecommons.org/licenses/by-nc-sa/2.0/legalcode",
    )?)
  }

  fn field_theme_song(&self, _: &Executor<'_, Context>) -> Result<Url, GqlError> {
    Ok(Url::parse(
      "https://soundcloud.com/cards-against-humanity/a-good-game-of-cards",
    )?)
  }
}

#[derive(Debug, Clone)]
pub enum GqlError {
  DecodeError,
  EmptyFormatText,
  InvalidID,
  LimitOutOfBounds,
  NegativeOrdinal,
  RatingOutOfBounds,
  UnexpectedError,
  UrlParseError(ParseError),
}

impl IntoFieldError for GqlError {
  fn into_field_error(self) -> FieldError {
    FieldError::from(match self {
      GqlError::DecodeError => "Provided ID value was not a valid format",
      GqlError::EmptyFormatText => "Format text cannot be empty",
      GqlError::InvalidID => "ID Field not a valid ID type",
      GqlError::LimitOutOfBounds => "0 ≤ Page Size ≤ 1000",
      GqlError::NegativeOrdinal => "Ordinal cannot be negative",
      GqlError::RatingOutOfBounds => "0 ≤ Rating ≤ 1",
      GqlError::UrlParseError(_) => "Tried to parse an invalid URL",
      _ => "Server Error!",
    })
  }
}

impl From<ParseError> for GqlError {
  fn from(e: ParseError) -> GqlError {
    GqlError::UrlParseError(e)
  }
}

impl From<AWError> for GqlError {
  fn from(_: AWError) -> GqlError {
    GqlError::UnexpectedError
  }
}

impl From<ParseIntError> for GqlError {
  fn from(_: ParseIntError) -> GqlError {
    GqlError::InvalidID
  }
}

impl From<DecodeError> for GqlError {
  fn from(_: DecodeError) -> GqlError {
    GqlError::DecodeError
  }
}
