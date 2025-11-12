use serde::Deserialize;
use sqlx::SqlitePool;

// TODO: Figure out how to get creation errors.
// So far I have no way to error out here besides just having the database freak out.
#[derive(Debug, Deserialize)]
pub struct NewContactErrors {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone_number: Option<String>,
    pub email: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct NewContact {
    pub first_name: String,
    pub last_name: String,
    pub phone_number: String,
    pub email: String,
    pub errors: Option<NewContactErrors>,
}

#[derive(Debug)]
pub struct Contact {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub phone_number: String,
    pub email: String,
    pub created_at: String,
}

impl Contact {
    /// Update the existing contact from a NewContact. This is useful when updating contacts via
    /// the edit form as we don't have to pass the entire Contact (id, created_at)
    pub fn update_from(&mut self, new: NewContact) {
        self.first_name = new.first_name;
        self.last_name = new.last_name;
        self.phone_number = new.phone_number;
        self.email = new.email;
    }
    pub async fn create(pool: &SqlitePool, new: NewContact) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Contact,
            "INSERT INTO contacts (first_name, last_name, phone_number, email) VALUES (?, ?, ?, ?) RETURNING *",
            new.first_name,
            new.last_name,
            new.phone_number,
            new.email,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn update(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!(
        "UPDATE contacts SET first_name = ?, last_name = ?, phone_number = ?, email = ? WHERE id = ?",
            self.first_name,
            self.last_name,
            self.phone_number,
            self.email,
            self.id,
        )
            .execute(pool)
            .await
            .map(|_| ()) // Like Ok(())
    }
    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM contacts WHERE id = ?", id,)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_all(pool: &SqlitePool, page: i64, per_page: i64) -> Result<Vec<Contact>, sqlx::Error> {
        let offset = (page - 1) * per_page;
        sqlx::query_as!(
            Contact,
            "SELECT * FROM contacts ORDER BY id LIMIT ? OFFSET ?",
            per_page,
            offset,
            )
            .fetch_all(pool)
            .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(Contact, "SELECT * FROM contacts WHERE id = ?", id)
            .fetch_one(pool)
            .await
    }

    pub async fn search(pool: &SqlitePool, search: &str, page: i64, per_page: i64) -> Result<Vec<Contact>, sqlx::Error> {
        let pattern = format!("%{}%", search);
        let offset = (page - 1) * per_page;
        sqlx::query_as!(
            Contact,
            "SELECT * FROM contacts WHERE first_name LIKE ? OR last_name LIKE ? LIMIT ? OFFSET ?",
            pattern,
            pattern,
            per_page,
            offset,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn validate_email(pool: &SqlitePool, email: &str) -> Result<bool, sqlx::Error> {
        // Check if there is an entry in the database.
        let exists: bool = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM contacts where email = ?)",
            email
        )
        .fetch_one(pool)
        .await?
            == 1;

        Ok(exists)
    }

    pub async fn validate_phone(
        pool: &SqlitePool,
        phone_number: &str,
    ) -> Result<bool, sqlx::Error> {
        // Check if there is an entry in the database.
        let exists: bool = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM contacts where phone_number = ?)",
            phone_number
        )
        .fetch_one(pool)
        .await?
            == 1;

        Ok(exists)
    }
}
