use super::sub_area::SubArea;
use diesel::prelude::*;
use lombok::AllArgsConstructor;

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    Associations,
    AsChangeset,
    AllArgsConstructor,
    Debug,
)]
#[diesel(table_name = crate::database::schema::maps)]
#[diesel(belongs_to(SubArea))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Map {
    pub id: i32,
    pub name: Option<String>,
    pub x: i16,
    pub y: i16,
    pub sub_area_id: i32,
}
