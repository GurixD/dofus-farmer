use diesel::prelude::*;
use lombok::AllArgsConstructor;

#[derive(Queryable, Selectable, Insertable, AllArgsConstructor)]
#[diesel(table_name = crate::database::schema::monsters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Monster {
    pub id: i32,
    pub name: String,
}
