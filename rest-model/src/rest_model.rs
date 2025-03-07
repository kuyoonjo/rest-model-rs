use serde::{Deserialize, Serialize};

pub trait RestModelBound: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> {}
impl<T> RestModelBound for T where T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> {}

pub trait RestModel: RestModelBound {
    fn get_db_name() -> &'static str;
    fn get_table_name() -> &'static str;
}
