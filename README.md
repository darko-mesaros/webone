# Web 1

> ⚠️NOTE: The code provided here is AS IS. Feel free to use it for inspiration. But do not implement anything from here in any sort of production environment. You have been warned.

This is my little learning repo where I build hypermedia powered applications. Based on the [hypermedia.systems](https://hypermedia.systems) book.

## What This Is

A contact management web app built with **Axum + HTMX + SQLite** following HATEOAS principles. Server returns HTML, client navigates via hyperlinks and forms. No client-side JavaScript framework needed.

## Tech Stack

- **Axum**: Web framework with routing and state management
- **Askama**: HTML templating (Jinja2-style syntax)
- **SQLX**: Compile-time verified SQLite queries
- **HTMX**: Hypermedia-driven interactions (embedded in templates)
- **Tokio**: Async runtime

## Project Structure

```
src/
├── main.rs       - Axum routes, handlers, app state
├── contacts.rs   - Contact model with CRUD operations
├── templates.rs  - Askama template structs
├── utils.rs      - Custom error type (AppError)
└── lib.rs        - Module exports

templates/        - Askama HTML templates
migrations/       - SQLX database migrations
```

## Key Features

### CRUD Operations
- **List contacts** with pagination (10 per page)
- **Search contacts** by first/last name
- **Create contact** with validation
- **View individual contact**
- **Edit contact** with pre-filled form
- **Delete contact** via HTTP DELETE

### HTMX-Powered Interactions
- **Live validation**: Email/phone uniqueness checked on input
- **Partial updates**: Error messages swap into `.error` divs
- **Form submissions**: POST without full page reload
- **Success redirects**: Flash message then redirect to list

### Database Layer (`contacts.rs`)
All operations use `sqlx::query_as!` for type safety:
- `Contact::get_all()` - Paginated list
- `Contact::search()` - Filter by name with LIKE
- `Contact::find_by_id()` - Single contact lookup
- `Contact::create()` - Insert new contact
- `Contact::update()` - Update existing contact
- `Contact::delete()` - Remove contact
- `Contact::validate_email()` - Check uniqueness
- `Contact::validate_phone()` - Check uniqueness

### Error Handling
Custom `AppError` type wraps `anyhow::Error` and implements `IntoResponse`:
- Returns HTML error page on failure
- Logs errors via `tracing`
- Graceful degradation if template rendering fails

## Routes

```
GET  /                      → Redirect to /contacts
GET  /contacts              → List all contacts (with search/pagination)
GET  /contacts/new          → New contact form
POST /contacts/new          → Create contact
GET  /contacts/{id}         → Show single contact
GET  /contacts/{id}/edit    → Edit contact form
POST /contacts/{id}/edit    → Update contact
DELETE /contacts/{id}       → Delete contact
GET  /contacts/validate     → Validate email/phone (HTMX endpoint)
```

## Running It

```bash
# Set up database
export DATABASE_URL="sqlite:database.db"
sqlx migrate run

# Run the server
cargo run
# Listens on http://0.0.0.0:2911
```

Or use the justfile:
```bash
just run
```

## Learning Notes

This project demonstrates:
- **Repository pattern**: Thin wrappers around database operations
- **HATEOAS**: Server drives navigation, client follows links
- **Progressive enhancement**: Works without JavaScript, enhanced with HTMX
- **Type-safe SQL**: SQLX macros catch errors at compile time
- **Custom error types**: Clean error handling in Axum handlers

