use crate::db::Db;
use crate::error::DbError;
use crate::models::author::Author;

pub async fn create(db: &Db, name: &str, description: Option<&str>) -> Result<Author, DbError> {
    let row = sqlx::query_as::<_, Author>(
        "INSERT INTO Authors (name, description) VALUES (?, ?) RETURNING *",
    )
    .bind(name)
    .bind(description)
    .fetch_one(&db.pool)
    .await?;
    Ok(row)
}

pub async fn get(db: &Db, id: i64) -> Result<Option<Author>, DbError> {
    let row = sqlx::query_as::<_, Author>("SELECT * FROM Authors WHERE id = ?")
        .bind(id)
        .fetch_optional(&db.pool)
        .await?;
    Ok(row)
}

pub async fn list(db: &Db) -> Result<Vec<Author>, DbError> {
    let rows = sqlx::query_as::<_, Author>("SELECT * FROM Authors ORDER BY id")
        .fetch_all(&db.pool)
        .await?;
    Ok(rows)
}

pub async fn update(
    db: &Db,
    id: i64,
    name: &str,
    description: Option<&str>,
) -> Result<Author, DbError> {
    let row = sqlx::query_as::<_, Author>(
        "UPDATE Authors SET name = ?, description = ? WHERE id = ? RETURNING *",
    )
    .bind(name)
    .bind(description)
    .bind(id)
    .fetch_optional(&db.pool)
    .await?
    .ok_or(DbError::NotFound)?;
    Ok(row)
}

pub async fn delete(db: &Db, id: i64) -> Result<(), DbError> {
    sqlx::query("DELETE FROM Authors WHERE id = ?")
        .bind(id)
        .execute(&db.pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    async fn setup() -> Db {
        let db = Db::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        db
    }

    #[tokio::test]
    async fn create_returns_author_with_id() {
        let db = setup().await;
        let author = create(&db, "Alice", Some("test author")).await.unwrap();
        assert!(author.id > 0);
        assert_eq!(author.name, "Alice");
        assert_eq!(author.description.as_deref(), Some("test author"));
    }

    #[tokio::test]
    async fn create_with_no_description() {
        let db = setup().await;
        let author = create(&db, "Bob", None).await.unwrap();
        assert!(author.description.is_none());
    }

    #[tokio::test]
    async fn get_returns_existing_author() {
        let db = setup().await;
        let created = create(&db, "Carol", None).await.unwrap();
        let found = get(&db, created.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Carol");
    }

    #[tokio::test]
    async fn get_returns_none_for_missing_id() {
        let db = setup().await;
        let found = get(&db, 999).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn list_returns_all_authors() {
        let db = setup().await;
        create(&db, "A", None).await.unwrap();
        create(&db, "B", None).await.unwrap();
        create(&db, "C", None).await.unwrap();
        let all = list(&db).await.unwrap();
        assert_eq!(all.len(), 3);
    }

    #[tokio::test]
    async fn list_returns_empty_when_none() {
        let db = setup().await;
        let all = list(&db).await.unwrap();
        assert!(all.is_empty());
    }

    #[tokio::test]
    async fn update_changes_name_and_description() {
        let db = setup().await;
        let author = create(&db, "Old Name", None).await.unwrap();
        let updated = update(&db, author.id, "New Name", Some("desc")).await.unwrap();
        assert_eq!(updated.id, author.id);
        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.description.as_deref(), Some("desc"));
    }

    #[tokio::test]
    async fn update_can_clear_description() {
        let db = setup().await;
        let author = create(&db, "X", Some("original")).await.unwrap();
        let updated = update(&db, author.id, "X", None).await.unwrap();
        assert!(updated.description.is_none());
    }

    #[tokio::test]
    async fn update_returns_not_found_for_missing_id() {
        let db = setup().await;
        let result = update(&db, 999, "X", None).await;
        assert!(matches!(result, Err(DbError::NotFound)));
    }

    #[tokio::test]
    async fn delete_removes_author() {
        let db = setup().await;
        let author = create(&db, "Temp", None).await.unwrap();
        delete(&db, author.id).await.unwrap();
        let found = get(&db, author.id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn delete_nonexistent_is_ok() {
        let db = setup().await;
        delete(&db, 999).await.unwrap();
    }
}
