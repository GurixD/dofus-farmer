// @generated automatically by Diesel CLI.

diesel::table! {
    areas (id) {
        id -> Int4,
        name -> Varchar,
    }
}

diesel::table! {
    drops (monster_id, item_id) {
        monster_id -> Int4,
        item_id -> Int4,
    }
}

diesel::table! {
    items (id) {
        id -> Int4,
        name -> Varchar,
        category -> Int2,
        image_id -> Int4,
    }
}

diesel::table! {
    maps (id) {
        id -> Int4,
        name -> Nullable<Varchar>,
        x -> Int2,
        y -> Int2,
        sub_area_id -> Int4,
    }
}

diesel::table! {
    monsters (id) {
        id -> Int4,
        name -> Varchar,
    }
}

diesel::table! {
    monsters_sub_areas (monster_id, sub_area_id) {
        monster_id -> Int4,
        sub_area_id -> Int4,
    }
}

diesel::table! {
    recipes (result_item_id, ingredient_item_id) {
        result_item_id -> Int4,
        ingredient_item_id -> Int4,
        quantity -> Int2,
    }
}

diesel::table! {
    sub_areas (id) {
        id -> Int4,
        name -> Varchar,
        area_id -> Int4,
    }
}

diesel::table! {
    user_ingredients (item_id) {
        item_id -> Int4,
        quantity -> Int2,
    }
}

diesel::table! {
    user_items (item_id) {
        item_id -> Int4,
        quantity -> Int2,
    }
}

diesel::joinable!(drops -> items (item_id));
diesel::joinable!(drops -> monsters (monster_id));
diesel::joinable!(maps -> sub_areas (sub_area_id));
diesel::joinable!(monsters_sub_areas -> monsters (monster_id));
diesel::joinable!(monsters_sub_areas -> sub_areas (sub_area_id));
diesel::joinable!(sub_areas -> areas (area_id));
diesel::joinable!(user_ingredients -> items (item_id));
diesel::joinable!(user_items -> items (item_id));

diesel::allow_tables_to_appear_in_same_query!(
    areas,
    drops,
    items,
    maps,
    monsters,
    monsters_sub_areas,
    recipes,
    sub_areas,
    user_ingredients,
    user_items,
);
