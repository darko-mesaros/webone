use askama::Template;

use crate::contacts::{Contact, NewContact};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub q: String,
    pub contacts: Vec<Contact>,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Template)]
#[template(path = "new.html")]
pub struct NewContactTemplate {
    pub contact: Option<NewContact>,
}

#[derive(Template)]
#[template(path = "show.html")]
pub struct ShowContactTemplate {
    pub contact: Contact,
}
#[derive(Template)]
#[template(path = "edit.html")]
pub struct EditContactTemplate {
    pub contact: Contact,
}
#[derive(Template)]
#[template(path = "error.html")]
pub struct Error5xxTemplate {
    pub error: String,
}
#[derive(Template)]
#[template(path = "success_redirect.html")]
pub struct SuccessRedirectTemplate {
    pub success_message: String,
}

#[derive(Template)]
#[template(path = "error_message.html")]
pub struct ErrorMessageTemplate {
    pub error_message: String,
}
