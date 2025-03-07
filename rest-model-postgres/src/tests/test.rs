use rest_model::{
    method::{Delete, Get, GetWithId, Init, Patch, Put},
    pagination::PaginationParams,
    Condition, DeleteParams, Doc, PatchParams, RestModel,
};
use rest_model_postgres::Db;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct User {
    pub name: String,
    pub age: i32,
    pub info: Value,
}

impl Init<User, Db> for User {}
impl GetWithId<User, Db> for User {}
impl Get<User, Db> for User {}
impl Put<User, Db> for User {}
impl Patch<User, Db> for User {}
impl Delete<User, Db> for User {}

impl RestModel for User {
    fn get_db_name() -> &'static str {
        "mydb.public"
    }

    fn get_table_name() -> &'static str {
        "users"
    }
}

#[test]
fn init() {
    let uri = &std::env::var("DATABASE_URL").unwrap();
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let client = Db::try_new(uri).await.unwrap();
        User::init(&client).await.unwrap();
        let tom = Doc {
            _id: "67c707bc698b8e529f994670".to_string(),
            data: User {
                name: "Tom".to_string(),
                age: 10,
                info: json!({
                    "a": 1,
                }),
            },
            _created_at: 0,
            _updated_at: 0,
        };
        let jerry = Doc {
            _id: "67c707bc698b8e529f994671".to_string(),
            data: User {
                name: "Jerry".to_string(),
                age: 9,
                info: json!({
                    "a": 1,
                }),
            },
            _created_at: 0,
            _updated_at: 0,
        };
        let spike = Doc {
            _id: "67c707bc698b8e529f994672".to_string(),
            data: User {
                name: "Spike".to_string(),
                age: 8,
                info: json!({
                    "a": 1,
                }),
            },
            _created_at: 0,
            _updated_at: 0,
        };
        User::put(&client, &vec![tom, jerry, spike]).await.unwrap();
        // User::patch(&client, &PatchParams {
        //     filter: FilterParams::Where("()")
        //     patch: json!({
        //         "age": 9,
        //     }),
        // })
        // User::User::get_with_id(client, id).await;
    });
}

#[test]
fn update() {
    let uri = &std::env::var("DATABASE_URL").unwrap();
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let client = Db::try_new(uri).await.unwrap();
        User::patch(
            &client,
            &PatchParams {
                filter: Condition::Gt("age".to_string(), json!(8)),
                patch: json!({
                    "age": 9,
                    "info": {
                      "a": 2,
                    },
                }),
            },
        )
        .await
        .unwrap();
        User::patch(
            &client,
            &PatchParams {
                filter: Condition::Eq("name".to_string(), json!("Spike")),
                patch: json!({
                    "age": 7,
                    "info": {
                      "a": 3,
                    },
                }),
            },
        )
        .await
        .unwrap();
        // User::User::get_with_id(client, id).await;
    });
}

#[test]
fn delete() {
    let uri = &std::env::var("DATABASE_URL").unwrap();
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let client = Db::try_new(uri).await.unwrap();
        User::delete(
            &client,
            &DeleteParams {
                filter: Condition::Gt("age".to_string(), json!(5)),
            },
        )
        .await
        .unwrap();
        // User::User::get_with_id(client, id).await;
    });
}
#[test]
fn get_with_id() {
    let uri = &std::env::var("DATABASE_URL").unwrap();
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let client = Db::try_new(uri).await.unwrap();
        let doc = User::get_with_id(&client, "67c707bc698b8e529f994670")
            .await
            .unwrap();
        println!("{:#?}", doc);
    });
}

#[test]
fn get() {
    let uri = &std::env::var("DATABASE_URL").unwrap();
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let client = Db::try_new(uri).await.unwrap();
        let json = json!({
            "filter": {
               "Or": [
                {
                  "In": ["_id", ["67c707bc698b8e529f994670", "67c707bc698b8e529f994671"]],
                },
               ],
            },
            "limit": 2,
            "page": 1,
            "sort": "+age",
        })
        .into();
        let pagination = serde_json::from_value::<PaginationParams>(json).unwrap();
        println!("{:#?}", pagination);
        let doc = User::get(&client, &pagination).await.unwrap();
        println!("{:#?}", doc);
    });
}
