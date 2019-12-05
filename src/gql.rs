/*
    The purpose of our app to be containerized:
        Everyone deserves to run their own instance and have fun
        - Update with new OFFICIAL sources and expansions

    When we host:
        Use 100% the same utilities as the publicized container.
        Collect cards, create decks, etc.
        WE FULLY EXPECT TO STORE/KEEP OUR OWN SNAPSHOTS AND BACKUPS
            pg_dump /mybackupfolder

    Graphql schema {
        license: "",
        authors: "",
        cardsAgainstHumanity: {
            site,
            license
        }
    }
*/

/**
 * WE GONNA MAKE A BLANK DATABASE
 * CREATE MIGRATIONS TABLE
 * DOCKER SNAPSHOT
 * 
 * CREATE MIGRATION SCRIPT (init)
 * DOCKER SNAPSHOT
 * 
 * CREATE MIGRATION SCRIPTS...
 * DOCKER SNAPSHOT
 * 
 */

/// THE SOLUTION
/**
 * mkdir /usr/migrations
 * // NEW SET COMES
 * cp new_sets.sql /usr/migrations/
 * 
 * CRONJOB
 * 0 0 * * * "psql --credentials-- < /usr/migrations/ && rm /usr/migrations/"
*/


use crate::{models, db, Context, DbPool};
use actix_web::{web, Error, HttpResponse};
use base64::{decode, encode};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel_full_text_search::{plainto_tsquery, TsVectorExtensions};
use futures::future::Future;
use juniper::http::playground::playground_source;
use juniper::{http::GraphQLRequest, Executor, FieldResult, ID};
use juniper_from_schema::graphql_schema_from_file;
use std::collections::HashSet;
use std::panic;
use std::sync::Arc;
use std::convert::TryInto;

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
    average_rating: f32,
    total_votes: u32,
    set: SetInfo,
}

impl CardFields for Card {
    fn field_id(&self, _: &Executor<'_, Context>) -> FieldResult<ID> {
        Ok(ID::from(self.id.to_string()))
    }

    fn field_format_text(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.format_text)
    }

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

    fn field_total_votes(&self, _: &Executor<'_, Context>) ->
        FieldResult<i32>
    {
        Ok(self.total_votes.try_into().unwrap_or(0))
    }

    fn field_average_rating(&self, _: &Executor<'_, Context>) ->
        FieldResult<f64>
    {
        Ok(self.average_rating.into())
    }
}

pub struct CardOperation {
    id: i32,
    format_text: String,
    color: CardColor,
    total_votes: u32,
    average_rating: f32
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

    fn field_total_votes(&self, _: &Executor<'_, Context>) ->
        FieldResult<i32>
    {
        Ok(self.total_votes.try_into().unwrap_or(0))
    }

    fn field_average_rating(&self, _: &Executor<'_, Context>) ->
        FieldResult<f64>
    {
        Ok(self.average_rating.into())
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
    cards: Option<CardResult>,
}

impl SetFields for Set {
    fn field_id(&self, _: &Executor<'_, Context>) -> FieldResult<ID> {
        Ok(ID::from(self.id.to_string()))
    }

    fn field_name(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.name)
    }

    fn field_cards(
        &self,
        _: &Executor<'_, Context>,
        _: &QueryTrail<'_, CardResult, Walked>,
        _: Option<String>,
        _: Option<CardColor>,
        _: Pagination,
        _: Option<bool>,
    ) -> FieldResult<&Option<CardResult>> {
        Ok(&self.cards)
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

pub struct Query {}

impl QueryFields for Query {
    fn field_cards(
        &self,
        executor: &Executor<'_, Context>,
        trail: &QueryTrail<'_, CardResult, Walked>,
        search: Option<String>,
        color: Option<CardColor>,
        pagination: Pagination,
        set_ids: Option<Vec<juniper::ID>>,
        randomized: Option<bool>,
    ) -> FieldResult<CardResult> {
        use super::schema::bb::card::dsl::{id as cid, *};
        use super::schema::bb::parent_set;
        use super::schema::bb::parent_set::{id as psid, *};
        use super::schema::bb::parent_set_card::dsl::*;
        use diesel::pg::expression::dsl::*;

        let mut db_cards = card.order_by(cid).inner_join(parent_set_card).into_boxed();

        if let Some(v) = set_ids {
            db_cards = db_cards.filter(
                parentsetid.eq(any(v
                    .iter()
                    .map(|i| i.parse::<i32>().unwrap())
                    .collect::<Vec<_>>())),
            );
        }

        if let Some(v) = pagination.cursor {
            let decoded_v = decode(&v.to_string()).unwrap();
            let v = decoded_v.iter().fold(0, |acc, &x| (acc << 8) + x as i32);
            db_cards = db_cards.filter(cid.gt(v));
        }

        match color {
            Some(CardColor::Black) => {
                db_cards = db_cards.filter(isblack.eq(true));
            }
            Some(CardColor::White) => {
                db_cards = db_cards.filter(isblack.eq(false));
            }
            _ => {}
        }

        if let Some(r) = search {
            db_cards = db_cards.filter(text_searchable_format_text.matches(plainto_tsquery(r)));
        }

        let limit: i64 = pagination.page_size.into();
        let db_cards = db_cards
            .limit(limit + 1)
            .select((cid, isblack, formattext, parentsetid))
            .load::<(i32, bool, String, i32)>(&executor.context().db_con)?;

        let has_more = db_cards.len() as i64 > limit;
        let last_id = db_cards.iter().nth(limit as usize - 1).map(|v| v.0);

        let mut db_cards = db_cards
            .iter()
            .take(limit as usize)
            .map(|c| Card {
                id: c.0,
                color: match c.1 {
                    true => CardColor::Black,
                    false => CardColor::White,
                },
                format_text: c.2.to_owned(),
                set: SetInfo {
                    id: c.3,
                    name: String::new(),
                },
            })
            .collect::<Vec<_>>();

        trail.results().walk();
        if let Some(_) = trail.results().set().walk() {
            let set_names: HashSet<i32> = db_cards.iter().map(|c| c.set.id).collect();

            let names = parent_set::table
                .select((psid, name))
                .filter(psid.eq(any(set_names.iter().collect::<Vec<_>>())))
                .load::<(i32, String)>(&executor.context().db_con)?;

            db_cards = db_cards
                .iter()
                .map(|c| {
                    let setname = names
                        .iter()
                        .find(|(i, _)| i == &c.set.id)
                        .map(|(_, v)| v.clone())
                        .unwrap_or(String::new());
                    Card {
                        id: c.id,
                        color: c.color,
                        format_text: c.format_text.to_owned(),
                        set: SetInfo {
                            id: c.set.id,
                            name: setname,
                        },
                    }
                })
                .collect::<Vec<_>>();
        }

        Ok(CardResult {
            results: db_cards,
            has_next_page: has_more,
            last_cursor: last_id,
            random_seed: None,
        })
    }

    fn field_set(
        &self,
        executor: &Executor<'_, Context>,
        trail: &QueryTrail<'_, Set, Walked>,
        id_f: ID,
    ) -> FieldResult<Set> {
        use crate::schema::bb::card::dsl::{id as cid, *};
        use crate::schema::bb::parent_set::dsl::{id as psid, *};
        use crate::schema::bb::parent_set_card;
        use crate::schema::bb::parent_set_card::*;

        let set = parent_set
            .select((psid, name))
            .find(id_f.parse::<i32>().unwrap())
            .first::<models::ParentSet>(&executor.context().db_con)?;

        let mut set = Set {
            id: set.id,
            name: set.name,
            cards: None,
        };

        if let Some(_) = trail.cards().walk() {
            let mut cards = card
                .inner_join(parent_set_card::table)
                .filter(parentsetid.eq(set.id))
                .into_boxed::<Pg>();

            cards = cards.order_by(cardid).filter(parentsetid.eq(set.id));

            let mut limit: i64 = 10;
            if let Some(a) = panic::catch_unwind(|| Some(trail.cards_args())).unwrap_or(None) {
                let search = panic::catch_unwind(|| a.search()).unwrap_or(None);

                if let Some(v) = search {
                    cards = cards.filter(text_searchable_format_text.matches(plainto_tsquery(v)));
                }

                let pagination = panic::catch_unwind(|| Some(a.pagination())).unwrap_or(None);

                if let Some(v) = pagination {
                    if let Some(c) = v.cursor {
                        cards = cards.filter(cid.gt(i32::from_id(c)));
                    }

                    limit = v.page_size.into();
                    cards = cards.limit(1 + limit);
                }

                let color = panic::catch_unwind(|| a.color()).unwrap_or(None);

                match color {
                    Some(CardColor::Black) => {
                        cards = cards.filter(isblack.eq(true));
                    }
                    Some(CardColor::White) => {
                        cards = cards.filter(isblack.eq(false));
                    }
                    _ => {}
                }
            }

            let cards = cards
                .select((cid, isblack, formattext))
                .load::<models::Card>(&executor.context().db_con)?;

            let has_more = cards.len() as i64 > limit;
            let last_id = cards.iter().nth(limit as usize - 1).map(|v| v.id);

            set.cards = Some(CardResult {
                results: cards
                    .iter()
                    .take(limit as usize)
                    .map(|v| Card {
                        id: v.id,
                        format_text: v.format_text.to_owned(),
                        color: match v.is_black {
                            true => CardColor::Black,
                            _ => CardColor::White,
                        },
                        set: SetInfo {
                            id: set.id,
                            name: set.name.clone(),
                        },
                    })
                    .collect(),
                last_cursor: last_id,
                has_next_page: has_more,
                random_seed: None,
            });
        }

        Ok(set)
    }

    fn field_sets(
        &self,
        executor: &Executor<'_, Context>,
        _: &QueryTrail<'_, SetResult, Walked>,
        search: Option<String>,
        pagination: Pagination,
    ) -> FieldResult<SetResult> {
        use crate::schema::bb::parent_set::dsl::*;

        let mut sets = parent_set.order_by(id).into_boxed();

        if let Some(v) = search {
            sets = sets.filter(text_searchable_name.matches(plainto_tsquery(v)));
        }

        if let Some(p) = pagination.cursor {
            sets = sets.filter(id.gt(i32::from_id(p)));
        }

        let limit = pagination.page_size as i64;
        let sets = sets
            .select((id, name))
            .limit(1 + limit)
            .load::<models::ParentSet>(&executor.context().db_con)?;

        let has_more = sets.len() as i64 > limit;
        let last_id = sets.iter().nth(limit as usize - 1).map(|v| v.id);

        let sets = sets
            .iter()
            .take(limit as usize)
            .map(|s| SetInfo {
                id: s.id,
                name: s.name.to_owned(),
            })
            .collect();

        Ok(SetResult {
            results: sets,
            last_cursor: last_id,
            has_next_page: has_more,
        })
    }
}

pub struct Mutation {}

impl MutationFields for Mutation {
    fn field_add_card(
        &self,
        executor: &Executor<'_, Context>,
        _: &QueryTrail<'_, CardOperation, Walked>,
        card: CreateCard,) ->
        FieldResult<CardOperation>
    {
        unimplemented!()
    }

    fn field_rate_card(
        &self,
        executor: &Executor<'_, Context>,
        _: &QueryTrail<'_, CardOperation, Walked>,
        rating: CardRating, ) ->
        FieldResult<CardOperation>
    {
        unimplemented!()
    }
}

fn playground() -> HttpResponse {
    let html = playground_source("");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

fn graphql(
    schema: web::Data<Arc<Schema>>,
    data: web::Json<GraphQLRequest>,
    db_pool: web::Data<DbPool>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let ctx = Context {
        db_con: db_pool.get().unwrap(),
    };

    web::block(move || {
        let res = data.execute(&schema, &ctx);
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .map_err(Error::from)
    .and_then(|user| {
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(user))
    })
}

pub fn register(config: &mut web::ServiceConfig) {
    let schema = std::sync::Arc::new(Schema::new(Query {}, Mutation {}));

    config
        .data(schema)
        .route("/", web::post().to_async(graphql))
        .route("/", web::get().to(playground));
}
