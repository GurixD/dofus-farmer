use diesel::prelude::*;
use lombok::AllArgsConstructor;

#[derive(Queryable, Selectable, Insertable, AllArgsConstructor, Debug)]
#[diesel(table_name = crate::database::schema::areas)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Area {
    pub id: i32,
    pub name: String,
}
