use tokio_postgres::{types::Type, Client, Statement};
use crate::models::{GetCards};

struct DbContext {
    client: Option<Client>,
    get_cards: Option<Statement>,
    get_sets: Option<Statement>
}

impl DbContext {
    pub async fn new(client: Client) -> Result<DbContext, tokio_postgres::Error> {
        let get_cards = client
            .prepare_typed(
                "SELECT id, format_text, is_black, parent_set_id, total_votes, average_rating FROM bb.get_cards($1, $2, $3, $4, $5, $6, $7)",
                &[
                    Type::TEXT,
                    Type::BOOL,
                    Type::INT4,
                    Type::INT4,
                    Type::INT4_ARRAY,
                    Type::BOOL,
                    Type::FLOAT4,
                ],
            )
            .await?;

        let get_sets = client
            .prepare_typed(
                "SELECT * FROM bb.get_sets($1, $2, $3)",
                &[Type::TEXT, Type::INT4, Type::INT4],
            )
            .await?;

        Ok(DbContext {
            client: Some(client),
            get_cards: Some(get_cards),
            get_sets: Some(get_sets)
        })
    }

    pub async fn get_cards(self) -> Vec<GetCards> {
        unimplemented!()
    }
}
