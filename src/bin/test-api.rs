use data_loader::{data_loader::DataLoader, dofusdb::api_loader::ApiLoader};

mod data_loader;
mod database;

fn main() {
    let loader = ApiLoader::new();

    let test = loader.load_all_sub_areas();
}
