use diesel::prelude::*;
use lombok::AllArgsConstructor;

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AllArgsConstructor,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Clone,
)]
#[diesel(table_name = crate::database::schema::sub_areas)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SubArea {
    pub id: i32,
    pub name: String,
    pub area_id: i32,
}
