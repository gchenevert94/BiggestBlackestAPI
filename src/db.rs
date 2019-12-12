use crate::models::{GetCardResults, GetSetResults};
use actix_web::error::ErrorInternalServerError;
use actix_web::Error as AWError;
// use bb8_postgres::{self, bb8};
// use tokio_postgres::{self, types::Type};
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

pub fn create_pool(pg_config: PgConfig, pool_config: &PoolConfiguration) -> Pool {
  let r2d2_manager = ConnectionManager::new(pg_config, postgres::NoTls);

  Pool::builder()
    .min_idle(pool_config.min_idle)
    .max_size(pool_config.max_size)
    .build(r2d2_manager)
    .unwrap()
}

pub fn get_cards(pool: &Pool, query: &GetCards) -> Result<Vec<GetCardResults>, AWError> {
  let client = pool.clone();
  let mut client = client.get().map_err(|e| ErrorInternalServerError(e))?;
  let stmt = client
        .prepare_typed(
            "SELECT id, format_text, is_black, parent_set_id, parent_set_name, total_votes, average_rating FROM bb.get_cards($1, $2, $3, $4, $5, $6, $7)",
            &[
                Type::TEXT,
                Type::BOOL,
                Type::INT4,
                Type::INT4,
                Type::INT4_ARRAY,
                Type::BOOL,
                Type::FLOAT4,
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

pub struct GetSets {
  pub search: Option<String>,
  pub n_results: Option<i32>,
  pub cursor: Option<i32>,
}

impl GetSets {
  pub fn default() -> GetSets {
    GetSets {
      search: None,
      n_results: Some(100),
      cursor: None,
    }
  }
}

pub struct GetCards {
  pub search: Option<String>,
  pub filter_black: Option<bool>,
  pub previous_cursor: Option<i32>,
  pub n_cards: Option<i32>,
  pub card_sets: Option<Vec<i32>>,
  pub get_random: Option<bool>,
  pub random_seed: Option<f32>,
}

impl GetCards {
  pub fn default() -> GetCards {
    GetCards {
      search: None,
      filter_black: None,
      previous_cursor: None,
      n_cards: Some(100),
      card_sets: None,
      get_random: Some(false),
      random_seed: None,
    }
  }
}
