use diesel::prelude::*;
use lombok::AllArgsConstructor;

#[derive(Queryable, Selectable, Insertable, AllArgsConstructor)]
#[diesel(table_name = crate::database::schema::maps)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Map {
    pub id: i32,
    pub name: Option<String>,
    pub x: i16,
    pub y: i16,
    pub sub_area_id: i32,
}
