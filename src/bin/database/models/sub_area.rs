use diesel::prelude::*;
use lombok::AllArgsConstructor;

#[derive(Queryable, Selectable, Identifiable, Insertable, AllArgsConstructor, Debug, Eq, Hash)]
#[diesel(table_name = crate::database::schema::sub_areas)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SubArea {
    pub id: i32,
    pub name: String,
    pub area_id: i32,
}

impl PartialEq for SubArea {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
