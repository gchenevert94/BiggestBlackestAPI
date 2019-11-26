use juniper::{RootNode, ID, FieldResult};
use chrono::{NaiveDateTime};
use diesel::pg::expression::dsl::any;
use diesel::prelude::*;
use super::{Context};
use crate::models::{Card, ParentSet};

#[juniper::object]
impl Card {
    pub fn id(&self) -> ID {
        ID::from(self.id.to_string())
    }

    pub fn is_black(&self) -> bool {
        self.is_black
    }

    pub fn format_text(&self) -> &str {
        &self.format_text
    }
}

#[juniper::object(
    Context = Context
)]
impl ParentSet {
    pub fn id(&self) -> ID {
        ID::from(self.id.to_string())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    #[graphql(
        arguments(
            limit(
                default = 100,
                description = "Number of cards to query"
            )
        )
    )]
    pub fn white_cards(&self, limit: i32, context: &Context) -> FieldResult<Vec<Card>> {
        use super::schema::bb::card::dsl::*;
        use super::schema::bb::parent_set_card::dsl::*;

        let conn = &context.db_con;

        let card_ids = parent_set_card
            .filter(parentsetid.eq(self.id))
            .select(cardid);

        match card
            .filter(id.eq(any(card_ids)))
            .filter(isblack.eq(false))
            .order_by(id)
            .limit(limit.into())
            .load::<Card>(conn) {
                Ok(v) => Ok(v),
                Err(e) => Err(e)?
            }
    }

    #[graphql(
        arguments(
            limit(
                default = 100,
                description = "Number of cards to query"
            )
        )
    )]
    pub fn black_cards(&self, limit: i32, context: &Context) -> FieldResult<Vec<Card>> {
        use diesel::pg::expression::dsl::any;
        use super::schema::bb::card::dsl::*;
        use super::schema::bb::parent_set_card::dsl::*;

        let conn = &context.db_con;

        let card_ids = parent_set_card
            .filter(parentsetid.eq(self.id))
            .select(cardid);

        match card
            .filter(id.eq(any(card_ids)))
            .filter(isblack.eq(true))
            .order_by(id)
            .limit(limit.into())
            .load::<Card>(conn) {
                Ok(v) => Ok(v),
                Err(e) => Err(e)?
            }
    }
}


impl juniper::Context for Context {}

pub struct Query;

#[juniper::object(
    Context = Context
)]
impl Query {
    pub fn api_version() -> &'static str {
        "0.1"
    }

    #[graphql(
        arguments(
            title(
                description = "Title of the set to look up"
            )
        )
    )]
    pub fn card_sets(title: Option<String>, context: &Context) -> FieldResult<Vec<ParentSet>> {
        use super::schema::bb::parent_set::dsl::*;
        let conn = &context.db_con;

        match title {
            Some(t) => match parent_set
                .filter(name.ilike(t + "%"))
                .load::<ParentSet>(conn) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(e)?
                },
            None => match parent_set
                .load::<ParentSet>(conn) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(e)?
                },
        }
    }

    pub fn card_set(id: ID, context: &Context) -> FieldResult<ParentSet> {
        use super::schema::bb::parent_set::dsl::*;
        let conn = &context.db_con;
        match parent_set
            .find(id)
            .first::<ParentSet>(conn) {
                Ok(v) => Ok(v),
                Err(e) => Err(e)?
            }
    }

    #[graphql(
        arguments(
            limit(
                default = 100,
                description = "Number of cards to query"
            )
        )
    )]
    pub fn black_cards(limit: i32, context: &Context) -> FieldResult<Vec<Card>> {
        use super::schema::bb::card::dsl::*;
        let conn = &context.db_con;
        match card
            .filter(isblack.eq(true))
            .limit(i64::from(limit))
            .load::<Card>(conn) {
                Ok(v) => Ok(v),
                Err(e) => Err(e)?
            }
    }

    #[graphql(
        arguments(
            limit(
                default = 100,
                description = "Number of cards to query"
            )
        )
    )]
    pub fn white_cards(limit: i32, context: &Context) -> FieldResult<Vec<Card>> {
        use super::schema::bb::card::dsl::*;
        let conn = &context.db_con;
        match card
            .filter(isblack.eq(false))
            .order_by(id)
            .limit(i64::from(limit))
            .load::<Card>(conn) {
                Ok(v) => Ok(v),
                Err(e) => Err(e)?
            }
    }
}

pub struct Mutation;

#[juniper::object(
    Context = Context
)]
impl Mutation {
    pub fn api_version() -> &'static str {
        "0.1"
    }
}

pub type Schema = RootNode<'static, Query, Mutation>;

pub fn create_schema() -> Schema {
    Schema::new(Query {}, Mutation {})
}
