use anyhow::Result;
use tokio_postgres::{Client, NoTls, Row};
use chrono::{NaiveDateTime, DateTime, Utc};

pub struct Database {
    client: Client,
}

#[derive(Debug, Clone)]
pub struct SchemaInfo {
    pub name: String,
    pub tables: Vec<String>,
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

    pub async fn list_databases(&self) -> Result<Vec<String>> {
        let rows = self
            .client
            .query(
                "SELECT datname FROM pg_database
                 WHERE datistemplate = false
                 ORDER BY datname",
                &[],
            )
            .await?;

        Ok(rows.iter().map(|row| row.get(0)).collect())
    }

    pub async fn list_schemas(&self) -> Result<Vec<String>> {
        let rows = self
            .client
            .query(
                "SELECT schema_name FROM information_schema.schemata
                 WHERE schema_name NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
                 ORDER BY schema_name",
                &[],
            )
            .await?;

        Ok(rows.iter().map(|row| row.get(0)).collect())
    }

    pub async fn list_tables(&self) -> Result<Vec<String>> {
        let rows = self
            .client
            .query(
                "SELECT table_name FROM information_schema.tables
                 WHERE table_schema = 'public'
                 ORDER BY table_name",
                &[],
            )
            .await?;

        Ok(rows.iter().map(|row| row.get(0)).collect())
    }

    pub async fn list_tables_in_schema(&self, schema: &str) -> Result<Vec<String>> {
        let rows = self
            .client
            .query(
                "SELECT table_name FROM information_schema.tables
                 WHERE table_schema = $1
                 AND table_type IN ('BASE TABLE', 'VIEW', 'MATERIALIZED VIEW')
                 ORDER BY table_name",
                &[&schema],
            )
            .await?;

        Ok(rows.iter().map(|row| row.get(0)).collect())
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
            schemas_map.entry(schema).or_insert_with(Vec::new).push(table);
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

    pub async fn query_table(&self, table_name: &str, limit: i64) -> Result<(Vec<String>, Vec<Vec<String>>)> {
        // Parse schema and table name
        let (schema, table) = if table_name.contains('.') {
            let parts: Vec<&str> = table_name.split('.').collect();
            (parts[0], parts[1])
        } else {
            ("public", table_name)
        };

        // Get column names
        let columns_query = format!(
            "SELECT column_name FROM information_schema.columns
             WHERE table_schema = '{}' AND table_name = '{}'
             ORDER BY ordinal_position",
            schema, table
        );
        let column_rows = self.client.query(&columns_query, &[]).await?;
        let columns: Vec<String> = column_rows.iter().map(|row| row.get(0)).collect();

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

    pub async fn execute_query(&self, query: &str) -> Result<(Vec<String>, Vec<Vec<String>>)> {
        let rows = self.client.query(query, &[]).await?;

        if rows.is_empty() {
            return Ok((vec![], vec![]));
        }

        let columns: Vec<String> = rows[0]
            .columns()
            .iter()
            .map(|col| col.name().to_string())
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
