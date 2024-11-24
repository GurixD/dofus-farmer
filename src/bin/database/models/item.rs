use std::{
    collections::BTreeMap,
    hash::Hash,
    ops::{Deref, DerefMut},
};

use diesel::prelude::*;
use lombok::AllArgsConstructor;

use super::recipe::Recipe;

#[derive(Queryable, Selectable, Insertable, AllArgsConstructor, Debug, Clone)]
#[diesel(table_name = crate::database::schema::items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Item {
    pub id: i32,
    pub name: String,
    pub category: i16,
    pub image_id: i32,
}

impl Hash for Item {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Item {}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl Item {
    pub fn has_recipe(&self, connection: &mut PgConnection) -> bool {
        use crate::database::schema::recipes::dsl::*;
        use diesel::dsl::*;

        let count: i64 = recipes
            .select(count(result_item_id))
            .filter(result_item_id.eq(self.id))
            .first(connection)
            .unwrap();

        count > 0
    }

    pub fn get_recipe(&self, quantity: &i16, connection: &mut PgConnection) -> ItemList {
        use crate::database::schema::*;
        use diesel::prelude::*;

        let (items_result, items_ingredient) =
            diesel::alias!(items as items_result, items as items_ingredient);

        let result: Vec<(Item, Recipe, Item)> = items_result
            .inner_join(
                recipes::table.on(items_result.field(items::id).eq(recipes::result_item_id)),
            )
            .inner_join(
                items_ingredient.on(items_ingredient
                    .field(items::id)
                    .eq(recipes::ingredient_item_id)),
            )
            .filter(items_result.field(items::id).eq(self.id))
            .load(connection)
            .unwrap();

        let mut items = ItemList::new();
        result
            .into_iter()
            .for_each(|(_, recipe, items_ingredient)| {
                items.add_item(&items_ingredient, &(recipe.quantity * quantity));
            });

        items
    }

    pub fn get_full_recipe(
        &self,
        quantity: &i16,
        connection: &mut PgConnection,
    ) -> (ItemList, Vec<ItemList>) {
        // used in loop to know which item we still need the recipe of
        let mut items_to_make = ItemList::new();
        items_to_make.add_item(self, quantity);

        // The last materials of the items
        let mut base_ingredients = ItemList::new();

        // All the steps to create the item
        let mut ingredients_steps: Vec<ItemList> = Vec::new();

        // Temporary to set items to make
        let mut next_items_to_make = ItemList::new();

        while !items_to_make.is_empty() {
            ingredients_steps.push(ItemList::new());
            items_to_make.iter().for_each(|(item, quantity)| {
                if item.has_recipe(connection) {
                    ingredients_steps
                        .last_mut()
                        .unwrap()
                        .add_item(item, quantity);

                    let next = &item.get_recipe(quantity, connection);
                    next_items_to_make.add_items(next);
                } else {
                    base_ingredients.add_item(item, quantity);
                }
            });

            items_to_make.clear();
            items_to_make.add_items(&next_items_to_make);
            next_items_to_make.clear();
        }

        // First is wanted item and last is empty
        ingredients_steps.remove(0);
        ingredients_steps.pop();

        // put first the closest to base ingredients and last closest to final item
        ingredients_steps.reverse();

        (base_ingredients, ingredients_steps)
    }
}

#[derive(Clone)]
pub struct ItemList {
    inner: BTreeMap<Item, i16>,
}

impl Default for ItemList {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemList {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    pub fn with_items(items: BTreeMap<Item, i16>) -> Self {
        Self { inner: items }
    }

    pub fn add_item(&mut self, item: &Item, quantity: &i16) {
        self.inner
            .entry(item.clone())
            .and_modify(|current_quantity| {
                *current_quantity += quantity;
            })
            .or_insert(*quantity);
    }

    pub fn add_items(&mut self, items: &ItemList) {
        items.iter().for_each(|(item, quantity)| {
            self.add_item(item, quantity);
        });
    }

    pub fn set_item(&mut self, item: &Item, new_quantity: &i16) {
        self.inner
            .entry(item.clone())
            .and_modify(|current_quantity| {
                *current_quantity = *new_quantity;
            })
            .or_insert(*new_quantity);
    }
}

impl Deref for ItemList {
    type Target = BTreeMap<Item, i16>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ItemList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl std::fmt::Debug for ItemList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
