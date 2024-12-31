use std::{env, sync::mpsc};

use data_loader::{data_loader::DataLoader, dofusdb::api_loader::ApiLoader};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use dotenvy::dotenv;

mod data_loader;
mod database;

fn main() {
    let (tx, rx) = mpsc::channel();
    let pool = establish_pooled_connection();

    let loader = ApiLoader::new(pool, tx);

    let test = loader.load_initial_ingredients_quantity();
}

pub fn establish_pooled_connection() -> Pool<ConnectionManager<PgConnection>> {
    dotenv().expect("Failed to load .env file");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .max_size(10)
        .build(manager)
        .expect("Failed to create pool")
}
