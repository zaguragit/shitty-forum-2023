use std::{fs::read_to_string, sync::Mutex};
use actix_web::{get, HttpResponse, web::{Data, Path, Query}};
use serde::Deserialize;
use crate::{db::DB, render::{render_page, render_topic_fav, render_thread, render_user_reply, render_reply, render_thread_fav, render_inspection_reply}, auth::UserSession, data::{UserID, TopicID, ThreadID, Moderatable}};

#[get("/")]
pub async fn page_home(db: Data<Mutex<DB>>, user: Option<UserSession>) -> HttpResponse {
    let db = db.lock().unwrap();
    render_page(&db, user.as_ref(), || {
        read_to_string("assets/page/root.html").unwrap()
    })
}

#[get("/λ/{topic_name}")]
pub async fn page_topic(db: Data<Mutex<DB>>, user: Option<UserSession>, topic_name: Path<String>) -> HttpResponse {
    let db = db.lock().unwrap();
    let topic_id = TopicID(topic_name.into_inner());
    let topic = db.get_topic(&topic_id);
    match topic {
        Some(topic) => render_page(&db, user.as_ref(), || {
            let thread_html = read_to_string("assets/element/thread.html").unwrap();
            let threads: Vec<String> = db.get_sorted_threads(&topic_id).iter().map(|x| render_thread(&db, thread_html.as_str(), x)).collect();
            let html = read_to_string("assets/page/topic.html").unwrap();
            html
                .replace("{{insert-favorite-here}}", render_topic_fav(&db, user.as_ref().map(|x| &x.user), &topic_id).as_str())
                .replace("{{topic-name}}", topic_id.0.as_str())
                .replace("{{create-thread}}", if user.is_some() {
                    read_to_string("assets/element/create-thread.html").unwrap()
                        .replace("{{topic-name}}", topic_id.0.as_str())
                } else { "".to_string() }.as_str())
                .replace("{{about}}", topic.about.as_str())
                .replace("{{threads}}", threads.join("").as_str())
        }),
        None => render_page(&db, user.as_ref(), || {
            let html = read_to_string("assets/page/topic-404.html").unwrap();
            html.replace("{{topic-name}}", topic_id.0.as_str())
        }),
    }
}

#[get("/u/{user_name}")]
pub async fn page_user(db: Data<Mutex<DB>>, current_user: Option<UserSession>, user_name: Path<String>) -> HttpResponse {
    let db = db.lock().unwrap();
    let user_id = UserID(user_name.into_inner());
    let user = db.get_user(&user_id);
    match user {
        Some(user) => render_page(&db, current_user.as_ref(), || {
            let reply_html = read_to_string("assets/element/reply/user-reply.html").unwrap();
            let replies: Vec<String> = db.collect_replies_for_user(
                &user_id,
                |thread_id, thread, reply|
                    render_user_reply(reply_html.as_str(), thread_id, thread, reply)
            );
            let html = read_to_string("assets/page/user.html").unwrap();
            html.replace("{{user-name}}", user_id.0.as_str())
                .replace("{{pronouns}}", user.pronouns.as_ref().map_or_else(|| "".to_string(), |x| x.join("/")).as_str())
                .replace("{{link-to-settings}}", match &current_user {
                    Some(x) if x.user == user_id => "<a href=/settings>Settings</a>",
                    _ => "",
                })
                .replace("{{about}}", user.about.as_str())
                .replace("{{replies}}", replies.join("").as_str())
        }),
        None => render_page(&db, current_user.as_ref(), || {
            let html = read_to_string("assets/page/user-404.html").unwrap();
            html.replace("{{user-name}}", user_id.0.as_str())
        }),
    }
}

#[get("/t/{thread_id}")]
pub async fn page_thread(db: Data<Mutex<DB>>, user: Option<UserSession>, thread_id: Path<String>) -> HttpResponse {
    let db = db.lock().unwrap();
    let thread_id = ThreadID(thread_id.into_inner());
    let thread = db.get_thread(&thread_id);
    match thread {
        Some(thread) => render_page(&db, user.as_ref(), || {
            let reply_html = read_to_string("assets/element/reply/reply.html").unwrap();
            let replies: Vec<String> = thread.replies.iter()
                .filter_map(|x| db.get_reply(x))
                .map(|x| render_reply(&db, reply_html.as_str(), x)).collect();
            let html = read_to_string("assets/page/thread.html").unwrap();
            html
                .replace("{{insert-favorite-here}}", render_thread_fav(&db, user.as_ref().map(|x| &x.user), &thread_id).as_str())
                .replace("{{insert-form-here}}", if user.is_some() {
                    read_to_string("assets/element/reply-form.html").unwrap()
                        .replace("{{thread-id}}", thread_id.0.as_str())
                } else { "".to_string() }.as_str())
                .replace("{{title}}", thread.title.as_str())
                .replace("{{replies}}", replies.join("").as_str())
        }),
        None => render_page(&db, user.as_ref(), || {
            read_to_string("assets/page/thread-404.html").unwrap()
        }),
    }
}

#[get("/λ/{topic}/create-thread")]
pub async fn page_create_thread(db: Data<Mutex<DB>>, user: UserSession, topic: Path<String>) -> HttpResponse {
    let db = db.lock().unwrap();
    render_page(&db, Some(&user), || {
        read_to_string("assets/page/create-thread.html").unwrap()
            .replace("{{topic-name}}", topic.as_str())
    })
}

#[derive(Debug, Deserialize)]
pub struct Error {
    error: Option<String>,
}

#[get("/settings")]
pub async fn page_settings(db: Data<Mutex<DB>>, user: UserSession, query: Query<Error>) -> HttpResponse {
    let db = db.lock().unwrap();
    render_page(&db, Some(&user), || {
        let user = db.get_user(&user.user).unwrap();
        read_to_string("assets/page/settings.html").unwrap()
            .replace("{{insert-error-here}}", query.0.error.unwrap_or_else(|| "".to_string()).as_str())
            .replace("{{pronouns}}", user.pronouns.as_ref().map_or_else(|| "".to_string(), |x| x.join("/")).as_str())
            .replace("{{about}}", html_escape::encode_text(user.about.as_str()).as_ref())
    })
}

#[get("/login")]
pub async fn page_login(db: Data<Mutex<DB>>, user: Option<UserSession>, query: Query<Error>) -> HttpResponse {
    let db = db.lock().unwrap();
    render_page(&db, user.as_ref(), || {
        read_to_string("assets/page/login.html").unwrap()
            .replace("{{insert-error-here}}", query.0.error.unwrap_or_else(|| "".to_string()).as_str())
    })
}

#[get("/signup")]
pub async fn page_signup(db: Data<Mutex<DB>>, user: Option<UserSession>, query: Query<Error>) -> HttpResponse {
    let db = db.lock().unwrap();
    render_page(&db, user.as_ref(), || {
        read_to_string("assets/page/signup.html").unwrap()
            .replace("{{insert-error-here}}", query.0.error.unwrap_or_else(|| "".to_string()).as_str())
    })
}

#[derive(Debug, Deserialize)]
pub struct Search {
    q: String,
}

#[get("/search")]
pub async fn page_search(db: Data<Mutex<DB>>, user: Option<UserSession>, query: Query<Search>) -> HttpResponse {
    let thread_html = read_to_string("assets/element/thread.html").unwrap();
    let db = db.lock().unwrap();
    let topics = db.search_topics(query.0.q.as_str());
    let threads = db.search_threads(query.0.q.as_str());
    render_page(&db, user.as_ref(), || {
        read_to_string("assets/page/search.html").unwrap()
            .replace("{{query}}", query.0.q.as_str())
            .replace("{{topics}}", topics.into_iter().map(|x| format!("<li><a href=\"/λ/{}\">{}</a></li>", x.0, x.0)).collect::<Vec<_>>().join("").as_str())
            .replace("{{threads}}", threads.into_iter().map(|x| render_thread(&db, thread_html.as_str(), &x)).collect::<Vec<_>>().join("").as_str())
    })
}

#[get("/inspection")]
pub async fn page_inspection(db: Data<Mutex<DB>>, user: UserSession) -> HttpResponse {
    let reply_html = read_to_string("assets/element/reply/inspection-reply.html").unwrap();
    let db = db.lock().unwrap();
    let inspection = db.get_inspection();
    render_page(&db, Some(&user), || {
        read_to_string("assets/page/inspection.html").unwrap()
            .replace("{{items}}", inspection.iter().map(|(_, item)| {
                match &item.thing {
                    Moderatable::User(user) => "user",
                    Moderatable::Topic(owner, topic) => "topic",
                    Moderatable::Thread(user, thread, topic_id) => "thread",
                    Moderatable::Reply(user, reply, thread_id) => "reply",
                }
            }).collect::<Vec<_>>().join("").as_str())
    })

    // render_inspection_reply(&db, reply_html.as_str(), thread_id, thread, reply).as_str()
}