use crate::db::Db;
use crate::error::DbError;
use crate::models::document::Document;

pub async fn create(db: &Db, name: &str, description: Option<&str>) -> Result<Document, DbError> {
    let row = sqlx::query_as::<_, Document>(
        "INSERT INTO Documents (name, description) VALUES (?, ?) RETURNING *",
    )
    .bind(name)
    .bind(description)
    .fetch_one(&db.pool)
    .await?;
    Ok(row)
}

pub async fn get(db: &Db, id: i64) -> Result<Option<Document>, DbError> {
    let row = sqlx::query_as::<_, Document>("SELECT * FROM Documents WHERE id = ?")
        .bind(id)
        .fetch_optional(&db.pool)
        .await?;
    Ok(row)
}

pub async fn list(db: &Db) -> Result<Vec<Document>, DbError> {
    let rows = sqlx::query_as::<_, Document>("SELECT * FROM Documents ORDER BY id")
        .fetch_all(&db.pool)
        .await?;
    Ok(rows)
}

pub async fn update(
    db: &Db,
    id: i64,
    name: &str,
    description: Option<&str>,
) -> Result<Document, DbError> {
    let row = sqlx::query_as::<_, Document>(
        "UPDATE Documents SET name = ?, description = ? WHERE id = ? RETURNING *",
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
    sqlx::query("DELETE FROM Documents WHERE id = ?")
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
    async fn create_returns_document_with_id() {
        let db = setup().await;
        let doc = create(&db, "My Doc", Some("a description")).await.unwrap();
        assert!(doc.id > 0);
        assert_eq!(doc.name, "My Doc");
        assert_eq!(doc.description.as_deref(), Some("a description"));
    }

    #[tokio::test]
    async fn create_with_no_description() {
        let db = setup().await;
        let doc = create(&db, "Bare Doc", None).await.unwrap();
        assert!(doc.description.is_none());
    }

    #[tokio::test]
    async fn get_returns_existing_document() {
        let db = setup().await;
        let created = create(&db, "Fetch Me", None).await.unwrap();
        let found = get(&db, created.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Fetch Me");
    }

    #[tokio::test]
    async fn get_returns_none_for_missing_id() {
        let db = setup().await;
        let found = get(&db, 999).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn list_returns_all_documents() {
        let db = setup().await;
        create(&db, "Doc 1", None).await.unwrap();
        create(&db, "Doc 2", None).await.unwrap();
        let all = list(&db).await.unwrap();
        assert_eq!(all.len(), 2);
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
        let doc = create(&db, "Old", None).await.unwrap();
        let updated = update(&db, doc.id, "New", Some("updated desc")).await.unwrap();
        assert_eq!(updated.id, doc.id);
        assert_eq!(updated.name, "New");
        assert_eq!(updated.description.as_deref(), Some("updated desc"));
    }

    #[tokio::test]
    async fn update_can_clear_description() {
        let db = setup().await;
        let doc = create(&db, "X", Some("original")).await.unwrap();
        let updated = update(&db, doc.id, "X", None).await.unwrap();
        assert!(updated.description.is_none());
    }

    #[tokio::test]
    async fn update_returns_not_found_for_missing_id() {
        let db = setup().await;
        let result = update(&db, 999, "X", None).await;
        assert!(matches!(result, Err(DbError::NotFound)));
    }

    #[tokio::test]
    async fn delete_removes_document() {
        let db = setup().await;
        let doc = create(&db, "Temporary", None).await.unwrap();
        delete(&db, doc.id).await.unwrap();
        let found = get(&db, doc.id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn delete_nonexistent_is_ok() {
        let db = setup().await;
        delete(&db, 999).await.unwrap();
    }
}
