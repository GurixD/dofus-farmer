use std::{collections::HashMap, sync::mpsc::Sender};

use const_format::formatcp;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use futures::executor::block_on;
use http_cache::{CacheMode, HttpCache, HttpCacheOptions};
use http_cache_reqwest::Cache;
use itertools::Itertools;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde_json::{from_str, from_value};

use crate::{
    data_loader::{
        data_loader::DataLoader,
        dofusdb::{item_dto::ItemDto, map_position_dto::MapPositionDto},
    },
    database::models::{
        item::{Item, ItemList},
        map::Map,
        sub_area::SubArea,
        user_ingredient::UserIngredient,
        user_item::UserItem,
    },
};

use super::{
    base_response::BaseResponse, my_cache_manager::MyCacheManager, query_params::QueryParams,
};

pub struct ApiLoader {
    client: ClientWithMiddleware,
    pool: Pool<ConnectionManager<PgConnection>>,
    item_tx: Sender<(Item, i16)>,
}

impl ApiLoader {
    const BASE_URL: &str = "https://api.beta.dofusdb.fr";
    const MAP_POSITIONS_URL: &str = formatcp!("{}/map-positions", ApiLoader::BASE_URL);
    const ITEMS_URL: &str = formatcp!("{}/items", ApiLoader::BASE_URL);
    const LIMIT: u32 = 50;

    pub fn new(pool: Pool<ConnectionManager<PgConnection>>, item_tx: Sender<(Item, i16)>) -> Self {
        let mut manager = MyCacheManager {};
        let client = ClientBuilder::new(Client::new())
            .with(Cache(HttpCache {
                mode: CacheMode::ForceCache,
                manager: manager,
                options: HttpCacheOptions::default(),
            }))
            .build();
        Self {
            client,
            pool,
            item_tx,
        }
    }
}

impl DataLoader for ApiLoader {
    #[tokio::main]
    async fn load_all_sub_areas(&self) -> HashMap<SubArea, Vec<Map>> {
        let mut query_params = QueryParams::new();
        query_params.set_param("worldMap", "1");
        // params.set_param("hasPriorityOnWorldmap", "true");
        query_params.add_param("$and[0][subAreaId][$ne]", "1036");
        query_params.set_param("$limit", &Self::LIMIT.to_string());
        query_params.set_param("$skip", "0");
        query_params.add_param("$select[]", "posX");
        query_params.add_param("$select[]", "posY");
        query_params.add_param("$select[]", "id");
        query_params.add_param("$select[]", "subAreaId");
        query_params.add_param("$select[]", "hasPriorityOnWorldmap");

        let mut skip = 0;
        let mut total = 10000;

        let mut sub_areas: HashMap<SubArea, Vec<(Map, bool)>> = HashMap::new();

        while skip < total {
            let url = Self::MAP_POSITIONS_URL.to_string() + "?" + &query_params.to_query_string();
            println!("{url}");

            let response = self
                .client
                .get(url)
                // .with_extension(CacheMode::OnlyIfCached)
                .send()
                .await
                .unwrap();

            println!("{:#?}", response.headers());

            let url = Self::MAP_POSITIONS_URL.to_string() + "?" + &query_params.to_query_string();
            println!("{url}");

            let response = self
                .client
                .get(url)
                // .with_extension(CacheMode::OnlyIfCached)
                .send()
                .await
                .unwrap();

            println!("{:#?}", response.headers());

            let text_response = response.text().await.unwrap();

            let base_response = from_str::<BaseResponse>(&text_response).unwrap();
            skip = base_response.skip + Self::LIMIT;
            total = base_response.total;

            let map_positions = from_value::<Vec<MapPositionDto>>(base_response.data).unwrap();
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
                    let sub_area = SubArea::new(map_position.sub_area_id, "test".to_string(), 0);
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

            query_params.set_param("$skip", &skip.to_string());
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

    #[tokio::main]
    async fn load_initial_ingredients_quantity(&self) -> ItemList {
        use crate::database::schema::{user_ingredients, user_items};
        use diesel::prelude::*;

        let mut connection = self.pool.get().unwrap();

        let mut query_params = QueryParams::new();
        query_params.set_param("$limit", "1");
        // query_params.add_param("$select[]", "id");
        // query_params.add_param("$select[]", "name");
        // query_params.add_param("$select[]", "type");
        // query_params.add_param("$select[]", "iconId");

        let user_items = user_items::table
            .load::<UserItem>(&mut connection)
            .unwrap()
            .into_iter()
            .map(|user_item| {
                let mut query_params = query_params.clone();
                async move {
                    query_params.set_param("id", &user_item.item_id.to_string());
                    let url = Self::ITEMS_URL.to_string() + "?" + &query_params.to_query_string();
                    println!("{url}");
                    let response = self.client.get(url).send().await.unwrap();
                    let json = from_str::<BaseResponse>(&response.text().await.unwrap()).unwrap();
                    let item = from_value::<Vec<ItemDto>>(json.data)
                        .unwrap()
                        .pop()
                        .unwrap();
                    (
                        user_item,
                        Item::new(item.id, item.name, item.category, item.icon),
                    )
                }
            })
            .map(|future| block_on(future))
            .collect_vec();

        user_items.into_iter().for_each(|(user_item, item)| {
            self.item_tx
                .send((item, user_item.quantity.into()))
                .unwrap();
        });

        ItemList::with_items(
            user_ingredients::table
                .load::<UserIngredient>(&mut connection)
                .unwrap()
                .into_iter()
                .map(|user_ingredient| {
                    let mut query_params = query_params.clone();
                    async move {
                        query_params.set_param("id", &user_ingredient.item_id.to_string());
                        let url = Self::MAP_POSITIONS_URL.to_string()
                            + "?"
                            + &query_params.to_query_string();
                        let response = self.client.get(url).send().await.unwrap();
                        let json =
                            from_str::<BaseResponse>(&response.text().await.unwrap()).unwrap();
                        let item = from_value::<ItemDto>(json.data).unwrap();
                        (
                            Item::new(item.id, item.name, item.category, item.icon),
                            user_ingredient.quantity,
                        )
                    }
                })
                .map(|future| block_on(future))
                .collect(),
        )
    }
}
