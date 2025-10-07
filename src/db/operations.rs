use poll_promise::Promise;
use anyhow::Result;
use std::sync::Arc;
use crate::db::{Database, ColumnInfo, SchemaInfo};

pub enum AsyncOperation {
    LoadStructure(Promise<Result<(Arc<Database>, Vec<SchemaInfo>)>>),
    LoadTableData(String, String, Promise<Result<(Vec<ColumnInfo>, Vec<Vec<String>>)>>, Option<usize>), // schema, table, promise, optional tab_index for reload
    ExecuteQuery(String, Promise<Result<(Vec<ColumnInfo>, Vec<Vec<String>>)>>, Option<usize>), // query, promise, optional tab_index for reload
}
