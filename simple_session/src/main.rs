use actix_web::{HttpResponse, HttpServer, Responder, cookie::Key, get};
use authfix::{
    AccountInfo, AuthToken,
    async_trait::async_trait,
    config::Routes,
    login::{LoadUserByCredentials, LoadUserError, LoginToken},
    session::app_builder::SessionLoginAppBuilder,
};
use serde::{Deserialize, Serialize};

// A user intended for session authentication must derive or implement Clone, Serialize, and Deserialize.
#[derive(Clone, Serialize, Deserialize)]
struct User {
    name: String,
}

// AccountInfo trait is used for disabling the user or to lock the account
// The user is enabled by default
impl AccountInfo for User {}

// Struct that handles the authentication
struct AuthenticationService;

// LoadUsersByCredentials uses async_trait, so its needed when implementing the trait for AuthenticationService
// async_trait is re-exported by authfix.
#[async_trait]
impl LoadUserByCredentials for AuthenticationService {
    type User = User;

    async fn load_user(&self, login_token: &LoginToken) -> Result<Self::User, LoadUserError> {
        // load user by email logic and check password
        // currently authfix does not provide hashing functions, you can use for example https://docs.rs/argon2/latest/argon2/
        if login_token.email == "test@example.org" && login_token.password == "password" {
            Ok(User {
                name: "Johnny".to_owned(),
            })
        } else {
            Err(LoadUserError::LoginFailed)
        }
    }
}

// You have access to the user via the AuthToken extractor in secured routes.
#[get("/secured")]
async fn secured(auth_token: AuthToken<User>) -> impl Responder {
    let user = auth_token.get_authenticated_user();

    HttpResponse::Ok().json(&*user)
}

#[get("/public")]
async fn public() -> impl Responder {
    HttpResponse::Ok().json(r#"{ value: "everyone can see this" }"#)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // In production, you should read the key from an external source so that you can use sessions across restarts
    // see: https://docs.rs/actix-session/0.10.1/actix_session/
    let key = Key::generate();
    HttpServer::new(move || {
        // SessionLoginAppBuilder is the simplest way to create an App instance configured with session based authentication
        SessionLoginAppBuilder::create(AuthenticationService, key.clone())
            // configure path names for the login handler and define paths that are not secured.
            // Routes::default() registers: /login, /login/mfa, /logout
            .set_login_routes_and_public_paths(Routes::default(), vec!["/public"])
            // create App instance with build()
            .build()
            .service(secured)
            .service(public)
    })
    .bind("127.0.0.1:7080")?
    .run()
    .await
}
