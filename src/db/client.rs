use anyhow::Result;
use tokio_postgres::{Client, NoTls, Row};
use chrono::{NaiveDateTime, DateTime, Utc};
use crate::db::{ColumnInfo, SchemaInfo};

pub struct Database {
    client: Client,
}

// Helper function to convert PostgreSQL values to strings
fn row_value_to_string(row: &Row, idx: usize) -> String {
    // Try various types in order

    // String/text types
    if let Ok(val) = row.try_get::<_, String>(idx) {
        return val;
    }

    // Integer types
    if let Ok(val) = row.try_get::<_, i32>(idx) {
        return val.to_string();
    }
    if let Ok(val) = row.try_get::<_, i64>(idx) {
        return val.to_string();
    }
    if let Ok(val) = row.try_get::<_, i16>(idx) {
        return val.to_string();
    }

    // Floating point types
    if let Ok(val) = row.try_get::<_, f32>(idx) {
        return val.to_string();
    }
    if let Ok(val) = row.try_get::<_, f64>(idx) {
        return val.to_string();
    }

    // Boolean
    if let Ok(val) = row.try_get::<_, bool>(idx) {
        return val.to_string();
    }

    // UUID
    if let Ok(val) = row.try_get::<_, uuid::Uuid>(idx) {
        return val.to_string();
    }

    // Timestamp types
    if let Ok(val) = row.try_get::<_, NaiveDateTime>(idx) {
        return val.to_string();
    }
    if let Ok(val) = row.try_get::<_, DateTime<Utc>>(idx) {
        return val.to_string();
    }

    // JSON types
    if let Ok(val) = row.try_get::<_, serde_json::Value>(idx) {
        return val.to_string();
    }

    // Byte arrays
    if let Ok(val) = row.try_get::<_, Vec<u8>>(idx) {
        return format!("<{} bytes>", val.len());
    }

    // If all else fails, check if it's NULL
    "(NULL)".to_string()
}

impl Database {
    pub async fn connect(connection_string: &str) -> Result<Self> {
        let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;

        // Keep connection alive in background task
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Database connection error: {}", e);
            }
        });

        Ok(Database { client })
    }

    pub async fn list_all_tables_grouped(&self) -> Result<Vec<SchemaInfo>> {
        // Get all tables grouped by schema in a single query
        let rows = self
            .client
            .query(
                "SELECT table_schema, table_name
                 FROM information_schema.tables
                 WHERE table_schema NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
                 AND table_type IN ('BASE TABLE', 'VIEW', 'MATERIALIZED VIEW')
                 ORDER BY table_schema, table_name",
                &[],
            )
            .await?;

        let mut schemas_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

        for row in rows {
            let schema: String = row.get(0);
            let table: String = row.get(1);
            schemas_map.entry(schema).or_default().push(table);
        }

        let mut result: Vec<SchemaInfo> = schemas_map
            .into_iter()
            .map(|(name, tables)| SchemaInfo { name, tables })
            .collect();

        result.sort_by(|a, b| a.name.cmp(&b.name));

        // If no schemas found, ensure public schema exists
        if result.is_empty() {
            result.push(SchemaInfo {
                name: "public".to_string(),
                tables: vec![],
            });
        }

        Ok(result)
    }

    pub async fn list_schemas_with_tables(&self) -> Result<Vec<SchemaInfo>> {
        // Use the more efficient grouped query
        self.list_all_tables_grouped().await
    }

    pub async fn query_table(&self, table_name: &str, limit: i64) -> Result<(Vec<ColumnInfo>, Vec<Vec<String>>)> {
        // Parse schema and table name
        let (schema, table) = if table_name.contains('.') {
            let parts: Vec<&str> = table_name.split('.').collect();
            (parts[0], parts[1])
        } else {
            ("public", table_name)
        };

        // Get column metadata including data types
        let columns_query = format!(
            "SELECT
                c.column_name,
                c.data_type,
                c.udt_name,
                CASE
                    WHEN c.character_maximum_length IS NOT NULL THEN c.data_type || '(' || c.character_maximum_length || ')'
                    WHEN c.numeric_precision IS NOT NULL AND c.numeric_scale IS NOT NULL THEN c.data_type || '(' || c.numeric_precision || ',' || c.numeric_scale || ')'
                    WHEN c.datetime_precision IS NOT NULL AND c.datetime_precision != 6 THEN c.udt_name || '(' || c.datetime_precision || ')'
                    WHEN c.datetime_precision IS NOT NULL AND c.datetime_precision = 6 THEN c.udt_name || '(6)'
                    ELSE c.udt_name
                END as full_data_type
             FROM information_schema.columns c
             WHERE c.table_schema = '{}' AND c.table_name = '{}'
             ORDER BY c.ordinal_position",
            schema, table
        );
        let column_rows = self.client.query(&columns_query, &[]).await?;

        // Get primary key columns
        let pk_query = format!(
            "SELECT kcu.column_name
             FROM information_schema.table_constraints tc
             JOIN information_schema.key_column_usage kcu
                 ON tc.constraint_name = kcu.constraint_name
                 AND tc.table_schema = kcu.table_schema
             WHERE tc.constraint_type = 'PRIMARY KEY'
                 AND tc.table_schema = '{}'
                 AND tc.table_name = '{}'",
            schema, table
        );
        let pk_rows = self.client.query(&pk_query, &[]).await?;
        let pk_columns: std::collections::HashSet<String> = pk_rows
            .iter()
            .map(|row| row.get::<_, String>(0))
            .collect();

        // Get foreign key columns
        let fk_query = format!(
            "SELECT kcu.column_name
             FROM information_schema.table_constraints tc
             JOIN information_schema.key_column_usage kcu
                 ON tc.constraint_name = kcu.constraint_name
                 AND tc.table_schema = kcu.table_schema
             WHERE tc.constraint_type = 'FOREIGN KEY'
                 AND tc.table_schema = '{}'
                 AND tc.table_name = '{}'",
            schema, table
        );
        let fk_rows = self.client.query(&fk_query, &[]).await?;
        let fk_columns: std::collections::HashSet<String> = fk_rows
            .iter()
            .map(|row| row.get::<_, String>(0))
            .collect();

        // Build column info
        let columns: Vec<ColumnInfo> = column_rows
            .iter()
            .map(|row| {
                let name: String = row.get(0);
                let full_data_type: String = row.get(3);
                ColumnInfo {
                    is_primary_key: pk_columns.contains(&name),
                    is_foreign_key: fk_columns.contains(&name),
                    name,
                    data_type: full_data_type,
                }
            })
            .collect();

        // Get data - use proper schema qualification
        let data_query = format!("SELECT * FROM {}.{} LIMIT {}", schema, table, limit);
        let rows = self.client.query(&data_query, &[]).await?;

        let data: Vec<Vec<String>> = rows
            .iter()
            .map(|row| {
                (0..row.len())
                    .map(|i| row_value_to_string(row, i))
                    .collect()
            })
            .collect();

        Ok((columns, data))
    }

    pub async fn execute_query(&self, query: &str) -> Result<(Vec<ColumnInfo>, Vec<Vec<String>>)> {
        let rows = self.client.query(query, &[]).await?;

        if rows.is_empty() {
            return Ok((vec![], vec![]));
        }

        // For generic queries, we only have basic column info
        let columns: Vec<ColumnInfo> = rows[0]
            .columns()
            .iter()
            .map(|col| ColumnInfo {
                name: col.name().to_string(),
                data_type: format!("{:?}", col.type_()),
                is_primary_key: false,
                is_foreign_key: false,
            })
            .collect();

        let data: Vec<Vec<String>> = rows
            .iter()
            .map(|row| {
                (0..row.len())
                    .map(|i| row_value_to_string(row, i))
                    .collect()
            })
            .collect();

        Ok((columns, data))
    }
}
