use diesel::prelude::*;
use lombok::AllArgsConstructor;

#[derive(
    Queryable, Selectable, Insertable, AllArgsConstructor, Debug, PartialEq, Eq, Hash, Clone,
)]
#[diesel(table_name = crate::database::schema::items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Item {
    pub id: i32,
    pub name: String,
    pub category: i16,
    pub image_id: i32,
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}
