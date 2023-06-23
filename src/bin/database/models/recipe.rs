use diesel::prelude::*;
use lombok::AllArgsConstructor;

#[derive(Queryable, Selectable, Insertable, AllArgsConstructor, Debug)]
#[diesel(table_name = crate::database::schema::recipes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Recipe {
    pub result_item_id: i32,
    pub ingredient_item_id: i32,
    pub quantity: i16,
}
