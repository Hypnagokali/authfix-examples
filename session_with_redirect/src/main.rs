use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use actix_web::{HttpServer, Responder, cookie::Key, get, web::Query};
use authfix::{
    AuthToken, async_trait,
    login::{LoadUserByCredentials, LoadUserError, LoginToken},
    multifactor::config::{HandleMfaRequest, MfaConfig, MfaError},
    session::{
        AccountInfo,
        app_builder::SessionLoginAppBuilder,
        auth_flow::LoginError,
        config::Routes,
        factor_impl::random_code_auth::{
            CodeSendError, CodeSender, MfaRandomCodeFactor, RandomCode,
        },
    },
};
use chrono::{DateTime, Local};
use maud::html;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    name: String,
}

impl AccountInfo for User {}

// Struct that handles the authentication
struct AuthenticationService;

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

// This struct will be used to send the random code to the user.
struct DummySender;

// To do so, it has to implement the CodeSender trait.
impl CodeSender for DummySender {
    type User = User;
    async fn send_code(&self, user: &User, random_code: RandomCode) -> Result<(), CodeSendError> {
        // send the code to the user
        let until = DateTime::<Local>::from(*random_code.valid_until());
        println!(
            "Send code to: {}\nyour code '{}' is valid until {}",
            user.name,
            random_code.value(),
            until.format("%H:%M")
        );
        Ok(())
    }
}

// Responsible for deciding whether the user needs a second factor and, if so, which one.
struct AlwaysAskForRandomCode;

// This implementation uses MfaRandomCodeFactor for all login attempts.
#[async_trait(?Send)]
impl HandleMfaRequest for AlwaysAskForRandomCode {
    type User = User;

    async fn mfa_id_by_user(&self, _: &Self::User) -> Result<Option<String>, MfaError> {
        Ok(Some(MfaRandomCodeFactor::id()))
    }
}

// Provides the login form. The path must be the same as the one in Routes (Routes::login).
// Notice: I used the maud crate as template engine here.
#[get("/login")]
async fn login(query: Query<LoginError>) -> impl Responder {
    html! {
        html {
            body {
                h1 { "Beautiful login page 🫣" }
                div {
                    @if query.is_error() {
                        p style="color: red;" { "Login failed, please try again." }
                    }
                }
                div {
                    form method="post" {
                        label style="display: inline-block;width: 100px" for="email" { "E-Mail:" }
                        input type="email" name="email" value="test@example.org" id="email" {}
                        br;
                        label style="display: inline-block;width: 100px" for="password" { "Password:" }
                        input type="password" name="password" value="password" id="password" {}
                        br;
                        input type="submit" value="Login" {}
                    }
                }
            }
        }
    }
}

// Provides MFA form.
#[get("/login/mfa")]
async fn login_mfa(query: Query<LoginError>) -> impl Responder {
    html! {
        html {
            body {
                h1 { "A code is needed" }
                div {
                    @if query.is_error() {
                        p style="color: red;" { "Code wrong, please try again." }
                    }
                }
                div {
                    form method="post" {
                        label style="display: inline-block;width: 100px" for="code" { "Code:" }
                        input type="text" name="code" value="123" id="code" {}
                        br;
                        input type="submit" value="Login" {}
                    }
                }
            }
        }
    }
}

// A logout page.
#[get("/logout")]
async fn logout() -> impl Responder {
    html! {
        html {
            body {
                h1 { "Do you really want to leave?" }
                div {
                    form method="post" {
                        input type="submit" value="Logout" {}
                    }
                }
            }
        }
    }
}

// The private page of the user.
#[get("/secured")]
async fn secured(token: AuthToken<User>) -> impl Responder {
    html! {
        html {
            body {
                h1 { "Private page!" }
                p { "User: " (token.authenticated_user().name) }
                a href="/logout" {
                    "I want to logout"
                }
            }
        }
    }
}

// An index page, just for the sake of completeness.
#[get("/")]
async fn index() -> impl Responder {
    html! {
        html {
            body {
                h1 { "Landing page" }
                a href="/secured" {
                    "Go to secured route"
                }
            }
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let key = Key::generate();

    // The code sender could be a shared service (MailSender, SmsSender), so we have to wrap in an Arc.
    let code_sender = Arc::new(DummySender);

    HttpServer::new(move || {
        // Create the factor:
        let code_factor = Box::new(MfaRandomCodeFactor::new(
            || {
                // Provide a function that generates a random code.
                // This one will always generate 123 and is valid for 5 minutes.
                RandomCode::new(
                    "123",
                    SystemTime::now()
                        .checked_add(Duration::from_secs(300))
                        .unwrap(),
                )
            },
            Arc::clone(&code_sender),
        ));

        // Then build the MfaConfig with the Factor and the MfaHandler
        let mfa_config = MfaConfig::new(vec![code_factor], AlwaysAskForRandomCode);

        // The default redirect is used after a successful login or if a user tries accessing a login route while alredy logged in.
        // The default value is "/".
        let routes = Routes::default().set_default_redirect("/secured");

        SessionLoginAppBuilder::create(AuthenticationService, key.clone())
            .set_mfa(mfa_config)
            .set_login_routes_and_public_paths(routes, vec!["/"])
            // Activate the redirect flow:
            .with_redirect_flow()
            .build()
            .service(index)
            .service(secured)
            .service(login)
            .service(login_mfa)
            .service(logout)
    })
    .bind(("127.0.0.1", 7080))?
    .run()
    .await
}
