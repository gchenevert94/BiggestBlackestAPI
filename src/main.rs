mod db;
mod gql;
mod models;

use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use std::env;
use std::io;
use std::path::Path;

use db::{PgConfig, Pool, PoolConfiguration};

pub struct Context {
  db: web::Data<Pool>,
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
  std::env::set_var("RUST_LOG", "actix_web=info");

  dotenv().ok();

  let pool_config = PoolConfiguration::default();
  let pg_config = pg_config_from_env().expect("Must provide connection to database");

  let pool = db::create_pool(pg_config, &pool_config);

  env_logger::init();

  // Start http server
  HttpServer::new(move || {
    App::new()
      .data(pool.clone())
      .configure(gql::register)
      .wrap(middleware::Logger::default())
      .wrap(middleware::Compress::default())
      .default_service(web::route().to(|| HttpResponse::NotFound()))
  })
  .bind("127.0.0.1:8080")?
  .start()
  .await?;

  Ok(())
}

pub fn pg_config_from_env() -> Result<PgConfig, String> {
  let mut config = PgConfig::new();
  if let Ok(host) = env::var("PG_HOST") {
    config.host(host.as_str());
  } else {
    if Path::new("/run/postgresql").exists() {
      config.host("/run/postgresql");
    } else {
      config.host("/tmp");
    }
  }
  if let Ok(port_string) = env::var("PG_PORT") {
    let port = port_string
      .parse::<u16>()
      .map_err(|_| format!("Invalid PG_PORT: {}", port_string))?;
    config.port(port);
  }
  if let Ok(user) = env::var("PG_USER") {
    config.user(user.as_str());
  } else if let Ok(user) = env::var("USER") {
    config.user(user.as_str());
  } else {
    return Err("Missing PG_USER. Fallback to USER failed as well.".into());
  }
  if let Ok(password) = env::var("PG_PASSWORD") {
    config.password(password.as_str());
  }
  if let Ok(dbname) = env::var("PG_DBNAME") {
    config.dbname(dbname.as_str());
  }
  Ok(config)
}
