mod asset;
mod mailchannels;
mod utils;

use worker::wasm_bindgen::prelude::*;
use worker::*;

// https://www.webdevsplanet.com/post/how-to-generate-rsa-private-and-public-keys

// fn log_request(req: &Request) {
//     console_log!(
//         "{} - [{}], located at: {:?}, within: {}",
//         Date::now().to_string(),
//         req.path(),
//         req.cf().coordinates().unwrap_or_default(),
//         req.cf().region().unwrap_or("unknown region".into())
//     );
// }

#[event(fetch, responde_with_errors)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    //log_request(&req);
    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    Router::new()
        .get_async("/*asset", |request, context| asset::serve(request, context))
        .post_async("/contact", |request, context| contact(request, context))
        .run(req, env)
        .await
}

pub async fn contact(mut request: Request, context: RouteContext<()>) -> Result<Response> {
    let bad_request = Response::error("Bad request", 400);
    let form = request.form_data().await?;
    /* get form fields */
    let name = match form.get("name") {
        Some(FormEntry::Field(name)) => name,
        _ => return bad_request,
    };
    let email = match form.get("email") {
        Some(FormEntry::Field(email)) => email,
        _ => return bad_request,
    };
    let affiliation = match form.get("affiliation") {
        Some(FormEntry::Field(affiliation)) => affiliation,
        _ => return bad_request,
    };
    let message = match form.get("message") {
        Some(FormEntry::Field(message)) => message,
        _ => return bad_request,
    };

    console_debug!(
        "name = {}, email = {}, affiliation = {}, message = \"{}\"",
        name,
        email,
        affiliation,
        message
    );
    /* prepare email */
    use mailchannels::{Body, Contact, Content, Dkim, Personalization};

    let dkim_private_key = context.secret("DKIM_PRIVATE_KEY")?.to_string();
    let dkim = Dkim::new("allwright.io", "mailchannels", dkim_private_key);
    let content = Content::new(
        "text/plain",
        format!(
            r"
name: {}
email: {}
affiliation: {}
message: {}
    ",
            name, email, affiliation, message
        ),
    );
    let from = Contact::new("no-reply@allwright.io", name);
    let personalization = Personalization::new(dkim).to(Contact::new(
        "contact@learnrobotics.io",
        "Michael Allwright",
    ));
    let body = Body::new(from, "Contact request")
        .personalization(personalization)
        .content(content);

    let body = serde_json::to_string(&body).unwrap();
    let mut headers = Headers::new();
    headers.append("Content-Type", "application/json").unwrap();
    let mut send_request = RequestInit::new();
    send_request
        .with_method(Method::Post)
        .with_headers(headers)
        .with_body(JsValue::from_str(&body).into());
    let send_request =
        Request::new_with_init("https://api.mailchannels.net/tx/v1/send", &send_request)?;

    let send_response = Fetch::Request(send_request).send().await?;

    Ok(send_response)
}
