use std::collections::HashMap;

use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use lombok::AllArgsConstructor;

use crate::database::models::{map::Map, sub_area::SubArea};

use super::data_loader::DataLoader;

#[derive(AllArgsConstructor)]
pub struct DatabaseLoader {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl DataLoader for DatabaseLoader {
    fn load_all_sub_areas(
        &self,
    ) -> std::collections::HashMap<
        crate::database::models::sub_area::SubArea,
        Vec<crate::database::models::map::Map>,
    > {
        use crate::database::schema::sub_areas;
        use diesel::prelude::*;

        let mut connection = self.pool.get().unwrap();

        let sub_areas = sub_areas::table
            .select(SubArea::as_select())
            .load(&mut connection)
            .unwrap();

        let maps = Map::belonging_to(&sub_areas)
            .select(Map::as_select())
            .load(&mut connection)
            .unwrap();

        let mut maps_per_sub_area: HashMap<SubArea, Vec<Map>> = maps
            .grouped_by(&sub_areas)
            .into_iter()
            .zip(sub_areas)
            .map(|(maps, sub_area)| (sub_area, maps))
            .collect();

        maps_per_sub_area.retain(|_, vec| !vec.is_empty());

        maps_per_sub_area
    }
}
