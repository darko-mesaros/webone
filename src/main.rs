use anyhow::Context;
use askama::Template;
use axum::{
    Form, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
};
use serde::Deserialize;
use sqlx::SqlitePool;
use webone::{
    contacts::{Contact, NewContact},
    templates::{EditContactTemplate, IndexTemplate, NewContactTemplate, ShowContactTemplate}, utils::AppError,
};

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
}
#[derive(Deserialize, Debug)]
struct ContactSearchParams {
    q: Option<String>,
}

async fn index() -> impl IntoResponse {
    Redirect::permanent("/contacts")
}

async fn contacts(
    State(state): State<AppState>,
    query: Query<ContactSearchParams>,
) -> Result<(StatusCode, Html<String>), AppError> {
    let contacts: Vec<Contact> = match &query.q {
        Some(search_query) => Contact::search(&state.db, search_query).await?,
        None => Contact::get_all(&state.db).await?,
    };
    let index_template = IndexTemplate {
        q: query.q.clone().unwrap_or_default(),
        contacts,
    };

    // PROCESS TEMPLATE
    let html = index_template.render()?;
    Ok((StatusCode::OK, Html(html)))
}

#[axum::debug_handler]
async fn post_new_contact(
    State(state): State<AppState>,
    Form(new_contact): Form<NewContact>,
) -> Result<Redirect, AppError> {

    // Axums Form extractor handles the NewContact
    Contact::create(&state.db, new_contact).await?;
    Ok(Redirect::to("/contacts"))
}

#[axum::debug_handler]
async fn get_new_contact() -> Result<(StatusCode, Html<String>), AppError> {
    let new_template = NewContactTemplate { contact: None };
    let html = new_template.render()?;
    Ok((StatusCode::OK, Html(html)))
}
#[axum::debug_handler]
async fn show_contact(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<(StatusCode, Html<String>), AppError> {
    let contact = Contact::find_by_id(&state.db, id).await?;
    let show_template = ShowContactTemplate { contact };
    let html = show_template.render()?;
    Ok((StatusCode::OK, Html(html)))
}
#[axum::debug_handler]
async fn get_edit_contact(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<(StatusCode, Html<String>), AppError> {
    let contact = Contact::find_by_id(&state.db, id).await?;
    let edit_template = EditContactTemplate { contact };
    let html = edit_template.render()?;
    Ok((StatusCode::OK, Html(html)))
}
#[axum::debug_handler]
async fn post_edit_contact(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(new_contact): Form<NewContact>,
) -> Result<Redirect, AppError> {
    let mut contact = Contact::find_by_id(&state.db, id).await?;

    contact.update_from(new_contact);
    contact.update(&state.db).await?;
    Ok(Redirect::to("/contacts"))
}

#[axum::debug_handler]
async fn delete_contact(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Redirect, AppError> {
    Contact::delete(&state.db, id).await?;

    Ok(Redirect::to("/contacts"))
}
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Connect to Database:
    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = SqlitePool::connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    // Set the app state
    let state = AppState { db: pool };
    let app = Router::new()
        .route("/", get(index))
        .route("/contacts", get(contacts))
        .route("/contacts/new", post(post_new_contact).get(get_new_contact))
        .route("/contacts/{id}", get(show_contact))
        .route("/contacts/{id}/edit", post(post_edit_contact).get(get_edit_contact))
        .route("/contacts/{id}/delete", post(delete_contact))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2911").await.unwrap();

    axum::serve(listener, app).await?;
    Ok(())
}
