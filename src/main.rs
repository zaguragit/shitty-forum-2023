use std::{sync::Mutex, io, fs::read_to_string};
use actix_web::{web::{self, Data}, App, HttpServer, Responder, Result, middleware::Logger, http::{Method, StatusCode}, HttpResponse, Either};

use auth::{Auth, UserSession};
use db::DB;

use render::render_page;
use routes::*;

mod render;
mod routes;
mod data;

mod auth;
mod db;

async fn default_handler(req: Method, db: Data<Mutex<DB>>, user: Option<UserSession>) -> Result<impl Responder> {
    match req {
        Method::GET => {
            let db = db.lock().unwrap();
            let response = render_page(&db, user.as_ref(), || {
                read_to_string("assets/page/404.html").unwrap()
            })
                .customize()
                .with_status(StatusCode::NOT_FOUND);
            Ok(Either::Left(response))
        }
        _ => Ok(Either::Right(HttpResponse::MethodNotAllowed().finish())),
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let auth = Data::new(Mutex::new(Auth::init()));
    let db = Data::new(Mutex::new(DB::load()));
    HttpServer::new(move || {
        App::new()
            .service(auth_signup)
            .service(auth_login)
            .service(auth_logout)

            .service(page_home)
            .service(page_user)
            .service(page_topic)
            .service(page_thread)

            .service(page_settings)
            .service(page_login)
            .service(page_signup)
            .service(page_create_thread)

            .service(page_search)
            .service(page_inspection)

            .service(make_reply)
            .service(make_thread)
            .service(favorite_topic)
            .service(favorite_thread)
            .service(update_settings)

            .service(delete_reply)
            .service(move_reply_to_inspection)

            .service(css_layout)
            .service(css_theme)
            .service(favicon)
            .service(auth_signup)
            .service(auth_signup)
            .service(auth_signup)
            .app_data(auth.clone())
            .app_data(db.clone())
            .wrap(Logger::default())
            .default_service(web::to(default_handler))
    })
    .bind(("0.0.0.0", 8080))?
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}