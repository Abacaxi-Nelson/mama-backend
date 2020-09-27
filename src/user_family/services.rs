use actix_web::web::{Data, Json};
use actix_web::{web, HttpResponse};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use diesel::result::Error;
use diesel::{ Insertable, Queryable, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::constants::{APPLICATION_JSON, CONNECTION_POOL_ERROR};
use crate::{DBPool, DBPooledConnection};
use crate::schema::users_families;
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize)]
pub struct UserFamily {
    pub id: String,
    pub user_id: Uuid,
    pub family_id: Uuid,
    pub role: String,
    pub created_at: DateTime<Utc>
}

impl UserFamily {
    pub fn new(user_id: String, family_id: String, role: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            user_id: Uuid::from_str(user_id.as_str()).unwrap(),
            family_id: Uuid::from_str(family_id.as_str()).unwrap(),
            role
        }
    }
    pub fn to_user_family_db(&self) -> UserFamilyDB {
        UserFamilyDB {
            id: Uuid::new_v4(),
            user_id: self.user_id.clone(),
            family_id: self.family_id.clone(),
            role: self.role.clone(),
            created_at: self.created_at.naive_utc(),
        }
    }
}

#[table_name = "users_families"]
#[derive(Queryable, Insertable, Identifiable, Debug)]
pub struct UserFamilyDB {
    pub id: Uuid,
    pub user_id: Uuid,
    pub family_id: Uuid,
    pub role: String,
    pub created_at: NaiveDateTime
}

impl UserFamilyDB {
    fn to_user_family(&self) -> UserFamily {
        UserFamily {
            id: self.id.to_string(),
            created_at: Utc.from_utc_datetime(&self.created_at),
            user_id: self.user_id,
            family_id: self.family_id,
            role: self.role.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserFamilyRequest {
    pub user_id: Option<String>,
    pub family_id: Option<String>,
    pub role: Option<String>,
}

impl UserFamilyRequest {
    pub fn to_user_family(&self) -> Option<UserFamily> {
        match &self { 
            UserFamilyRequest { 
                user_id: Some(user_id),
                family_id: Some(family_id), 
                role: Some(role)
            }  =>  Some(UserFamily::new(user_id.to_string(), family_id.to_string(), role.to_string())),
            _ => None
        }  
    }
}

#[post("/family")]
pub async fn create(user_family_req: Json<UserFamilyRequest>, pool: Data<DBPool>) -> HttpResponse {
    let conn = pool.get().expect(CONNECTION_POOL_ERROR);
    let user_family = web::block(move || create_user_family(user_family_req.to_user_family().unwrap(), &conn)).await;

    match user_family {
        Ok(user_family) => HttpResponse::Created()
            .content_type(APPLICATION_JSON)
            .json(user_family),
        _ => HttpResponse::NoContent().await.unwrap(),
    }
}

fn create_user_family(user_family: UserFamily, conn: &DBPooledConnection) -> Result<UserFamily, Error> {
    use crate::schema::users_families::dsl::*;

    let user_family_db = user_family.to_user_family_db();
    let _ = diesel::insert_into(users_families).values(&user_family_db).execute(conn);

    Ok(user_family_db.to_user_family())
}




