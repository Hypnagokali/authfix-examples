use std::sync::Arc;

use actix_web::{HttpResponse, HttpServer, Responder, cookie::Key, get};
use authfix::{
    async_trait::async_trait, config::Routes, login::{LoadUserByCredentials, LoadUserError, LoginToken}, mfa::{HandleMfaRequest, MfaConfig, MfaError}, multifactor::{authenticator::{AuthenticatorFactor, MFA_ID_AUTHENTICATOR_TOTP}, GetTotpSecretError, TotpSecretRepository}, session::app_builder::SessionLoginAppBuilder, AccountInfo, AuthToken
};
use google_authenticator::GoogleAuthenticator;
use serde::{Deserialize, Serialize};

const SECRET: &str = "I3VFM3JKMNDJCDH5BMBEEQAW6KJ6NOE3";

// This is our user
#[derive(Clone, Serialize, Deserialize)]
struct User {
    name: String,
}

// AccountInfo for user with default implementation
impl AccountInfo for User {}

// The TotpSecretRepository is for loading the secret
struct StaticTotpSecretRepo;

#[async_trait]
impl TotpSecretRepository<User> for StaticTotpSecretRepo {
    async fn get_auth_secret(&self, _user: &User) -> Result<String, GetTotpSecretError> {
        // here you would get the secret from the user
        Ok(SECRET.to_owned())
    }
}

struct MfaHandler;

// HandleMfaRequest is used, to decide if a challenge is needed and if yes, which one
// The trait is not Send, so its only allowed to use inside the middleware
#[async_trait(?Send)]
impl HandleMfaRequest for MfaHandler {
    type User = User;

    async fn get_mfa_id_by_user(&self, _user: &Self::User) -> Result<Option<String>, MfaError> {
        // To decide which challenge should be used, you have to implement this method
        // if it returns None, the user needs no mfa check
        Ok(Some(MFA_ID_AUTHENTICATOR_TOTP.to_owned()))
    }

    // By default, every login needs a mfa challenge, but you can override this behaviour by implementing is_condition_met:
    // async fn is_condition_met(&self, user: &Self::User, req: HttpRequest) -> bool {
    //     true
    // }

    // If you want to mutate the response after success (e.g. setting a cookie) you may implement handle_success:
    // async fn handle_success(&self, user: &Self::User, mut res: HttpResponse) -> HttpResponse {
    //     res
    // }
}

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

#[get("/code")]
async fn code() -> impl Responder {
    let auth = GoogleAuthenticator::new();
    let code = auth.get_code(SECRET, 0).unwrap();

    HttpResponse::Ok().body(code)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let key = Key::generate();
    HttpServer::new(move || {
        SessionLoginAppBuilder::create(AuthenticationService, key.clone())
            .set_login_routes_and_public_paths(Routes::default(), vec!["/code"])
            // Arc is used, because the TotpSecret repo might be a shared service
            .set_mfa(MfaConfig::new(vec![Box::new(AuthenticatorFactor::new(Arc::new(StaticTotpSecretRepo)))], MfaHandler))
            .build()
            .service(secured)
            .service(code)
    })
    .bind("127.0.0.1:7080")?
    .run()
    .await
}
