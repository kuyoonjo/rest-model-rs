use anyhow::{bail, Result};
use bb8_postgres::{bb8::Pool, tokio_postgres::NoTls, PostgresConnectionManager};
use oid::ObjectId;
use rest_model::{
    pagination::{Pagination, PaginationParams},
    DbClient, DeleteParams, DeleteResult, Doc, PaginationResult, PatchParams, RestModel,
    UpdateResult, UpsertResult,
};
use serde_json::Value;
use tokio_postgres::types::ToSql;

mod query;
pub use query::*;
use tracing::debug;
mod oid;

pub struct Db {
    pub pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl Db {
    pub async fn try_new(postgres_uri: &str) -> Result<Self> {
        let manager = PostgresConnectionManager::new_from_stringlike(postgres_uri, NoTls)?;
        let pool = Pool::builder().max_size(10).build(manager).await?;
        Ok(Self { pool })
    }
}

impl<T: RestModel> DbClient<T> for Db {
    fn generate_id(&self) -> String {
        ObjectId::new().to_hex()
    }

    async fn init(
        &self,
        db_name: &str,
        table_name: &str,
    ) -> std::result::Result<(), anyhow::Error> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {}.{} (
                _id VARCHAR(24) PRIMARY KEY,
                data JSONB NOT NULL,
                _created_at BIGINT DEFAULT (EXTRACT(EPOCH FROM NOW()) * 1000)::BIGINT,
                _updated_at BIGINT DEFAULT (EXTRACT(EPOCH FROM NOW()) * 1000)::BIGINT
            );",
            db_name, table_name
        );
        self.pool
            .get()
            .await?
            .execute(&sql, &[])
            .await
            .map(|_| ())
            .map_err(|e| anyhow::Error::from(e))
    }

    async fn select_by_id(&self, db_name: &str, table_name: &str, id: &str) -> Result<Doc<T>> {
        let sql = format!(
            "SELECT * FROM {}.{} WHERE _id = '{}'",
            db_name, table_name, id
        );
        let conn = self.pool.get().await?;
        let rows = conn.query(&sql, &[]).await?;
        if rows.is_empty() {
            bail!("Document not found");
        }
        let row = rows.get(0).unwrap();
        let data: Value = row.get("data");
        let data: T = serde_json::from_value(data)?;
        let doc = Doc {
            _id: row.get("_id"),
            data,
            _created_at: row.get("_created_at"),
            _updated_at: row.get("_updated_at"),
        };
        Ok(doc)
    }

    async fn paginate(
        &self,
        db_name: &str,
        table_name: &str,
        pagination_params: &PaginationParams,
    ) -> Result<PaginationResult<T>> {
        let page = pagination_params.page.unwrap_or(1).max(1);
        let limit = pagination_params.limit.unwrap_or(10).max(1);
        let offset = (page - 1) * limit;

        let mut seq = 1u32;
        let mut bindings = vec![];

        let where_sql = if let Some(ref filter) = pagination_params.filter {
            let sql = cond_to_sql(filter, &mut bindings, &mut seq)?;
            if sql.is_empty() {
                "".to_string()
            } else {
                format!("WHERE {}", sql)
            }
        } else {
            "".to_string()
        };

        // 处理排序
        let order_sql = if let Some(sort_expr) = &pagination_params.sort {
            sort_to_sql(sort_expr)?
        } else {
            "_id ASC".to_string()
        };

        // 查询分页数据
        let query_sql = format!(
            "SELECT * FROM {}.{} {} ORDER BY {} LIMIT {} OFFSET {}",
            db_name, table_name, where_sql, order_sql, limit, offset
        );

        // 查询总数
        let total_sql = format!(
            "SELECT COUNT(*) FROM {}.{} {}",
            db_name, table_name, where_sql
        );

        let conn = self.pool.get().await?;
        let args: Vec<&(dyn ToSql + Sync)> =
            bindings.iter().map(|v| v.as_ref()).collect::<Vec<_>>();
        debug!("args: {:?}", args);
        debug!("total_sql: {}", total_sql);
        let row = conn.query_one(&total_sql, &args).await?;
        let total_count: i64 = row.get(0);
        let total_count = total_count as u32;

        let items = if total_count > 0 {
            debug!("query_sql: {}", query_sql);
            let row = conn.query(&query_sql, &args).await?;
            let mut items = Vec::new();
            for row in row {
                let data: Value = row.get(1);
                let data: T = serde_json::from_value(data)?;
                let doc = Doc {
                    _id: row.get(0),
                    data,
                    _created_at: row.get(2),
                    _updated_at: row.get(3),
                };
                items.push(doc);
            }
            items
        } else {
            vec![]
        };

        let total_pages = total_count / limit + if total_count % limit > 0 { 1 } else { 0 };
        Ok(PaginationResult {
            items,
            pagination: Pagination {
                total_count,
                total_pages,
                current_page: page,
                items_per_page: limit,
            },
        })
    }

    async fn upsert(
        &self,
        db_name: &str,
        table_name: &str,
        items: Vec<Doc<T>>,
    ) -> Result<UpsertResult> {
        if items.is_empty() {
            return Ok(UpsertResult {
                created_count: 0,
                updated_count: 0,
            });
        }

        let mut query = format!("INSERT INTO {}.{} (_id, data) VALUES ", db_name, table_name);

        let mut values = Vec::new();
        let mut args: Vec<Box<dyn ToSql + Sync>> = Vec::new();

        for (i, doc) in items.iter().enumerate() {
            values.push(format!("(${}, ${})", i * 2 + 1, i * 2 + 2,));

            args.push(Box::new(doc._id.clone()));
            args.push(Box::new(serde_json::to_value(&doc.data)?));
        }

        query.push_str(&values.join(", "));
        query.push_str(
            " ON CONFLICT (_id) DO UPDATE SET
              data = EXCLUDED.data,
              _updated_at = (EXTRACT(EPOCH FROM NOW()) * 1000)::BIGINT
            RETURNING (xmax = 0) AS inserted;",
        );

        debug!("{}", query);

        let conn = self.pool.get().await?;
        let args_refs: Vec<&(dyn ToSql + Sync)> = args.iter().map(|x| x.as_ref()).collect();
        let rows = conn.query(&query, &args_refs[..]).await?;

        let created_count = rows.iter().filter(|row| row.get::<_, bool>(0)).count() as u32;
        let updated_count = rows.len() as u32 - created_count;

        Ok(UpsertResult {
            created_count,
            updated_count,
        })
    }

    async fn update(
        &self,
        db_name: &str,
        table_name: &str,
        params: &PatchParams,
    ) -> Result<UpdateResult> {
        // 2️⃣ 解析 `patch` 生成 `JSONB SET` 语句
        let mut set_sql = "data = ".to_string();
        let mut args: Vec<Box<dyn ToSql + Sync>> = Vec::new();
        let mut jsonb_expr = "data".to_string(); // 初始值为 `data`

        for (key, value) in params.patch.as_object().unwrap() {
            let path_arg_index = args.len() + 1;
            let value_arg_index = path_arg_index + 1;

            jsonb_expr = format!(
                "jsonb_set({}, ${}, ${}, true)",
                jsonb_expr, path_arg_index, value_arg_index
            );

            args.push(Box::new(vec![key.to_string()])); // JSON 路径，必须是 `TEXT[]`
            args.push(Box::new(value.clone())); // JSON 值
        }

        set_sql.push_str(&jsonb_expr);

        // 更新 `_updated_at`
        let set_sql = format!(
            "{}, _updated_at = EXTRACT(EPOCH FROM NOW()) * 1000",
            set_sql
        );

        // 1️⃣ 解析 `filter` 生成 `WHERE` 语句
        let mut bindings = vec![];
        let seq = &mut (args.len() as u32 + 1);
        let where_sql = cond_to_sql(&params.filter, &mut bindings, seq)?;

        // 3️⃣ 生成 SQL
        let query = format!(
            "UPDATE {}.{} SET {} {} RETURNING _id;",
            db_name, table_name, set_sql, where_sql
        );
        args.append(&mut bindings);

        // 4️⃣ 执行 SQL
        debug!("{}", query);
        debug!("{:?}", args);
        let args_refs: Vec<&(dyn ToSql + Sync)> = args.iter().map(|x| x.as_ref()).collect();
        let conn = self.pool.get().await?;
        let rows = conn.query(&query, &args_refs[..]).await?;

        // 5️⃣ 返回更新的行数
        Ok(UpdateResult {
            updated_count: rows.len() as u32,
        })
    }

    async fn delete(
        &self,
        db_name: &str,
        table_name: &str,
        params: &DeleteParams,
    ) -> Result<DeleteResult> {
        // 1️⃣ 解析 `filter` 生成 `WHERE` 语句
        let bindings = &mut vec![];
        let seq = &mut 1;
        let where_sql = cond_to_sql(&params.filter, bindings, seq)?;

        // 2️⃣ 生成 SQL
        let query = format!(
            "DELETE FROM {}.{} {} RETURNING _id;",
            db_name, table_name, where_sql
        );

        // 3️⃣ 执行 SQL
        let conn = self.pool.get().await?;
        let args_refs: Vec<&(dyn ToSql + Sync)> = bindings.iter().map(|x| x.as_ref()).collect();
        let rows = conn.query(&query, &args_refs).await?;

        // 4️⃣ 返回删除的行数
        Ok(DeleteResult {
            deleted_count: rows.len() as u32,
        })
    }
}
