use actix_web::{get, post, web::{Form, Query, Redirect}, HttpResponse, Responder};
use authfix::multifactor::factor_impl::authenticator::{Authenticator, TotpSecretGenerator};
use serde::Deserialize;

#[derive(Deserialize)]
struct QrCodeReq {
    pub code: String,
    pub secret: String,
}

#[derive(Deserialize)]
struct CodeAnswer {
    success: Option<bool>,
}


#[post("/qr-code/check")]
pub async fn check_code(qr_code_req: Form<QrCodeReq>) -> impl Responder {
    if Authenticator::verify(&qr_code_req.secret, &qr_code_req.code, 0) {
        Redirect::to("/qr-code?success=true").see_other()
    } else {
        Redirect::to("/qr-code?success=false").see_other()
    }
}

#[get("/qr-code")]
pub async fn show_qr_code(query: Query<CodeAnswer>) -> impl Responder {   
    let generator = TotpSecretGenerator::new("MyApp", "test@example.org");

    let qr_code = generator.qr_code().unwrap();
    let secret = generator.secret();

    let html = match query.success {
        Some(true) => {
            r#"<p style="color: green">Your code has been registered</p>"#.to_owned()
        },
        Some(false) => {
            r#"<p style="color: red">Your code was wrong</p>"#.to_owned()
        }
        None => { 
            format!(r#"
                <div>
                {qr_code}
                </div>
                <div>
                    <p>If the QR code does not work, you can register the secret manually</p>
                    <p><strong>{secret}</strong></p>
                </div>
                <div>
                    <p>Check the code from your authenticator app:</p>
                    <form method="POST" action="/qr-code/check">
                        <input type="text" name="code">
                        <input type="hidden" name="secret" value="{secret}">
                        <input type="submit" value="Register">
                    </form>            
                </div>
            "#)
        }
    };

    HttpResponse::Ok().body(
        format!(r#"
        <!DOCTYPE html>
        <html>
        <body>
            <h1>Register Authenticator</h1>
            {html}
        </body>
        </html>
        "#
    ))
}