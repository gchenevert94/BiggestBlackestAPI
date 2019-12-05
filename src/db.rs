use actix::{fut::wrap_future, prelude::*};
use tokio_postgres::{connect, Client, Connection, Statement, NoTls, types::Type};
use std::sync::Arc;

pub struct PgConnection {
  pg_client: Option<Client>,
  con: Option<Arc<Connection<tokio_postgres::Socket, tokio_postgres::tls::NoTlsStream>>>,
}

impl Actor for PgConnection {
  type Context = Context<Self>;
}

impl PgConnection {
  pub async fn connect(db_url: &str) -> Result<Addr<PgConnection>, tokio_postgres::Error> {
    let (client, con) = connect(db_url, NoTls).await?;

    Ok(PgConnection::create(move |ctx| {
      PgConnection {
        pg_client: Some(client),
        con: Some(Arc::new(con))
      }
    }))
  }
}