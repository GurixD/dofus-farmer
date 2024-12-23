use std::collections::HashMap;

use const_format::formatcp;
use itertools::Itertools;
use reqwest::blocking::Client;
use serde_json::{from_str, from_value};

use crate::{
    data_loader::{data_loader::DataLoader, dofusdb::map_position::MapPosition},
    database::models::{map::Map, sub_area::SubArea},
};

use super::{base_response::BaseResponse, query_params::QueryParams};

pub struct ApiLoader {
    client: Client,
}

impl ApiLoader {
    const BASE_URL: &str = "https://api.beta.dofusdb.fr";
    const MAP_POSITIONS_URL: &str = formatcp!("{}/map-positions", ApiLoader::BASE_URL);
    const LIMIT: u32 = 50;

    pub fn new() -> Self {
        let client = Client::new();
        Self { client }
    }
}

impl DataLoader for ApiLoader {
    fn load_all_sub_areas(
        &self,
    ) -> std::collections::HashMap<
        crate::database::models::sub_area::SubArea,
        Vec<crate::database::models::map::Map>,
    > {
        let mut params = QueryParams::new();
        params.set_param("worldMap", "1");
        // params.set_param("hasPriorityOnWorldmap", "true");
        params.add_param("$and[0][subAreaId][$ne]", "1036");
        params.set_param("$limit", &Self::LIMIT.to_string());
        params.set_param("$skip", "0");
        params.add_param("$select[]", "posX");
        params.add_param("$select[]", "posY");
        params.add_param("$select[]", "id");
        params.add_param("$select[]", "subAreaId");
        params.add_param("$select[]", "hasPriorityOnWorldmap");

        let mut skip = 0;
        let mut total = 10000;

        let mut sub_areas: HashMap<SubArea, Vec<(Map, bool)>> = HashMap::new();

        while skip < total {
            let url = Self::MAP_POSITIONS_URL.to_owned() + "?" + &params.to_query_string();
            println!("{url}");

            let response = self.client.get(url).send().unwrap();

            let text_response = response.text().unwrap();

            let base_response = from_str::<BaseResponse>(&text_response).unwrap();
            skip = base_response.skip + Self::LIMIT;
            total = base_response.total;

            let map_positions = from_value::<Vec<MapPosition>>(base_response.data).unwrap();
            map_positions.iter().for_each(|map_position| {
                // drop other maps with same pos if this one has prio
                if map_position.has_priority {
                    sub_areas.iter_mut().for_each(|(_, maps)| {
                        maps.retain(|map| map.0.x != map_position.x || map.0.y != map_position.y);
                    });
                }

                // don't execute if the map has low prio and other map already exists
                if !(!map_position.has_priority
                    && sub_areas.iter().any(|(_, maps)| {
                        maps.iter()
                            .find(|map| map.0.x == map_position.x && map.0.y == map_position.y)
                            .is_some()
                    }))
                {
                    let sub_area = SubArea::new(map_position.sub_area_id, "test".to_owned(), 0);
                    sub_areas.entry(sub_area).or_default().push((
                        Map::new(
                            map_position.id,
                            None,
                            map_position.x,
                            map_position.y,
                            map_position.sub_area_id,
                        ),
                        map_position.has_priority,
                    ));
                }
            });

            params.set_param("$skip", &skip.to_string());
        }

        sub_areas
            .into_iter()
            .map(|(sub_area, maps)| {
                (
                    sub_area,
                    maps.into_iter().map(|(map, _prio)| map).collect_vec(),
                )
            })
            .collect()
    }
}
