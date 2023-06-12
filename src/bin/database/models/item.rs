use diesel::prelude::*;
use lombok::AllArgsConstructor;

#[derive(Queryable, Selectable, Insertable, AllArgsConstructor)]
#[diesel(table_name = crate::database::schema::items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Item {
    pub id: i32,
    pub name: String,
}
