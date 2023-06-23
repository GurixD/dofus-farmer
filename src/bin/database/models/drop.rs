use diesel::prelude::*;
use lombok::AllArgsConstructor;

#[derive(Queryable, Selectable, Insertable, AllArgsConstructor, Debug)]
#[diesel(table_name = crate::database::schema::drops)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Drop {
    pub monster_id: i32,
    pub item_id: i32,
}
