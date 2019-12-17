use crate::{
  db::{self, GetCards, GetSets, Pool},
  Context,
};
use actix_web::{web, Error, HttpResponse};
use base64::{decode, encode};
use juniper::http::playground::playground_source;
use juniper::{graphql_value, http::GraphQLRequest, Executor, FieldResult, ID};
use juniper_from_schema::graphql_schema_from_file;
use std::panic;
use std::sync::Arc;
use url::Url;

impl juniper::Context for Context {}

graphql_schema_from_file!("schema.graphql");

trait ToJuniperID {
  fn to_id(&self) -> ID;
  fn from_id(id: ID) -> Self;
}

impl ToJuniperID for f32 {
  fn to_id(&self) -> ID {
    let encoding = self.to_be_bytes();
    ID::new(encode(&encoding))
  }

  fn from_id(id: ID) -> f32 {
    let decoded = decode(&id.to_string()).unwrap();
    let decoded = decoded
      .iter()
      .take(4)
      .fold((0, [0; 4]), |(i, mut acc), &x| {
        acc[i] = x;
        (i + 1, acc)
      });
    f32::from_be_bytes(decoded.1)
  }
}

impl ToJuniperID for i32 {
  fn to_id(&self) -> ID {
    let encoding = self.to_be_bytes();
    ID::new(encode(&encoding))
  }

  fn from_id(id: ID) -> i32 {
    let decoded_v = decode(&id.to_string()).unwrap();
    decoded_v.iter().fold(0, |acc, &x| (acc << 8) + x as i32)
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
  fn field_id(&self, _: &Executor<'_, Context>) -> FieldResult<ID> {
    Ok(ID::from(self.id.to_string()))
  }

  /// Format text includes basic HTML markdown for
  /// text-decorations. Cards are formatted with a
  /// `<prompt/>` tag in place of '_', or the like
  /// to reduce confusion
  fn field_format_text(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
    Ok(&self.format_text)
  }

  /// Cards should be partitioned by clients for greater
  /// flexibility
  fn field_color(&self, _: &Executor<'_, Context>) -> FieldResult<CardColor> {
    Ok(self.color)
  }

  fn field_set(
    &self,
    _: &Executor<'_, Context>,
    _: &QueryTrail<'_, SetInfo, Walked>,
  ) -> FieldResult<&SetInfo> {
    Ok(&self.set)
  }

  fn field_total_votes(&self, _: &Executor<'_, Context>) -> FieldResult<i32> {
    Ok(self.total_votes)
  }

  fn field_average_rating(&self, _: &Executor<'_, Context>) -> FieldResult<Option<f64>> {
    Ok(self.average_rating.map(|v| v.into()))
  }
}

pub struct CardOperation {
  id: i32,
  format_text: String,
  color: CardColor,
  total_votes: i32,
  average_rating: Option<f32>,
}
impl CardOperationFields for CardOperation {
  fn field_id(&self, _: &Executor<'_, Context>) -> FieldResult<ID> {
    Ok(ID::from(self.id.to_string()))
  }

  fn field_format_text(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
    Ok(&self.format_text)
  }

  fn field_color(&self, _: &Executor<'_, Context>) -> FieldResult<CardColor> {
    Ok(self.color)
  }

  fn field_total_votes(&self, _: &Executor<'_, Context>) -> FieldResult<i32> {
    Ok(self.total_votes)
  }

  fn field_average_rating(&self, _: &Executor<'_, Context>) -> FieldResult<Option<f64>> {
    Ok(self.average_rating.map(|v| v.into()))
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
  ) -> FieldResult<&Vec<Card>> {
    Ok(&self.results)
  }

  fn field_last_cursor(&self, _: &Executor<'_, Context>) -> FieldResult<Option<ID>> {
    Ok(self.last_cursor.map(|v| v.to_id()))
  }

  fn field_has_next_page(&self, _: &Executor<'_, Context>) -> FieldResult<bool> {
    Ok(self.has_next_page)
  }

  fn field_random_seed(&self, _: &Executor<'_, Context>) -> FieldResult<Option<ID>> {
    Ok(self.random_seed.map(|c| c.to_id()))
  }
}

pub struct Set {
  id: i32,
  name: String,
}

impl SetFields for Set {
  fn field_id(&self, _: &Executor<'_, Context>) -> FieldResult<ID> {
    Ok(ID::from(self.id.to_string()))
  }

  fn field_name(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
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
  ) -> FieldResult<Option<CardResult>> {
    if pagination.page_size > 1000 {
      return FieldResult::Err(juniper::FieldError::new(
        "Page Size must be <= 1000 cards per query",
        graphql_value!({"validation error": "Page_Size must be <= 1000 cards per query"}),
      ));
    } else if pagination.page_size < 0 {
      return FieldResult::Err(juniper::FieldError::new(
        "Page Size cannot be negative",
        graphql_value!({"validation error": "Page_Size cannot be negative"}),
      ));
    }

    let mut get_cards = GetCards::default();
    get_cards.n_cards = Some(pagination.page_size + 1);
    get_cards.search = search;
    get_cards.card_sets = Some(vec![self.id]);
    get_cards.previous_cursor = pagination.cursor.map(|v| i32::from_id(v));

    match card_color {
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

      if let Some(s) = pagination.random_seed {
        get_cards.random_seed = Some(f32::from_id(s));
      } else {
        get_cards.random_seed = Some(rand::random::<f32>());
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
  fn field_id(&self, _: &Executor<'_, Context>) -> FieldResult<ID> {
    Ok(ID::from(self.id.to_string()))
  }

  fn field_name(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
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
  ) -> FieldResult<&Vec<SetInfo>> {
    Ok(&self.results)
  }

  fn field_last_cursor(&self, _: &Executor<'_, Context>) -> FieldResult<Option<ID>> {
    Ok(self.last_cursor.map(|c| c.to_id()))
  }

  fn field_has_next_page(&self, _: &Executor<'_, Context>) -> FieldResult<bool> {
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
  ) -> FieldResult<CardResult> {
    // Error handling
    let limit = pagination.page_size;

    if limit > 1000 {
      return FieldResult::Err(juniper::FieldError::new(
        "Limit must be <= 1000 cards per query",
        graphql_value!({"validation error": "Limit must be <= 1000 cards per query"}),
      ));
    } else if limit < 0 {
      return FieldResult::Err(juniper::FieldError::new(
        "Limit cannot be negative",
        graphql_value!({"validation error": "Limit cannot be negative"}),
      ));
    }

    let mut get_cards = GetCards::default();

    get_cards.n_cards = Some(limit + 1);

    if let Some(v) = set_ids {
      get_cards.card_sets = Some(v.iter().map(|i| i.parse().unwrap()).collect());
    }

    get_cards.previous_cursor = pagination.cursor.map(|v| i32::from_id(v));
    get_cards.search = search;

    get_cards.user_submitted = match card_source {
      CardSource::All => None,
      CardSource::User => Some(true),
      CardSource::Official => Some(false),
      _ => unreachable!()
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

      if let Some(s) = pagination.random_seed {
        get_cards.random_seed = Some(f32::from_id(s));
      } else {
        get_cards.random_seed = Some(rand::random::<f32>());
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
  ) -> FieldResult<Set> {
    let set = db::get_set_by_id(&executor.context().db, id.parse().expect("Id was not valid"))?;
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
  ) -> FieldResult<SetResult> {
    let limit = pagination.page_size;

    if limit > 1000 {
      return FieldResult::Err(juniper::FieldError::new(
        "Limit must be <= 1000 cards per query",
        graphql_value!({"validation error": "Limit must be <= 1000 cards per query"}),
      ));
    } else if limit < 0 {
      return FieldResult::Err(juniper::FieldError::new(
        "Limit cannot be negative",
        graphql_value!({"validation error": "Limit cannot be negative"}),
      ));
    }

    let mut get_sets = GetSets::default();
    get_sets.n_results = Some(limit);
    get_sets.search = search;

    get_sets.cursor = pagination.cursor.map(|v| i32::from_id(v));

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

  fn field_license(&self, _: &Executor<'_, Context>) -> FieldResult<Url> {
    Ok(Url::parse(
      "https://creativecommons.org/licenses/by-nc-sa/2.0/legalcode",
    )?)
  }

  fn field_api_version(&self, _: &Executor<'_, Context>) -> FieldResult<String> {
    Ok(String::from("0.1.0"))
  }

  fn field_authors(&self, _: &Executor<'_, Context>) -> FieldResult<Vec<String>> {
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
  ) -> FieldResult<CardsAgainstHumanity> {
    Ok(CardsAgainstHumanity {})
  }
}

/// Not currently implemented
pub struct Mutation {}

impl MutationFields for Mutation {
  fn field_add_card(
    &self,
    _: &Executor<'_, Context>,
    _: &QueryTrail<'_, CardOperation, Walked>,
    _: CreateCard,
  ) -> FieldResult<CardOperation> {
    unimplemented!()
  }

  fn field_rate_card(
    &self,
    _: &Executor<'_, Context>,
    _: &QueryTrail<'_, CardOperation, Walked>,
    _: CardRating,
  ) -> FieldResult<CardOperation> {
    unimplemented!()
  }
}

fn playground() -> HttpResponse {
  let html = playground_source("");
  HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(html)
}

async fn graphql(
  schema: web::Data<Arc<Schema>>,
  data: web::Json<GraphQLRequest>,
  db_pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
  let ctx = Context { db: db_pool, authenticated_user_id: 1 };

  let res = web::block(move || {
    let res = data.execute(&schema, &ctx);
    Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
  })
  .await?;
  Ok(
    HttpResponse::Ok()
      .content_type("application/json")
      .body(res),
  )
}

pub fn register(config: &mut web::ServiceConfig) {
  let schema = std::sync::Arc::new(Schema::new(Query {}, Mutation {}));

  config
    .data(schema)
    .route("/", web::post().to(graphql))
    .route("/", web::get().to(playground));
}

pub struct CardsAgainstHumanity {}

impl CardsAgainstHumanityFields for CardsAgainstHumanity {
  fn field_url(&self, _: &Executor<'_, Context>) -> FieldResult<Url> {
    Ok(Url::parse("https://cardsagainsthumanity.com/")?)
  }

  fn field_license(&self, _: &Executor<'_, Context>) -> FieldResult<Url> {
    Ok(Url::parse(
      "https://creativecommons.org/licenses/by-nc-sa/2.0/legalcode",
    )?)
  }

  fn field_theme_song(&self, _: &Executor<'_, Context>) -> FieldResult<Url> {
    Ok(Url::parse(
      "https://soundcloud.com/cards-against-humanity/a-good-game-of-cards"
    )?)
  }
}
