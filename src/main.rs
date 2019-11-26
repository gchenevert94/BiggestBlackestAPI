#[warn(unused_imports)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate juniper;
#[macro_use]
extern crate juniper_from_schema;

use std::io;
use std::sync::Arc;

use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use diesel::{
    r2d2::{Pool, ConnectionManager, PooledConnection},
    PgConnection
};
use futures::Future;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;

mod schema;
mod models;
mod gql_schema;

use crate::gql_schema::{create_schema, Schema};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbCon = PooledConnection<ConnectionManager<PgConnection>>;

pub struct Context {
    db_con: DbCon
}

fn graphiql() -> HttpResponse {
    let html = graphiql_source("http://127.0.0.1:8080/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

fn graphql(
    st: web::Data<Arc<Schema>>,
    data: web::Json<GraphQLRequest>,
    db_pool: web::Data<DbPool>
) -> impl Future<Item = HttpResponse, Error = Error> {

    let ctx = Context {
        db_con: db_pool.get().unwrap()
    };

    web::block(move || {
        let res = data.execute(&st, &ctx);
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .map_err(Error::from)
    .and_then(|user| {
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(user))
    })
}

fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");

    dotenv::dotenv().ok();

    env_logger::init();

    let db_pool = create_db_pool();

    // Create Juniper schema
    let schema = std::sync::Arc::new(create_schema());

    // Start http server
    HttpServer::new(move || {
        App::new()
            .data(schema.clone())
            .data(db_pool.clone())
            .wrap(middleware::Logger::default())
            .service(web::resource("/graphql").route(web::post().to_async(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
    })
    .bind("127.0.0.1:8080")?
    .run()
}

fn create_db_pool() -> DbPool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Pool::builder()
        .max_size(3)
        .build(ConnectionManager::<PgConnection>::new(database_url))
        .expect("failed to create db connection pool")
}
