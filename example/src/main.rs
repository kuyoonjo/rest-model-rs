use rest_model::{
    Doc,
    method::{Init, Put},
    rest_model,
};
use rest_model_postgres::Db;
use serde::{Deserialize, Serialize};

const DB_NAME: &str = "mydb.public";
const TABLE_NAME: &str = "guest";

#[rest_model(db(Db, DB_NAME, TABLE_NAME), with(all))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Guest {
    pub name: String,
}

// tokio main
#[tokio::main]
async fn main() {
    let uri = &std::env::var("DATABASE_URL").unwrap();
    let client = Db::try_new(uri).await.unwrap();
    Guest::init(&client).await.unwrap();
    Guest::put(
        &client,
        vec![
            Doc::new(
                &client,
                Guest {
                    name: "John".to_string(),
                },
            ),
            Doc::new(
                &client,
                Guest {
                    name: "Jane".to_string(),
                },
            ),
            Doc::new(
                &client,
                Guest {
                    name: "Jim".to_string(),
                },
            ),
            Doc::new(
                &client,
                Guest {
                    name: "Joe".to_string(),
                },
            ),
        ],
    )
    .await
    .unwrap();
}
