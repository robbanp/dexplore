use poll_promise::Promise;
use anyhow::Result;
use std::sync::Arc;
use crate::db::{Database, ColumnInfo, SchemaInfo};

// Type aliases to simplify complex Promise types
type TableDataPromise = Promise<Result<(Vec<ColumnInfo>, Vec<Vec<String>>)>>;
type StructurePromise = Promise<Result<(Arc<Database>, Vec<SchemaInfo>)>>;

pub enum AsyncOperation {
    LoadStructure(StructurePromise),
    LoadTableData(String, String, TableDataPromise, Option<usize>), // schema, table, promise, optional tab_index for reload
    ExecuteQuery(String, TableDataPromise, Option<usize>), // query, promise, optional tab_index for reload
}
