fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use sqlx::{Connection, Error, PgConnection, Pool, Postgres, Row};
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
