#[macro_use]
extern crate actix_web;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate rand;


use std::{env, io};
use dotenv::dotenv;
use actix_web::{middleware, App, HttpServer};
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::{Pool, PooledConnection};

mod constants;
mod response;
mod sms;
mod schema;
mod user;
mod family;
mod user_family;
pub type DBPool = Pool<ConnectionManager<PgConnection>>;
pub type DBPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

//curl -X POST -d '{"tel": "064857458"}' -H "Content-type: application/json" http://localhost:9090/sms
//curl http://localhost:9090/sms/813bd9f8-46f3-4705-9e9c-4f6819ded89f/9094641

#[actix_rt::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    // set up database connection pool
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    HttpServer::new(move || {
        App::new()
            // Set up DB pool to be used with web::Data<Pool> extractor
            .data(pool.clone())
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
            // register HTTP requests handlers
            .service(sms::services::get)
            .service(sms::services::create)
            .service(user::services::get)
            .service(user::services::create)
            .service(family::services::get)
            .service(family::services::create)
    })
    .bind("0.0.0.0:9090")?
    .run()
    .await
}