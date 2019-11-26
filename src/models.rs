use chrono::{NaiveDateTime};
use juniper_eager_loading::impl_load_from_for_diesel_pg;
use diesel::pg::PgConnection;
use diesel::prelude::*;

use super::DbCon;
use super::schema::bb::{card, parent_set, parent_set_card};

#[derive(Clone, Queryable)]
pub struct Card {
    pub id: i32,
    pub is_black: bool,
    pub format_text: String,
    is_active: bool,
    last_modified: NaiveDateTime
}

#[derive(Clone, Queryable)]
pub struct ParentSet {
    pub id: i32,
    pub name: String,
    is_active: bool,
    last_modified: NaiveDateTime
}

#[derive(Queryable)]
pub struct ParentSetCard {
    parent_set_id: i32,
    card_id: i32,
    is_active: bool,
    last_modified: NaiveDateTime
}

impl juniper_eager_loading::LoadFrom<ParentSet> for Card {
  type Error = diesel::result::Error;
  type Connection = PgConnection;

  fn load(
    parent_sets: &[ParentSet],
    _: &(),
    ctx: &Self::Connection,
  ) -> Result<Vec<Self>, Self::Error> {
    use super::schema::bb::card::dsl::*;
    use super::schema::bb::parent_set_card::dsl::*;

    use diesel::pg::expression::dsl::any;

    let parent_set_ids = parent_sets
      .iter()
      .map(|s| s.id)
      .collect::<Vec<_>>();

    let card_ids = parent_set_card
      .filter(parentsetid.eq(any(parent_set_ids)))
      .select(cardid);

    card
      .filter(id.eq(any(card_ids)))
      .load::<Card>(ctx)
  }
}
