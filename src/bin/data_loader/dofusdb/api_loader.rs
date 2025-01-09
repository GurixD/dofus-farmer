use std::{
    collections::{HashMap, HashSet},
    sync::mpsc::Sender,
};

use const_format::formatcp;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use egui::{ColorImage, Context};
use futures::executor::block_on;
use itertools::Itertools;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::de::DeserializeOwned;
use serde_json::{from_str, from_value};

use crate::{
    data_loader::{data_loader::DataLoader, dofusdb::dto::item_dto::ItemDto},
    database::models::{
        drop::Drop,
        item::{Item, ItemList},
        map::Map,
        monster::Monster,
        monster_sub_area::MonsterSubArea,
        sub_area::SubArea,
        user_ingredient::UserIngredient,
        user_item::UserItem,
    },
    gui::{image::Image, main_window::Ingredients},
};

use super::{
    base_response::BaseResponse,
    dofusdb_cache::DofusDbCache,
    dto::{map_position_dto::MapPositionDto, monster_dto::MonsterDto},
    query_params::QueryParams,
};

pub struct ApiLoader {
    client: ClientWithMiddleware,
    pool: Pool<ConnectionManager<PgConnection>>,
    item_tx: Sender<(Item, i16)>,
}

impl ApiLoader {
    pub const BASE_URL: &str = "https://api.beta.dofusdb.fr";
    pub const BASE_IMG_URL: &str = formatcp!("{}/img", ApiLoader::BASE_URL);
    const MAP_POSITIONS_URL: &str = formatcp!("{}/map-positions", ApiLoader::BASE_URL);
    const ITEMS_URL: &str = formatcp!("{}/items", ApiLoader::BASE_URL);
    const MONSTERS_URL: &str = formatcp!("{}/monsters", ApiLoader::BASE_URL);
    const WORLD_MAPS_URL: &str = formatcp!("{}/worlds", ApiLoader::BASE_IMG_URL);
    const ITEMS_IMG_URL: &str = formatcp!("{}/items", ApiLoader::BASE_IMG_URL);
    const MONSTERS_IMG_URL: &str = formatcp!("{}/monsters", ApiLoader::BASE_IMG_URL);
    const DEFAULT_WORLD: u32 = 1;
    const LIMIT: u32 = 50;
}

impl DataLoader for ApiLoader {
    fn new(pool: Pool<ConnectionManager<PgConnection>>, item_tx: Sender<(Item, i16)>) -> Self {
        let client = ClientBuilder::new(Client::new())
            .with(DofusDbCache::default())
            .build();
        Self {
            client,
            pool,
            item_tx,
        }
    }

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
            let response = self.client.get(url).send().await.unwrap();
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

        let user_items = user_items::table
            .load::<UserItem>(&mut connection)
            .unwrap()
            .into_iter()
            .map(|user_item| {
                let mut query_params = query_params.clone();
                async move {
                    query_params.set_param("id", &user_item.item_id.to_string());
                    let url = Self::ITEMS_URL.to_string() + "?" + &query_params.to_query_string();
                    let response = self.client.get(url).send().await.unwrap();
                    let json = from_str::<BaseResponse>(&response.text().await.unwrap()).unwrap();
                    let item = from_value::<Vec<ItemDto>>(json.data)
                        .unwrap()
                        .pop()
                        .unwrap();
                    (
                        user_item,
                        Item::new(item.id, item.name, item.category, item.icon_id),
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
                            Item::new(item.id, item.name, item.category, item.icon_id),
                            user_ingredient.quantity,
                        )
                    }
                })
                .map(|future| block_on(future))
                .collect(),
        )
    }
    #[tokio::main]
    async fn item_has_recipe(&self, item: &Item) -> bool {
        Self::load_item(&self.client, item.id).await.has_recipe
    }

    fn get_calculated_inventory(&self, inventory: &ItemList) -> ItemList {
        let mut connection = self.pool.get().unwrap();
        let mut calculated_inventory = ItemList::new();
        inventory.iter().for_each(|(ingredient, quantity)| {
            calculated_inventory.add_item(ingredient, quantity);
            if self.item_has_recipe(ingredient) {
                let (base_ingredients, steps) =
                    ingredient.get_full_recipe(quantity, &mut connection);
                calculated_inventory.add_items(&base_ingredients);
                steps.iter().for_each(|step_list| {
                    calculated_inventory.add_items(step_list);
                });
            }
        });

        calculated_inventory
    }

    fn map_from_ui_and_index(
        &self,
        ctx: Context,
        index: u16,
        tx: Sender<(Image, u16, usize)>,
        zoom: f32,
        zoom_index: usize,
    ) {
        let client = self.client.clone();
        tokio::spawn(async move {
            let color_image = Self::load_map_image(client, zoom, index).await;
            let handle = ctx.load_texture(index.to_string(), color_image, Default::default());
            let image = Image::new(handle, true);
            tx.send((image, index, zoom_index)).unwrap();
            ctx.request_repaint();
        });
    }

    fn item_from_id(&self, tx: Sender<(Item, Image)>, ctx: Context, item: Item) {
        let client = self.client.clone();
        tokio::spawn(async move {
            let color_image = Self::load_item_image(client, item.id).await;
            let handle = ctx.load_texture(item.name.clone(), color_image, Default::default());
            let image = Image::new(handle, true);
            tx.send((item, image)).unwrap();
            ctx.request_repaint();
        });
    }

    fn load_monster_image(&self, tx: Sender<(Monster, Image)>, ctx: Context, monster: Monster) {
        let client = self.client.clone();
        tokio::spawn(async move {
            let color_image = Self::load_monster_image(client, monster.id).await;
            let handle = ctx.load_texture(monster.name.clone(), color_image, Default::default());
            let image = Image::new(handle, true);
            tx.send((monster, image)).unwrap();
            ctx.request_repaint();
        });
    }

    fn load_recipe_of_item(&self, tx: Sender<Ingredients>, item: Item, quantity: i16) {
        let pool = self.pool.clone();
        tokio::spawn(async move {
            use crate::database::schema::*;
            use diesel::prelude::*;

            let mut connection = pool.get().unwrap();

            let mut result_hash_map = HashMap::new();

            let (base_ingredients, ingredients_steps) = item.get_full_recipe(&1, &mut connection);

            base_ingredients.iter().for_each(|(ingredient, quantity)| {
                let result: Vec<(SubArea, MonsterSubArea, Monster, Drop, Item)> = sub_areas::table
                    .inner_join(
                        monsters_sub_areas::table
                            .on(sub_areas::id.eq(monsters_sub_areas::sub_area_id)),
                    )
                    .inner_join(monsters::table.on(monsters_sub_areas::monster_id.eq(monsters::id)))
                    .inner_join(drops::table.on(monsters::id.eq(drops::monster_id)))
                    .inner_join(items::table.on(drops::item_id.eq(items::id)))
                    .filter(items::id.eq(ingredient.id))
                    .load(&mut connection)
                    .unwrap();

                result_hash_map.insert(ingredient.clone(), (*quantity, Default::default()));
                let mut sub_areas_for_monsters: HashMap<Monster, HashSet<SubArea>> = HashMap::new();

                result.into_iter().for_each(|(sub_area, _, monster, _, _)| {
                    sub_areas_for_monsters
                        .entry(monster)
                        .and_modify(|sub_areas| {
                            sub_areas.insert(sub_area.clone());
                        })
                        .or_insert_with(|| {
                            let mut sub_areas = HashSet::new();
                            sub_areas.insert(sub_area);
                            sub_areas
                        });
                });

                result_hash_map
                    .entry((*ingredient).clone())
                    .and_modify(|(_, monsters)| {
                        *monsters = sub_areas_for_monsters;
                    });
            });

            tx.send((item, quantity, result_hash_map, ingredients_steps))
                .unwrap();
        });
    }
}

impl ApiLoader {
    pub async fn load_single<T: DeserializeOwned>(
        client: &ClientWithMiddleware,
        url: &str,
        id: i32,
    ) -> T {
        let mut query_params = QueryParams::new();
        query_params.set_param("$limit", "1");
        query_params.set_param("id", &id.to_string());

        let url = url.to_string() + "?" + &query_params.to_query_string();
        let response = client.get(url).send().await.unwrap();
        let json = from_str::<BaseResponse>(&response.text().await.unwrap()).unwrap();
        from_value::<Vec<T>>(json.data).unwrap().pop().unwrap()
    }

    pub async fn load_item(client: &ClientWithMiddleware, id: i32) -> ItemDto {
        Self::load_single(&client, Self::ITEMS_URL, id).await
    }

    pub async fn load_monster(client: &ClientWithMiddleware, id: i32) -> MonsterDto {
        Self::load_single(&client, Self::MONSTERS_URL, id).await
    }

    pub async fn load_image(url: &str, client: &ClientWithMiddleware) -> ColorImage {
        let response = client.get(url).send().await.unwrap();
        let data = response.bytes().await.unwrap();
        let image = image::load_from_memory(&data).unwrap();
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();
        ColorImage::from_rgba_unmultiplied(size, pixels.as_slice())
    }

    pub async fn load_map_image(client: ClientWithMiddleware, zoom: f32, index: u16) -> ColorImage {
        let url = &format!(
            "{}/{}/{}/{}.jpg",
            Self::WORLD_MAPS_URL,
            Self::DEFAULT_WORLD,
            zoom,
            index + 1
        );
        Self::load_image(url, &client).await
    }

    pub async fn load_item_image(client: ClientWithMiddleware, id: i32) -> ColorImage {
        let item = Self::load_item(&client, id).await;
        let url = &format!("{}/{}.png", Self::ITEMS_IMG_URL, item.icon_id);
        Self::load_image(url, &client).await
    }

    pub async fn load_monster_image(client: ClientWithMiddleware, id: i32) -> ColorImage {
        let monster = Self::load_monster(&client, id).await;
        let url = &format!("{}/{}.png", Self::MONSTERS_IMG_URL, monster.gfx_id);
        Self::load_image(url, &client).await
    }
}
