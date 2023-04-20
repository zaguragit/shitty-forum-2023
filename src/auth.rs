use std::{collections::HashMap, sync::Mutex, pin::Pin, future::Future};

use actix_web::{cookie, FromRequest, HttpRequest, dev::Payload, ResponseError, http::StatusCode, HttpResponseBuilder, cookie::{Cookie, SameSite}, web::Data};
use chrono::{NaiveDateTime, Local, Duration};
use regex::Regex;
use sha2::{Sha256, Digest};
use rand::distributions::{Alphanumeric, DistString};

use crate::{db::{DB, store::{load_user_auth, store_user_auth}}, data::UserID};

pub struct Auth {
    sessions: HashMap<SessionID, (UserID, NaiveDateTime)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionID(pub String);

pub struct PasswordStore {
    pub salt: String,
    pub hashed: String,
}

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error("Wrong credentials")]
    WrongCredentials,
    #[error("Invalid user name. Only alphanumeric characters, '_' & '-' are allowed")]
    InvalidUserName,
}

#[derive(thiserror::Error, Debug)]
pub enum SignupError {
    #[error("User with such name already exists")]
    AlreadyExists,
    #[error("Invalid user name. Only alphanumeric characters, '_' & '-' are allowed")]
    InvalidUserName,
}

impl Auth {
    pub fn init() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    fn secure_password(password: &str) -> PasswordStore {
        let salt = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
        let mut hasher = Sha256::new();
        hasher.update(password.to_string() + salt.as_str());
        let hashed = hasher.finalize();
        let hashed = String::from_utf8_lossy(hashed.as_slice()).to_string();
        PasswordStore { salt, hashed }
    }

    fn match_password(&self, user_name: &str, password: &str) -> bool {
        let store = load_user_auth(user_name);
        match store {
            Some(store) => {
                let mut hasher = Sha256::new();
                hasher.update(password.to_string() + store.salt.as_str());
                let hashed = hasher.finalize();
                let hashed = String::from_utf8_lossy(hashed.as_slice()).to_string();
                hashed == store.hashed
            },
            None => false,
        }
    }

    fn gen_session_id(&self) -> SessionID {
        let id = SessionID(Alphanumeric.sample_string(&mut rand::thread_rng(), 128));
        if self.sessions.contains_key(&id) {
            self.gen_session_id()
        } else {
            id
        }
    }

    fn create_session(&mut self, user: UserID) -> SessionID {
        let session_id = self.gen_session_id();
        self.sessions.insert(session_id.clone(), (user, Local::now().naive_local()));
        session_id
    }

    pub fn signup(&mut self, user_name: &str, password: &str, db: &mut DB) -> Result<UserID, SignupError> {
        if Regex::new("[a-zA-Z0-9_-]+").unwrap().captures(user_name).is_none() {
            Err(SignupError::InvalidUserName)
        } else if load_user_auth(user_name).is_some() {
            Err(SignupError::AlreadyExists)
        } else {
            let password_store = Self::secure_password(password);
            Ok(db.create_new_user(user_name, &password_store))
        }
    }

    pub fn login(&mut self, user_name: &str, password: &str) -> Result<(UserID, SessionID), LoginError> {
        if Regex::new("[a-zA-Z0-9_-]+").unwrap().captures(user_name).is_none() {
            Err(LoginError::InvalidUserName)
        } else if self.match_password(user_name, password) {
            let password_store = Self::secure_password(password);
            store_user_auth(user_name, &password_store);
            let id = UserID(user_name.to_string());
            Ok((id.clone(), self.create_session(id)))
        } else {
            Err(LoginError::WrongCredentials)
        }
    }

    pub fn logout(&mut self, user: UserSession) {
        self.sessions.remove(&user.session_id);
    }

    pub fn get_user_for_session_id(&mut self, session_id: SessionID) -> Option<(SessionID, &UserID)> {
        match self.sessions.remove(&session_id) {
            Some((user, _)) => {
                let session_id = self.gen_session_id();
                self.sessions.insert(session_id.clone(), (user, Local::now().naive_local()));
                let user = self.sessions.get(&session_id).map(|x| &x.0);
                user.map(|u| (session_id, u))
            },
            None => None,
        }
    }

    pub fn delete_sessions_older_than(&mut self, age: &Duration) {
        self.sessions.retain(|_, (_, last_use)| Local::now().naive_local().signed_duration_since(last_use.clone()).gt(age))
    }
}

pub struct UserSession {
    pub user: UserID,
    pub session_id: SessionID,
}

impl UserSession {
    pub fn keep<'a>(&self, response: &'a mut HttpResponseBuilder) -> &'a mut HttpResponseBuilder {
        response.cookie(build_session_cookie(&self.session_id))
    }
}

pub fn build_session_cookie<'a>(session_id: &'a SessionID) -> Cookie<'a> {
    Cookie::build("session-id", session_id.0.as_str())
        .path("/")
        //.secure(true) <-- only works with https
        .same_site(SameSite::Strict)
        .http_only(true)
        .max_age(cookie::time::Duration::days(30))
        .finish()
}

#[derive(thiserror::Error, Debug)]
pub enum SessionRequestError {
    #[error("No Session")]
    NoSession,
}

impl ResponseError for SessionRequestError {
    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }
}

impl FromRequest for UserSession {
    type Error = SessionRequestError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {        
        let mut auth = req.app_data::<Data<Mutex<Auth>>>().unwrap().lock().unwrap();
        let cookie = req.cookie("session-id")
            .map(|c| c.value().to_string())
            .and_then(|id|
                auth.get_user_for_session_id(SessionID(id)).map(|(new_session_id, user)| {
                    UserSession { user: user.clone(), session_id: new_session_id }
                })
            );
        Box::pin(async move {
            cookie.map_or_else(|| Err(SessionRequestError::NoSession), |x| Ok(x))
        })
    }
}