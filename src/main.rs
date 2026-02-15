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
use tower_http::services::ServeDir;
use webone::templates::ErrorMessageTemplate;
use webone::templates::SuccessRedirectTemplate;
use webone::{
    contacts::{Contact, NewContact},
    templates::{EditContactTemplate, IndexTemplate, NewContactTemplate, ShowContactTemplate},
    utils::AppError,
};

// For pagination
const PER_PAGE: i64 = 10;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
}
#[derive(Deserialize, Debug)]
struct ContactSearchParams {
    q: Option<String>,
    page: Option<i64>,
}
#[derive(Deserialize, Debug)]
struct ValidateParams {
    email: Option<String>,
    phone_number: Option<String>,
}

async fn index() -> impl IntoResponse {
    Redirect::permanent("/contacts")
}


/// Template function: Gets all contacts and renders them to the HTML. Limits the amount of
/// contacts displayed based on the `PER_PAGE` constant.
#[axum::debug_handler]
async fn contacts(
    State(state): State<AppState>,
    query: Query<ContactSearchParams>,
) -> Result<(StatusCode, Html<String>), AppError> {

    let page = query.page.unwrap_or(1);
    let contacts: Vec<Contact> = match &query.q {
        Some(search_query) => Contact::search(&state.db, search_query, page, PER_PAGE).await?,
        None => Contact::get_all(&state.db, page, PER_PAGE).await?,
    };
    let index_template = IndexTemplate {
        q: query.q.clone().unwrap_or_default(),
        contacts,
        page,
        per_page: PER_PAGE
    };

    // PROCESS TEMPLATE
    let html = index_template.render()?;
    Ok((StatusCode::OK, Html(html)))
}

/// New contact creation from form data. It performs checks to verify if the email and phone are
/// unique. Otherwise it creates the contact, flashes the success message on screen and redirects.
#[axum::debug_handler]
async fn post_new_contact(
    State(state): State<AppState>,
    Form(new_contact): Form<NewContact>,
) -> Result<Html<String>, AppError> {
    // Axums Form extractor handles the NewContact
    // Validate fields
    let valid_email = Contact::validate_email(&state.db, new_contact.email.as_str()).await?;
    let valid_phone = Contact::validate_phone(&state.db, new_contact.phone_number.as_str()).await?;

    if valid_email || valid_phone {
        let error_message = ErrorMessageTemplate {
            error_message: "Email and/or phone number is already in use. Contact NOT SAVED".into()
        };
        let html = error_message.render()?;
        Ok(Html(html))
    } else {
        //Err(anyhow!("The email and/or phone number is already in use").into())
        Contact::create(&state.db, new_contact).await?;
        let success_template = SuccessRedirectTemplate { success_message: "Contact succesfully created. Redirecting".into()};
        let html = success_template.render()?;
        Ok(Html(html))
    }
}


/// Template function: Renders the new contact creation HTML.
#[axum::debug_handler]
async fn get_new_contact() -> Result<(StatusCode, Html<String>), AppError> {
    let new_template = NewContactTemplate { contact: None };
    let html = new_template.render()?;
    Ok((StatusCode::OK, Html(html)))
}

/// Template function: Renders the individual contact HTML with the `Contact` data.
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

/// Template function: Renders the Edit contact HTML with the `Contact` data.
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
/// Updates existing contact by passing all the parameters, and updating the `Contact` struct from
/// the new data. Then calling the `.update()` method with `&self` to make the changes in the
/// database.
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

/// Deletes contact by extracting the `id` from the path. 
///
/// Example usage: 
/// By passing on a HTTP `DELETE` method to the `/contacts/{id}` path, we can trigger this function.
#[axum::debug_handler]
async fn delete_contact(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Redirect, AppError> {
    Contact::delete(&state.db, id).await?;

    Ok(Redirect::to("/contacts"))
}

/// Validates input parameters by checking if email and/or phone already exist in the database.
/// Returns form-level error HTML and updates the submit button state via OOB swap.
///
/// This validates BOTH fields together to avoid race conditions where fixing one field
/// might incorrectly enable the button while the other field is still invalid.
///
/// Example usage:
/// A GET request from `HTMX` when entering an email or phone into a form. The response
/// replaces the `#form-errors` div and updates the submit button via `hx-swap-oob`.
#[axum::debug_handler]
async fn validate_input(
    State(state): State<AppState>,
    Query(params): Query<ValidateParams>,
) -> Result<(StatusCode, Html<String>), AppError> {
    // Validate both fields (either may be None if not yet entered)
    let email_exists = match &params.email {
        Some(email) if !email.is_empty() => {
            Contact::validate_email(&state.db, email).await?
        }
        _ => false,
    };

    let phone_exists = match &params.phone_number {
        Some(phone) if !phone.is_empty() => {
            Contact::validate_phone(&state.db, phone).await?
        }
        _ => false,
    };

    // Build error message and button state based on combined validation
    let (error_msg, button_html) = match (email_exists, phone_exists) {
        (true, true) => (
            "⛔ Email and phone number already exist in your contacts",
            r#"<button id="submit-btn" hx-swap-oob="true" disabled class="btn-disabled">Cannot save</button>"#
        ),
        (true, false) => (
            "⛔ This email already exists in your contacts",
            r#"<button id="submit-btn" hx-swap-oob="true" disabled class="btn-disabled">Cannot save</button>"#
        ),
        (false, true) => (
            "⛔ This phone number already exists in your contacts",
            r#"<button id="submit-btn" hx-swap-oob="true" disabled class="btn-disabled">Cannot save</button>"#
        ),
        (false, false) => (
            "",  // No error
            r#"<button id="submit-btn" hx-swap-oob="true">Save</button>"#
        ),
    };

    // Return error message (goes to #form-errors) + OOB button update
    Ok((StatusCode::OK, Html(format!(
        r#"{}<span hx-swap-oob="true" id="form-errors">{}</span>"#,
        button_html, error_msg
    ))))
}
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Tracing
    tracing_subscriber::fmt::init();

    // Connect to Database:
    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = SqlitePool::connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    // Set the app state
    let state = AppState { db: pool };

    // Create the axum router
    let app = Router::new()
        .route("/", get(index)) // Main Page redirects to /contacts
        .route("/contacts", get(contacts)) // Shows the contaxt
        .route("/contacts/new", post(post_new_contact).get(get_new_contact)) // New POST endpoint
        .route("/contacts/{id}", get(show_contact).delete(delete_contact)) // Contact GET/DELETE
        .route( // Edit contact POST endpoint
            "/contacts/{id}/edit",
            post(post_edit_contact).get(get_edit_contact),
        )
        .route("/contacts/validate", get(validate_input)) // Endpoint for validating input
        .nest_service("/static", ServeDir::new("static")) // Serve static content
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2911").await.unwrap();

    axum::serve(listener, app).await?;
    Ok(())
}
