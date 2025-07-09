use actix_web::{App, HttpServer, Responder, get};
use authfix::AuthToken;
use maud::html;

struct User {
    pub name: String,
}

#[get("/secured")]
async fn secured(token: AuthToken<User>) -> impl Responder {
    html! {
        html {
            body {
                h1 { "Private page!" }
                p { "User: " (token.get_authenticated_user().name) }
            }
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(secured))
        .bind(("127.0.0.1", 6789))?
        .run()
        .await
}
