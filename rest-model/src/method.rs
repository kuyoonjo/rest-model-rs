use anyhow::Error;

use crate::{
    db_client::DbClient, pagination::PaginationParams, response::DeleteResult, DeleteParams, Doc,
    PaginationResult, PatchParams, RestModel, UpdateResult, UpsertResult,
};

pub trait Init<T, Db>
where
    T: RestModel,
    Db: DbClient<T>,
{
    fn init(client: &Db) -> impl std::future::Future<Output = Result<(), Error>> {
        async { client.init(T::get_db_name(), T::get_table_name()).await }
    }
}

pub trait GetWithId<T, Db>
where
    T: RestModel,
    Db: DbClient<T>,
{
    fn get_with_id(
        client: &Db,
        id: &str,
    ) -> impl std::future::Future<Output = Result<Doc<T>, Error>> {
        async {
            client
                .select_by_id(T::get_db_name(), T::get_table_name(), id)
                .await
        }
    }
}

pub trait Get<T, Db>
where
    T: RestModel,
    Db: DbClient<T>,
{
    fn get(
        client: &Db,
        pagination_params: &PaginationParams,
    ) -> impl std::future::Future<Output = Result<PaginationResult<T>, Error>> {
        async {
            client
                .paginate(T::get_db_name(), T::get_table_name(), pagination_params)
                .await
        }
    }
}

pub trait Put<T, Db>
where
    T: RestModel,
    Db: DbClient<T>,
{
    fn put(
        client: &Db,
        items: &[Doc<T>],
    ) -> impl std::future::Future<Output = Result<UpsertResult, Error>> {
        async {
            client
                .upsert(T::get_db_name(), T::get_table_name(), items)
                .await
        }
    }
}

pub trait Patch<T, Db>
where
    T: RestModel,
    Db: DbClient<T>,
{
    fn patch(
        client: &Db,
        params: &PatchParams,
    ) -> impl std::future::Future<Output = Result<UpdateResult, Error>> {
        async move {
            client
                .update(T::get_db_name(), T::get_table_name(), &params)
                .await
        }
    }
}

pub trait Delete<T, Db>
where
    T: RestModel,
    Db: DbClient<T>,
{
    fn delete(
        client: &Db,
        params: &DeleteParams,
    ) -> impl std::future::Future<Output = Result<DeleteResult, Error>> {
        async move {
            client
                .delete(T::get_db_name(), T::get_table_name(), &params)
                .await
        }
    }
}
