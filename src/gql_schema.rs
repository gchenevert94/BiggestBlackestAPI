use juniper::{RootNode, ID, FieldResult};
use chrono::{NaiveDateTime};
use dotenv::dotenv;
use diesel::PgConnection;
use diesel::prelude::*;
use std::env;
use super::schema::bb::{ Card, ParentSet };
use super::{Context};


#[derive(Queryable)]
pub struct CardModel {
    pub id: i32,
    pub is_black: bool,
    pub format_text: String,
    is_active: bool,
    last_modified: NaiveDateTime
}

#[juniper::object]
impl CardModel {
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

#[derive(Queryable)]
pub struct CardSet {
    pub id: i32,
    pub name: String,
    is_active: bool,
    last_modified: NaiveDateTime
}

#[juniper::object(
    Context = Context
)]
impl CardSet {
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
    pub fn white_cards(&self, limit: i32, context: &Context) -> FieldResult<Vec<CardModel>> {
        use diesel::pg::expression::dsl::any;
        use super::schema::bb::Card::dsl::*;
        use super::schema::bb::ParentSetCard::dsl::*;

        let conn = &context.db_con;

        let card_ids = ParentSetCard
            .filter(parentsetid.eq(self.id))
            .select(cardid);

        match Card
            .filter(id.eq(any(card_ids)))
            .filter(isblack.eq(false))
            .order_by(id)
            .limit(limit.into())
            .load::<CardModel>(conn) {
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
    pub fn black_cards(&self, limit: i32, context: &Context) -> FieldResult<Vec<CardModel>> {
        use diesel::pg::expression::dsl::any;
        use super::schema::bb::Card::dsl::*;
        use super::schema::bb::ParentSetCard::dsl::*;

        let conn = &context.db_con;

        let card_ids = ParentSetCard
            .filter(parentsetid.eq(self.id))
            .select(cardid);

        match Card
            .filter(id.eq(any(card_ids)))
            .filter(isblack.eq(true))
            .order_by(id)
            .limit(limit.into())
            .load::<CardModel>(conn) {
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
    pub fn card_sets(title: Option<String>, context: &Context) -> FieldResult<Vec<CardSet>> {
        use super::schema::bb::ParentSet::dsl::*;
        let conn = &context.db_con;

        match title {
            Some(t) => match ParentSet
                .filter(name.ilike(t + "%"))
                .load::<CardSet>(conn) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(e)?
                },
            None => match ParentSet
                .load::<CardSet>(conn) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(e)?
                },
        }
    }

    pub fn card_set(id: ID, context: &Context) -> FieldResult<CardSet> {
        use super::schema::bb::ParentSet::dsl::*;
        let conn = &context.db_con;
        match ParentSet
            .find(id)
            .first::<CardSet>(conn) {
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
    pub fn black_cards(limit: i32, context: &Context) -> FieldResult<Vec<CardModel>> {
        use super::schema::bb::Card::dsl::*;
        let conn = &context.db_con;
        match Card
            .filter(isblack.eq(true))
            .limit(i64::from(limit))
            .load::<CardModel>(conn) {
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
    pub fn white_cards(limit: i32, context: &Context) -> FieldResult<Vec<CardModel>> {
        use super::schema::bb::Card::dsl::*;
        let conn = &context.db_con;
        match Card
            .filter(isblack.eq(false))
            .order_by(id)
            .limit(i64::from(limit))
            .load::<CardModel>(conn) {
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
