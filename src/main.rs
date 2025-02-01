fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use chrono::{Local, NaiveDateTime};
    use sqlx::{Connection, Error, FromRow, PgConnection, Pool, Postgres, Row, Transaction};
    use sqlx::postgres::{PgPoolOptions, PgRow};
    use futures::TryStreamExt;

    async fn get_pool() -> Result<Pool<Postgres>, Error> {
        let url = "postgres://postgres:123@localhost:5432/rust_db";
        PgPoolOptions::new()
            .max_connections(10) // Maksimal Konek Pool
            .min_connections(5) // Minimal konek ke Pool
            .acquire_timeout(Duration::from_secs(5)) // ketika mau ngambil koneksi, berapa lama nunggunya
            .idle_timeout(Duration::from_secs(60)) // Koneksi pool tidak melakukan transaksi tapi
            // masih terkonek ke db
            .connect(url).await
    }

    #[tokio::test]
    async fn test_auto_increment_transaction() -> Result<(), Error> {
        let pool: Pool<Postgres> = get_pool().await?;
        let mut trx: Transaction<Postgres> = pool.begin().await?;
        
        let stmt: String = String::from("INSERT INTO sellers(name) VALUES ($1)");
        sqlx::query(&stmt).bind("Contoh").execute(&mut *trx).await?;

        let stmt: String = String::from("SELECT lastval() as id");

        let result: PgRow = sqlx::query(&stmt).fetch_one(&mut *trx).await?;

        let id: i32 = result.get_unchecked("id");
        println!("ID: {}", id);

        trx.commit().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_auto_increment_returning() -> Result<(), Error> {
        let pool: Pool<Postgres> = get_pool().await?;
        let stmt: String = String::from("INSERT INTO sellers(name) VALUES ($1) RETURNING id");
        let result: PgRow = sqlx::query(&stmt).bind("Contoh").fetch_one(&pool).await?;

        let id: i32 = result.get("id");
        println!("ID Seller: {}", id);

        Ok(())
    }

    #[tokio::test]
    async fn test_transaction() -> Result<(), Error> {
        let pool: Pool<Postgres> = get_pool().await?;
        let mut transaction: Transaction<Postgres> = pool.begin().await?;
        sqlx::query("INSERT INTO categories VALUES ($1,$2,$3);")
            .bind("Z")
            .bind("Test Category TRX")
            .bind("Test Desc Trx")
            .execute(&mut *transaction).await?;
        sqlx::query("INSERT INTO brands VALUES ($1,$2,$3,$4,$4);")
            .bind("M")
            .bind("Test Brand Trx")
            .bind("Test Desc Brand Trx")
            .bind(Local::now().naive_local())
            .execute(&mut *transaction).await?;
        transaction.commit().await?;
        Ok(())
    }

    #[allow(dead_code)]
    #[derive(Debug,FromRow)]
    struct Brand {
        id: String,
        name: String,
        description: String,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime
    }

    #[tokio::test]
    async fn test_mapping_brands() -> Result<(), Error> {
        let pool: Pool<Postgres> = get_pool().await?;
        let stmt: String = String::from("SELECT * FROM brands;");
        let result: Vec<Brand> = sqlx::query_as(&stmt).fetch_all(&pool).await?;

        for brand in result {
            println!("{:?}", brand)
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_insert_createdat() -> Result<(), Error> {
        let pool = get_pool().await?;
        let stmt = String::from("INSERT INTO brands VALUES ($1,$2,$3,$4,$4);");
        sqlx::query(&stmt)
            .bind("A").bind("Name").bind("Description").bind(Local::now().naive_local())
            .execute(&pool).await?;

        Ok(())
    }

    #[tokio::test]
    // Menampilkan melalui Struct
    async fn test_mapping_struct_from_row() -> Result<(), Error> {
        let pool = get_pool().await?;
        let stmt = String::from("SELECT * FROM categories");
        let result: Vec<Categories> = sqlx::query_as(&stmt)
            .fetch_all(&pool).await?;

        for category in result {
            println!("{:?}", category);
        }

        Ok(())
    }

    #[allow(dead_code)]
    #[derive(Debug, FromRow)]
    struct Categories {
        id: String,
        name: String,
        description: String
    }

    #[tokio::test]
    // Menampilkan melalui Struct
    async fn test_mapping_struct() -> Result<(), Error> {
        let pool = get_pool().await?;
        let stmt = String::from("SELECT * FROM categories");
        let result: Vec<Category> = sqlx::query(&stmt)
            .map(|row: PgRow|{
                Category {
                    id: row.get("id"),
                    name: row.get("name"),
                    description: row.get("description")
                }
            })
            .fetch_all(&pool).await?;

        for category in result {
            println!("{:?}", category);
        }

        Ok(())
    }

    #[allow(dead_code)]
    #[derive(Debug)]
    struct Category {
        id: String,
        name: String,
        description: String
    }


    #[tokio::test]
    // Untuk melakukan lazy load dengan menampilkan jutaan data
    async fn test_fetch_stream() -> Result<(), Error> {
        let pool = get_pool().await?;
        let stmt = String::from("SELECT * FROM categories");
        let mut result = sqlx::query(&stmt)
            .fetch(&pool);

        while let Some(row) = result.try_next().await? {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let desc: String = row.get("description");
            println!("ID: {}, Name: {}, Desc: {}", id, name, desc);
        }

        Ok(())
    }

    #[tokio::test]
    // Menampilkan semua data baik kosong atau ada
    async fn test_fetch_all() -> Result<(), Error> {
        let pool = get_pool().await?;
        let stmt = String::from("SELECT * FROM categories");
        let result: Vec<PgRow> = sqlx::query(&stmt)
            .fetch_all(&pool).await?;

        for val in result {
            let id: String = val.get("id");
            let name: String = val.get("name");
            let desc: String = val.get("description");

            println!("ID: {}, Name: {}, Desc: {}", id, name, desc);
        }

        Ok(())
    }

    #[tokio::test]
    // Menampilkan data wajib ada, tidak boleh kosong
    async fn test_fetch_one() -> Result<(), Error> {
        let pool = get_pool().await?;
        let stmt = String::from("SELECT * FROM categories WHERE id = $1");
        let result: PgRow = sqlx::query(&stmt)
            .bind("C")
            .fetch_one(&pool).await?;

        let id: String = result.get("id");
        let name: String = result.get("name");
        let desc: String = result.get("description");

        println!("ID: {}, Name: {}, Desc: {}", id, name, desc);

        Ok(())
    }

    #[tokio::test]
    // Menampilkan data ada maupun kosong
    async fn test_fetch_optional() -> Result<(), Error> {
        let pool = get_pool().await?;
        let stmt = String::from("SELECT * FROM categories WHERE id = $1");
        let result: Option<PgRow> = sqlx::query(&stmt)
            .bind("E")
            .fetch_optional(&pool).await?;

        if let Some(row) = result {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let desc: String = row.get("description");

            println!("ID: {}, Name: {}, Desc: {}", id, name, desc)
        } else {
            println!("Data Not Found")
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_prepare_stmt_query_as_stmt_using_bind() -> Result<(), Error> {
        let pool = get_pool().await?;
        let stmt = String::from("INSERT INTO categories VALUES ($1,$2,$3)");
        sqlx::query(&stmt)
            .bind("F").bind("Name").bind("Desc")
            .execute(&pool).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_prepare_stmt_same_bind() -> Result<(), Error> {
        let pool = get_pool().await?;
        sqlx::query("INSERT INTO categories VALUES ($1,$2,$2)")
            .bind("C").bind("Name and Desc")
            .execute(&pool).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_prepare_stmt() -> Result<(), Error> {
        let pool = get_pool().await?;
        sqlx::query("INSERT INTO categories VALUES ($1,$2,$3)")
            .bind("B").bind("Name").bind("Desc")
            .execute(&pool).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_sql() -> Result<(), Error> {
        let pool = get_pool().await?;
        sqlx::query("INSERT INTO categories VALUES ('1','Nama','Deskripsii')")
            .execute(&pool).await?;

        Ok(())
    }


    #[tokio::test]
    async fn test_execute_sql_query_as_stmt() -> Result<(), Error> {
        let pool = get_pool().await?;
        let stmt = String::from("INSERT INTO categories VALUES ('2','Nama','Deskripsii')");
        sqlx::query(&stmt)
            .execute(&pool).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_connection_pool() -> Result<(), Error> {
        let pool = get_pool().await?;

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_manual_connection() -> Result<(), Error> {
        let url = "postgres://postgres:123@localhost:5432/rust_db";
        let connection: PgConnection = PgConnection::connect(url).await?;

        connection.close().await?;
        Ok(())
    }
}
