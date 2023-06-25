use diesel::prelude::*;
use lombok::AllArgsConstructor;

#[derive(
    Queryable,
    Selectable,
    Insertable,
    AsChangeset,
    AllArgsConstructor,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Clone,
)]
#[diesel(table_name = crate::database::schema::user_items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserItem {
    pub item_id: i32,
    pub quantity: i16,
}
