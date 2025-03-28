use log::debug;
use sqlx::{Column, ColumnIndex, Decode, MySqlPool, PgPool, Row, Type};
use crate::db_element::db::QueryResult;

pub async fn mysql_query(pool: &MySqlPool, count_query: String, select_query: String, offset: usize, limit: usize) -> Result<QueryResult, String>

{
    let total_rows :u64 = sqlx::query_scalar(&count_query)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    let rows = sqlx::query(&select_query)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    if rows.is_empty() {
        return Ok(QueryResult {
            columns: Vec::new(),
            rows: Vec::new(),
            current_page: 0,
            total_pages: 0,
            limit: 0,
        });
    }

    // Extract column names
    let columns = rows[0]
        .columns()
        .iter()
        .map(|c| {
            c.name().to_string()
        })
        .collect();

    // Extract row data
    let result_rows = rows
        .iter()
        .map(|row| {
            process_row(row)
        })
        .collect();

    debug!("Finish running query: {}", select_query);


    Ok(QueryResult {
        columns,
        rows: result_rows,
        current_page: (offset / limit) + 1,
        total_pages: ((total_rows as f64 / limit as f64).ceil() as u64) as usize,
        limit,
    })
}

pub async fn postgres_query(pool: &PgPool, count_query: String, select_query: String, offset: usize, limit: usize) -> Result<QueryResult, String>
{
    let total_rows: i64 = sqlx::query_scalar(&count_query)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
    let rows = sqlx::query(&select_query)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    if rows.is_empty() {
        return Ok(QueryResult {
            columns: Vec::new(),
            rows: Vec::new(),
            current_page: 0,
            total_pages: 0,
            limit: 0,
        });
    }


    // Extract column names
    let columns = rows[0]
        .columns()
        .iter()
        .map(|c| {
            c.name().to_string()
        })
        .collect();

    // Extract row data
    let result_rows = rows
        .iter()
        .map(|row| {
            process_row(row)
        })
        .collect();

    debug!("Finish running query: {}", select_query);
    Ok(QueryResult {
        columns,
        rows: result_rows,
        current_page: (offset / limit) + 1,
        total_pages: (total_rows as f64 / limit as f64).ceil() as usize,
        limit,
    })
}




fn process_row<R: Row>(row: &R) -> Vec<String>
where
    R: Row,
    for<'r> String: Decode<'r, R::Database> + Type<R::Database>,
    for<'r> Option<String>: Decode<'r, R::Database> + Type<R::Database>,
    for<'r> i64: Decode<'r, R::Database> + Type<R::Database>,  // Handle integers
    for<'r> i32: Decode<'r, R::Database> + Type<R::Database>,  // Handle integers
    for<'r> f64: Decode<'r, R::Database> + Type<R::Database>,  // Handle floats
    for<'r> Option<Vec<u8>>: Decode<'r, R::Database> + Type<R::Database>, usize: ColumnIndex<R> // Handle binary
{
    (0..row.columns().len())
        .map(|i| {

            if let Ok(value) = row.try_get::<String, _>(i) {
                return value;
            }

            if let Ok(value) = row.try_get::<i32, _>(i) {
                return value.to_string();
            }

            if let Ok(value) = row.try_get::<i64, _>(i) {
                return value.to_string();
            }

            if let Ok(value) = row.try_get::<f64, _>(i) {
                return value.to_string();
            }

            if let Ok(Some(value)) = row.try_get::<Option<String>, _>(i) {
                return value;
            }

            if let Ok(Some(_bytes)) = row.try_get::<Option<Vec<u8>>, _>(i) {
                return "<binary>".to_string();
            }

            "<unknown>".to_string()
        })
        .collect()
}