#[warn(unused_imports)]

#[macro_use]
extern crate diesel;
extern crate diesel_full_text_search;
extern crate base64;
extern crate serde_derive;
extern crate juniper;
extern crate juniper_from_schema;

use std::io;

use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use diesel::{
    r2d2::{Pool, ConnectionManager, PooledConnection},
    PgConnection
};

mod schema;
mod models;
mod gql;
mod db;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbCon = PooledConnection<ConnectionManager<PgConnection>>;

pub struct Context {
    db_con: DbCon
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");

    dotenv::dotenv().ok();

    env_logger::init();

    let db_pool = create_db_pool();

    // Start http server
    HttpServer::new(move || {
        App::new()
            .data(db_pool.clone())
            .configure(gql::register)
            .wrap(middleware::Logger::default())
            .default_service(web::route().to(|| HttpResponse::NotFound()))
    })
    .bind("127.0.0.1:8080")?
    .start()
    .await
}

fn create_db_pool() -> DbPool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Pool::builder()
        .max_size(3)
        .build(ConnectionManager::<PgConnection>::new(database_url))
        .expect("failed to create db connection pool")
}
