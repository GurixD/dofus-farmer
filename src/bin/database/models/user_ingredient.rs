use diesel::prelude::*;
use lombok::AllArgsConstructor;

#[derive(
    Queryable,
    Selectable,
    AsChangeset,
    Insertable,
    AllArgsConstructor,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Clone,
)]
#[diesel(table_name = crate::database::schema::user_ingredients)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserIngredient {
    pub item_id: i32,
    pub quantity: i16,
}
