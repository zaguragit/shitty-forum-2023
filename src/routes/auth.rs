use std::sync::Mutex;

use crate::{auth::{Auth, UserSession, build_session_cookie}, db::DB};
use actix_web::{get, post, web::{Form, Data}, cookie::Cookie, HttpResponse, http::{StatusCode, header::LOCATION}};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Signup {
    user_name: String,
    password: String,
}
#[derive(Deserialize)]
pub struct Login {
    user_name: String,
    password: String,
}

#[post("/auth/signup")]
pub async fn auth_signup(auth: Data<Mutex<Auth>>, db: Data<Mutex<DB>>, Form(form): Form<Signup>) -> HttpResponse {
    let mut db = db.lock().unwrap();
    let mut auth = auth.lock().unwrap();
    match auth.signup(form.user_name.as_str(), form.password.as_str(), &mut db) {
        Ok(_) => {
            let (_, session_id) = auth.login(form.user_name.as_str(), form.password.as_str()).unwrap();
            HttpResponse::build(StatusCode::SEE_OTHER)
                .append_header((LOCATION, "/"))
                .cookie(build_session_cookie(&session_id))
                .finish()
        },
        Err(e) => 
            HttpResponse::build(StatusCode::SEE_OTHER)
                .append_header((LOCATION, format!("/signup?error={}", e.to_string())))
                .finish(),
    }
}

#[post("/auth/login")]
pub async fn auth_login(auth: Data<Mutex<Auth>>, Form(form): Form<Login>) -> HttpResponse {
    let mut auth = auth.lock().unwrap();
    let session_id = auth.login(form.user_name.as_str(), form.password.as_str());
    match session_id {
        Ok((_, session_id)) => {
            HttpResponse::build(StatusCode::SEE_OTHER)
                .append_header((LOCATION, "/"))
                .cookie(build_session_cookie(&session_id))
                .finish()
        },
        Err(e) =>
            HttpResponse::build(StatusCode::SEE_OTHER)
                .append_header((LOCATION, format!("/login?error={}", e.to_string())))
                .finish()
    }
}

#[get("/auth/logout")]
pub async fn auth_logout(auth: Data<Mutex<Auth>>, user: UserSession) -> HttpResponse {
    let mut auth = auth.lock().unwrap();
    auth.logout(user);
    let mut cookie = Cookie::new("session-id", "");
    cookie.make_removal();
    HttpResponse::build(StatusCode::SEE_OTHER)
        .append_header((LOCATION, "/"))
        .cookie(cookie)
        .finish()
}