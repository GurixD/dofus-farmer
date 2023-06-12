use diesel::prelude::*;
use lombok::AllArgsConstructor;

#[derive(Queryable, Selectable, Insertable, AllArgsConstructor)]
#[diesel(table_name = crate::database::schema::monsters_sub_areas)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MonsterSubArea {
    pub monster_id: i32,
    pub sub_area_id: i32,
}
