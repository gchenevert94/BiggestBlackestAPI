use crate::models::{
  AddCard, AddCardRating, AddCardRatingCombination, AddCardRatingResult, AddCardResult,
  GetCardResults, GetCards, GetSetResults, GetSets,
};
use actix_web::{error::ErrorInternalServerError, Error as AWError};
use fallible_iterator::FallibleIterator;
use postgres::types::Type;
use r2d2_postgres::PostgresConnectionManager;

pub type ConnectionManager = PostgresConnectionManager<postgres::NoTls>;
pub type Pool = r2d2_postgres::r2d2::Pool<ConnectionManager>;
pub type PgConfig = postgres::Config;

pub struct PoolConfiguration {
  min_idle: Option<u32>,
  max_size: u32,
}

impl PoolConfiguration {
  pub fn default() -> PoolConfiguration {
    PoolConfiguration {
      min_idle: None,
      max_size: 4,
    }
  }
}

/// Creates an r2d2 Pool using the provided configuration and connection
/// information. This should be replaced with `bb8` and `tokio_postgres`
/// when the crates are compatible with actix_rt's `tokio` runtime, or is
/// no longer in alpha.
pub fn create_pool(pg_config: PgConfig, pool_config: &PoolConfiguration) -> Pool {
  let r2d2_manager = ConnectionManager::new(pg_config, postgres::NoTls);

  Pool::builder()
    .min_idle(pool_config.min_idle)
    .max_size(pool_config.max_size)
    .build(r2d2_manager)
    .expect("Unable to build connection pool")
}

/// Get cards database call. Calls the prepare_typed method to ensure our data
/// types match the SQL types used in the statement. Future implementation should
/// ensure that the preparation of these statements is cached, possibly configured
/// in the `create_pool` method using the `r2d2::CustomizeConnection` trait.
///
/// Uses the database function `bb.get_cards(search, filter_black, previous_cursor, n_cards, card_sets, get_random, random_seed, user_submitted)`
///
/// `Row::get()` accepts "Column Name", or Ordinal_i32. To add minor performance,
/// we are using the ordinal to access the value of the column, as well as choosing
/// not to use the `SELECT *` syntax.
/// This does replace maintainability for performance, but the goal is currently
/// to focus on performance and control.
pub fn get_cards(pool: &Pool, query: &GetCards) -> Result<Vec<GetCardResults>, AWError> {
  let mut client = pool
    .clone()
    .get()
    .map_err(|e| ErrorInternalServerError(e))?;
  let stmt = client
        .prepare_typed(
            "SELECT id, format_text, is_black, parent_set_id, parent_set_name, total_votes, average_rating FROM bb.get_cards($1, $2, $3, $4, $5, $6, $7, $8)",
            &[
                Type::TEXT,
                Type::BOOL,
                Type::INT4,
                Type::INT4,
                Type::INT4_ARRAY,
                Type::BOOL,
                Type::FLOAT4,
                Type::BOOL
            ],)
        .map_err(|e| ErrorInternalServerError(e))?;

  let results = client
    .query_iter(
      &stmt,
      &[
        &query.search,
        &query.filter_black,
        &query.previous_cursor,
        &query.n_cards,
        &query.card_sets,
        &query.get_random,
        &query.random_seed,
        &query.user_submitted,
      ],
    )
    .map_err(|e| ErrorInternalServerError(e))?;
  Ok(
    results
      .map(|r| {
        Ok(GetCardResults {
          id: r.get::<_, i32>(0),
          format_text: r.get::<_, String>(1),
          is_black: r.get::<_, bool>(2),
          parent_set_id: r.get::<_, i32>(3),
          parent_set_name: r.get::<_, String>(4),
          total_votes: r.get::<_, i32>(5),
          average_rating: r.get::<_, Option<f32>>(6),
        })
      })
      .map_err(|e| ErrorInternalServerError(e))
      .collect::<Vec<_>>()?,
  )
}

/// Get sets database call. Calls the prepare_typed method to ensure our data
/// types match the SQL types used in the statement. Future implementation should
/// ensure that the preparation of these statements is cached, possibly configured
/// in the `create_pool` method using the `r2d2::CustomizeConnection` trait.
///
/// Uses the database function `bb.get_sets(search, limit, cursor)`
///
/// `Row::get()` accepts "Column Name", or Ordinal_i32. To add minor performance,
/// we are using the ordinal to access the value of the column, as well as choosing
/// not to use the `SELECT *` syntax.
/// This does replace maintainability for performance, but the goal is currently
/// to focus on performance and control.
pub fn get_sets(pool: &Pool, query: &GetSets) -> Result<Vec<GetSetResults>, AWError> {
  let client = pool.clone();
  let mut client = client.get().map_err(|e| ErrorInternalServerError(e))?;
  let stmt = client
    .prepare_typed(
      "SELECT id, name FROM bb.get_sets($1, $2, $3)",
      &[Type::TEXT, Type::INT4, Type::INT4],
    )
    .map_err(|e| ErrorInternalServerError(e))?;
  let results = client
    .query_iter(&stmt, &[&query.search, &query.n_results, &query.cursor])
    .map_err(|e| ErrorInternalServerError(e))?;

  Ok(
    results
      .map(|r| {
        Ok(GetSetResults {
          id: r.get::<_, i32>(0),
          name: r.get::<_, String>(1),
        })
      })
      .map_err(|e| ErrorInternalServerError(e))
      .collect::<Vec<_>>()?,
  )
}

/// Get sets by ID database call. Calls the prepare_typed method to ensure our data
/// types match the SQL types used in the statement. Future implementation should
/// ensure that the preparation of these statements is cached, possibly configured
/// in the `create_pool` method using the `r2d2::CustomizeConnection` trait.
///
/// `Row::get()` accepts "Column Name", or Ordinal_i32. To add minor performance,
/// we are using the ordinal to access the value of the column, as well as choosing
/// not to use the `SELECT *` syntax.
/// This does replace maintainability for performance, but the goal is currently
/// to focus on performance and control.
pub fn get_set_by_id(pool: &Pool, query: i32) -> Result<GetSetResults, AWError> {
  let client = pool.clone();
  let mut client = client.get().map_err(|e| ErrorInternalServerError(e))?;
  let stmt = client
    .prepare_typed(
      "SELECT id, name FROM bb.parent_set WHERE id = $1",
      &[Type::INT4],
    )
    .map_err(|e| ErrorInternalServerError(e))?;
  let result = &client
    .query(&stmt, &[&query])
    .map_err(|e| ErrorInternalServerError(e))?[0];

  Ok(GetSetResults {
    id: result.get::<_, i32>(0),
    name: result.get::<_, String>(1),
  })
}

/// Create card database call. Calls the prepare_typed method to ensure our data
/// types match the SQL types used in the statement. Future implementation should
/// ensure that the preparation of these statements is cached, possibly configured
/// in the `create_pool` method using the `r2d2::CustomizeConnection` trait.
///
/// Uses the `bb.create_card(format_text, is_black, created_by_user_id)` method.
/// The database should return an error when submitting a card that has already
/// been created.
///
/// `Row::get()` accepts "Column Name", or Ordinal_i32. To add minor performance,
/// we are using the ordinal to access the value of the column, as well as choosing
/// not to use the `SELECT *` syntax.
/// This does replace maintainability for performance, but the goal is currently
/// to focus on performance and control.
pub fn add_card(pool: &Pool, query: &AddCard) -> Result<AddCardResult, AWError> {
  let client = pool.clone();
  let mut client = client.get().map_err(|e| ErrorInternalServerError(e))?;
  let stmt = client
    .prepare_typed(
      "select create_card from bb.create_card($1, $2, $3)",
      &[Type::TEXT, Type::BOOL, Type::INT4],
    )
    .map_err(|e| ErrorInternalServerError(e))?;

  let result = &client
    .query(
      &stmt,
      &[&query.format_text, &query.is_black, &query.user_id],
    )
    .map_err(|e| ErrorInternalServerError(e))?[0];

  Ok(AddCardResult {
    id: result.get::<_, i32>(0),
  })
}

/// Add User Rating to Card database call. Calls the prepare_typed method to ensure our data
/// types match the SQL types used in the statement. Future implementation should
/// ensure that the preparation of these statements is cached, possibly configured
/// in the `create_pool` method using the `r2d2::CustomizeConnection` trait.
///
/// Uses the `bb.user_rate_card(user_id, card_id, rating)` method.
/// The database should ensure that an UPSERT is executed to change a user's rating
/// of a card when a conflict occurs. This should additionally update in-place the
/// card's total_votes and average_rating fields accordingly (Insertion of a new record/Updating an already-cast vote)
/// This function also does not return a result set, but rather out parameters to
/// ensure that only a single record is returned from the resulting query.
///
/// `Row::get()` accepts "Column Name", or Ordinal_i32. To add minor performance,
/// we are using the ordinal to access the value of the column, as well as choosing
/// not to use the `SELECT *` syntax.
/// This does replace maintainability for performance, but the goal is currently
/// to focus on performance and control.
pub fn add_user_rating_to_card(
  pool: &Pool,
  query: &AddCardRating,
) -> Result<AddCardRatingResult, AWError> {
  let client = pool.clone();
  let mut client = client.get().map_err(|e| ErrorInternalServerError(e))?;
  let stmt = client
    .prepare_typed(
      "SELECT out_total_votes, out_average_rating FROM bb.user_rate_card($1, $2, $3)",
      &[Type::INT4, Type::INT4, Type::FLOAT4],
    )
    .map_err(|e| ErrorInternalServerError(e))?;

  let result = &client
    .query(&stmt, &[&query.user_id, &query.card_id, &query.rating])
    .map_err(|e| ErrorInternalServerError(e))?[0];

  Ok(AddCardRatingResult {
    total_votes: result.get::<_, i32>(0),
    average_rating: result.get::<_, f32>(1),
  })
}

/// Add User Rating to Card Combination database call. Calls the prepare_typed method to ensure our data
/// types match the SQL types used in the statement. Future implementation should
/// ensure that the preparation of these statements is cached, possibly configured
/// in the `create_pool` method using the `r2d2::CustomizeConnection` trait.
///
/// Uses the `bb.user_rate_card_combination(user_id, black_card_id, white_card, rating, ordinal)` method.
/// The database should ensure that an UPSERT is executed to change a user's rating
/// of a card when a conflict occurs. Unlike the `add_user_rate_card()` method,
/// this does not need to update any currently maintained statistics. The Ordinal
/// is needed for white cards with multiple prompts. There is currently no way
/// to query this information from the GraphQL API, as it is intended for analytical
/// use later.
///
/// This database function has no return value, but can throw an error (hence the `Result<(), AWError>` type)
pub fn add_user_rate_card_combination(
  pool: &Pool,
  query: &AddCardRatingCombination,
) -> Result<(), AWError> {
  let client = pool.clone();
  let mut client = client.get().map_err(|e| ErrorInternalServerError(e))?;
  let stmt = client
    .prepare_typed(
      "CALL bb.user_rate_card_combination($1, $2, $3, $4, $5)",
      &[Type::INT4, Type::INT4, Type::INT4, Type::FLOAT4, Type::INT4],
    )
    .map_err(|e| ErrorInternalServerError(e))?;

  &client
    .execute(
      &stmt,
      &[
        &query.user_id,
        &query.black_card_id,
        &query.white_card_id,
        &query.rating,
        &query.ordinal,
      ],
    )
    .map_err(|e| ErrorInternalServerError(e))?;

  Ok(())
}
