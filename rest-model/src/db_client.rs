use anyhow::Error;

use crate::{DeleteParams, Doc, PaginationResult, PatchParams, UpdateResult, UpsertResult};

use super::{pagination::PaginationParams, response::DeleteResult, RestModel};

pub trait DbClient<T>: Sync
where
    T: RestModel,
{
    fn generate_id(&self) -> String;

    fn init(
        &self,
        db_name: &str,
        table_name: &str,
    ) -> impl std::future::Future<Output = Result<(), Error>>;

    /// GET /resources/:id
    fn select_by_id(
        &self,
        db_name: &str,
        table_name: &str,
        id: &str,
    ) -> impl std::future::Future<Output = Result<Doc<T>, Error>>;

    /// GET /resources
    fn paginate(
        &self,
        db_name: &str,
        table_name: &str,
        pagination_params: &PaginationParams,
    ) -> impl std::future::Future<Output = Result<PaginationResult<T>, Error>>;

    /// PUT /resources
    fn upsert(
        &self,
        db_name: &str,
        table_name: &str,
        items: &[Doc<T>],
    ) -> impl std::future::Future<Output = Result<UpsertResult, Error>>;

    /// PATCH /resources
    fn update(
        &self,
        db_name: &str,
        table_name: &str,
        params: &PatchParams,
    ) -> impl std::future::Future<Output = Result<UpdateResult, Error>>;

    /// DELETE /resources
    fn delete(
        &self,
        db_name: &str,
        table_name: &str,
        filter: &DeleteParams,
    ) -> impl std::future::Future<Output = Result<DeleteResult, Error>>;
}
