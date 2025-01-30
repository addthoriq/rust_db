fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use sqlx::{postgres::PgPoolOptions, Connection, Error, PgConnection, Pool, Postgres};

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
