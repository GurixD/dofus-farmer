use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::Connection;
use dotenvy::dotenv;
use std::env;
use tracing::{trace, trace_span};

pub fn establish_pooled_connection() -> Pool<ConnectionManager<PgConnection>> {
    let span = trace_span!("establishing pooled connection");
    let _guard = span.enter();

    dotenv().expect("Failed to load .env file");

    trace!("Loading database_url");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    trace!("Creating manager");
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    trace!("Creating pool");
    Pool::builder()
        .max_size(1)
        .build(manager)
        .expect("Failed to create pool.")
}

pub fn _establish_connection() -> PgConnection {
    dotenv().expect("Failed to load .env file");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
