use actix_web::web::{Data, Json, Path};
use actix_web::{web, HttpResponse};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use diesel::result::Error;
use diesel::{ExpressionMethods, Insertable, Queryable, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rand::Rng;

use crate::constants::{APPLICATION_JSON, CONNECTION_POOL_ERROR};
use crate::response::Response;
use crate::response::JsonVal;
use crate::{DBPool, DBPooledConnection};
use crate::schema::smss;
use diesel::query_dsl::methods::{FilterDsl, LimitDsl, OrderDsl};
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize)]
pub struct Sms {
    pub id: String,
    pub tel: String,
    pub code: String,
    pub created_at: DateTime<Utc>
}

impl Sms {
    pub fn new(tel: String) -> Self {
        let mut rng = rand::thread_rng();
        let number: u32 = rng.gen_range(0, 999999);
        let code = format!("{:06}", number);
        println!("{}", code);

        Self {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            tel,
            code
        }
    }
    pub fn to_sms_db(&self) -> SmsDB {
        SmsDB {
            id: Uuid::new_v4(),
            tel: self.tel.clone(), 
            code: self.code.clone(), 
            created_at: self.created_at.naive_utc(),
        }
    }
}

#[table_name = "smss"]
#[derive(Queryable, Insertable)]
pub struct SmsDB {
    pub id: Uuid,
    pub tel: String,
    pub code: String,
    pub created_at: NaiveDateTime
}

impl SmsDB {
    fn to_sms(&self) -> Sms {
        Sms {
            id: self.id.to_string(),
            created_at: Utc.from_utc_datetime(&self.created_at),
            tel: self.tel.clone(),
            code: self.code,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SmsRequest {
    pub tel: Option<String>,
}

impl SmsRequest {
    pub fn to_sms(&self) -> Option<Sms> {
        match &self.tel {
            Some(tel) => Some(Sms::new(tel.to_string())),
            None => None,
        }
    }
}

#[post("/sms")]
pub async fn create(sms_req: Json<SmsRequest>, pool: Data<DBPool>) -> HttpResponse {
    let conn = pool.get().expect(CONNECTION_POOL_ERROR);
    let sms = web::block(move || create_sms(sms_req.to_sms().unwrap(), &conn)).await;

    match sms {
        Ok(sms) => HttpResponse::Created()
            .content_type(APPLICATION_JSON)
            .json(sms),
        _ => HttpResponse::NoContent().await.unwrap(),
    }
}

#[get("/sms/{id}/{code}")]
pub async fn get(path: Path<(String,)>, pool: Data<DBPool>) -> HttpResponse {
    let conn = pool.get().expect(CONNECTION_POOL_ERROR);
    let sms = web::block(move || find_sms(Uuid::from_str(path.0.as_str()).unwrap(), &conn)).await;

    match sms {
        Ok(sms) => {
            HttpResponse::Ok()
            .content_type(APPLICATION_JSON)
            .json(JsonVal{success: path.1 == sms.code})
        }
        _ => HttpResponse::NoContent()
            .content_type(APPLICATION_JSON)
            .await
            .unwrap(),
    }
}

fn create_sms(sms: Sms, conn: &DBPooledConnection) -> Result<Sms, Error> {
    use crate::schema::smss::dsl::*;

    let sms_db = sms.to_sms_db();
    let _ = diesel::insert_into(smss).values(&sms_db).execute(conn);

    Ok(sms_db.to_sms())
}

fn find_sms(_id: Uuid, conn: &DBPooledConnection) -> Result<Sms, Error> {
    use crate::schema::smss::dsl::*;

    let res = smss.filter(id.eq(_id)).load::<SmsDB>(conn);
    match res {
        Ok(sms_db) => match sms_db.first() {
            Some(sms_db) => Ok(sms_db.to_sms()),
            _ => Err(Error::NotFound),
        },
        Err(err) => Err(err),
    }
}


