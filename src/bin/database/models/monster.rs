use diesel::prelude::*;
use lombok::AllArgsConstructor;

#[derive(
    Queryable, Selectable, Insertable, AllArgsConstructor, Debug, PartialEq, Eq, Hash, Clone,
)]
#[diesel(table_name = crate::database::schema::monsters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Monster {
    pub id: i32,
    pub name: String,
}

impl PartialOrd for Monster {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for Monster {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}
