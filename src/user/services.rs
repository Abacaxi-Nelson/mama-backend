use actix_web::web::{Data, Json};
use actix_web::{web, HttpResponse};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use diesel::result::Error;
use diesel::{ExpressionMethods, Insertable, Queryable, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::constants::{APPLICATION_JSON, CONNECTION_POOL_ERROR};
use crate::{DBPool, DBPooledConnection};
use crate::schema::users;
use diesel::query_dsl::methods::{FilterDsl};
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub tel: String,
    pub nom: String,
    pub email: String,
    pub created_at: DateTime<Utc>
}

impl User {
    pub fn new(tel: String, nom: String, email: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            nom,
            email,
            tel
        }
    }
    pub fn to_user_db(&self) -> UserDB {
        UserDB {
            id: Uuid::new_v4(),
            tel: self.tel.clone(), 
            nom: self.nom.clone(), 
            email: self.email.clone(), 
            created_at: self.created_at.naive_utc(),
        }
    }
}

#[table_name = "users"]
#[derive(Queryable, Insertable)]
pub struct UserDB {
    pub id: Uuid,
    pub tel: String,
    pub nom: String,
    pub email: String,
    pub created_at: NaiveDateTime
}

impl UserDB {
    fn to_user(&self) -> User {
        User {
            id: self.id.to_string(),
            created_at: Utc.from_utc_datetime(&self.created_at),
            tel: self.tel.clone(),
            nom: self.nom.clone(),
            email: self.email.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserRequest {
    pub tel: Option<String>,
    pub nom: Option<String>,
    pub email: Option<String>,
}

impl UserRequest {
    pub fn to_user(&self) -> Option<User> {
        match &self { 
            UserRequest { 
                tel: Some(tel),
                nom: Some(nom), 
                email: Some(email)
            }  =>  Some(User::new(tel.to_string(), nom.to_string(), email.to_string())),
            _ => None
        }   
        /*match &self{
            Some(tel, nom, email) => Some(User::new(tel.to_string(), nom.to_string(), email.to_string())),
            None => None,
        }
        */

    }
}

#[post("/user")]
pub async fn create(user_req: Json<UserRequest>, pool: Data<DBPool>) -> HttpResponse {
    let conn = pool.get().expect(CONNECTION_POOL_ERROR);
    let user = web::block(move || create_user(user_req.to_user().unwrap(), &conn)).await;

    match user {
        Ok(user) => HttpResponse::Created()
            .content_type(APPLICATION_JSON)
            .json(user),
        _ => HttpResponse::NoContent().await.unwrap(),
    }
}

#[get("/user/{id}")]
pub async fn get(web::Path(id): web::Path<String>, pool: Data<DBPool>) -> HttpResponse {
    let conn = pool.get().expect(CONNECTION_POOL_ERROR);
    let user = web::block(move || 
        find_user(Uuid::from_str(id.as_str()).unwrap(), &conn)
    ).await;

    match user {
        Ok(user) => {
            HttpResponse::Ok()
            .content_type(APPLICATION_JSON)
            .json(user)
        }
        _ => HttpResponse::NoContent()
            .content_type(APPLICATION_JSON)
            .await
            .unwrap(),
    }
}

fn find_user(_id: Uuid, conn: &DBPooledConnection) -> Result<User, Error> {
    use crate::schema::users::dsl::*;

    let res = users.filter(id.eq(_id)).load::<UserDB>(conn);
    match res {
        Ok(user_db) => match user_db.first() {
            Some(user_db) => Ok(user_db.to_user()),
            _ => Err(Error::NotFound),
        },
        Err(err) => Err(err),
    }
}

fn create_user(user: User, conn: &DBPooledConnection) -> Result<User, Error> {
    use crate::schema::users::dsl::*;

    let user_db = user.to_user_db();
    let _ = diesel::insert_into(users).values(&user_db).execute(conn);

    Ok(user_db.to_user())
}




