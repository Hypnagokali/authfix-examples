use actix_session::{
    SessionMiddleware,
    config::{PersistentSession, SessionLifecycle},
    storage::CookieSessionStore,
};
use actix_web::{HttpResponse, HttpServer, Responder, cookie::Key, get, middleware::Logger};
use authfix::{
    AuthToken,
    login::{LoadUserByCredentials, LoadUserError, LoginToken},
    session::{AccountInfo, app_builder::SessionLoginAppBuilder},
};
use serde::{Deserialize, Serialize};

// A user intended for session authentication must derive or implement Serialize, and Deserialize.
#[derive(Serialize, Deserialize)]
struct User {
    name: String,
}

impl AccountInfo for User {}

// Struct that handles the authentication
struct AuthenticationService;

// LoadUsersByCredentials uses async_trait, so its needed when implementing the trait for AuthenticationService
// async_trait is re-exported by authfix.
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
    let user = auth_token.authenticated_user();
    HttpResponse::Ok().json(&*user)
}

pub fn session_config(key: Key) -> SessionMiddleware<CookieSessionStore> {
    let persistent_session = PersistentSession::default();
    let lc = SessionLifecycle::PersistentSession(persistent_session);
    SessionMiddleware::builder(CookieSessionStore::default(), key)
        .cookie_name("sessionId".to_string())
        .cookie_http_only(true)
        .cookie_same_site(actix_web::cookie::SameSite::Strict)
        .cookie_secure(false)
        .session_lifecycle(lc)
        .build()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let key = Key::generate();
    HttpServer::new(move || {
        // SessionLoginAppBuilder is the simplest way to create an App instance configured with session based authentication
        SessionLoginAppBuilder::create_with_session_middleware(
            AuthenticationService,
            session_config(key.clone()),
        )
        // create App instance with build()
        .build()
        .wrap(Logger::default())
        .service(secured)
    })
    .bind("127.0.0.1:7080")?
    .run()
    .await
}
