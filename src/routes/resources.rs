use std::io::Result;
use actix_files::NamedFile;
use actix_web::get;

#[get("/layout.css")]
pub async fn css_layout() -> Result<NamedFile> {
    NamedFile::open("assets/layout.css")
}

#[get("/theme.css")]
pub async fn css_theme() -> Result<NamedFile> {
    NamedFile::open("assets/theme.css")
}

#[get("/favicon.png")]
pub async fn favicon() -> Result<NamedFile> {
    NamedFile::open("assets/favicon.png")
}