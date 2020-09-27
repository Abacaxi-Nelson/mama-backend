use actix_web::web::{Data, Json};
use actix_web::{web, HttpResponse};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use diesel::result::Error;
use diesel::{ExpressionMethods, Insertable, Queryable, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::constants::{APPLICATION_JSON, CONNECTION_POOL_ERROR};
use crate::{DBPool, DBPooledConnection};
use crate::schema::families;
use diesel::query_dsl::methods::{FilterDsl};
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize)]
pub struct Family {
    pub id: String,
    pub nom: String,
    pub created_at: DateTime<Utc>
}

impl Family {
    pub fn new(nom: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            nom,
        }
    }
    pub fn to_family_db(&self) -> FamilyDB {
        FamilyDB {
            id: Uuid::new_v4(),
            nom: self.nom.clone(), 
            created_at: self.created_at.naive_utc(),
        }
    }
}

#[table_name = "families"]
#[derive(Queryable, Insertable)]
pub struct FamilyDB {
    pub id: Uuid,
    pub nom: String,
    pub created_at: NaiveDateTime
}

impl FamilyDB {
    fn to_family(&self) -> Family {
        Family {
            id: self.id.to_string(),
            created_at: Utc.from_utc_datetime(&self.created_at),
            nom: self.nom.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FamilyRequest {
    pub nom: Option<String>,
}

impl FamilyRequest {
    pub fn to_family(&self) -> Option<Family> {
        match &self.nom {
            Some(nom) => Some(Family::new(nom.to_string())),
            None => None,
        }
    }
}

#[post("/family")]
pub async fn create(family_req: Json<FamilyRequest>, pool: Data<DBPool>) -> HttpResponse {
    let conn = pool.get().expect(CONNECTION_POOL_ERROR);
    let family = web::block(move || create_family(family_req.to_family().unwrap(), &conn)).await;

    match family {
        Ok(family) => HttpResponse::Created()
            .content_type(APPLICATION_JSON)
            .json(family),
        _ => HttpResponse::NoContent().await.unwrap(),
    }
}

#[get("/family/{id}")]
pub async fn get(web::Path(id): web::Path<String>, pool: Data<DBPool>) -> HttpResponse {
    let conn = pool.get().expect(CONNECTION_POOL_ERROR);
    let family = web::block(move || 
        find_family(Uuid::from_str(id.as_str()).unwrap(), &conn)
    ).await;

    match family {
        Ok(family) => {
            HttpResponse::Ok()
            .content_type(APPLICATION_JSON)
            .json(family)
        }
        _ => HttpResponse::NoContent()
            .content_type(APPLICATION_JSON)
            .await
            .unwrap(),
    }
}

fn find_family(_id: Uuid, conn: &DBPooledConnection) -> Result<Family, Error> {
    use crate::schema::families::dsl::*;

    let res = families.filter(id.eq(_id)).load::<FamilyDB>(conn);
    match res {
        Ok(family_db) => match family_db.first() {
            Some(family_db) => Ok(family_db.to_family()),
            _ => Err(Error::NotFound),
        },
        Err(err) => Err(err),
    }
}

fn create_family(family: Family, conn: &DBPooledConnection) -> Result<Family, Error> {
    use crate::schema::families::dsl::*;

    let family_db = family.to_family_db();
    let _ = diesel::insert_into(families).values(&family_db).execute(conn);

    Ok(family_db.to_family())
}




