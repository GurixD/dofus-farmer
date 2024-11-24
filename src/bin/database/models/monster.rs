use std::hash::Hash;

use diesel::prelude::*;
use lombok::AllArgsConstructor;

#[derive(Queryable, Selectable, Insertable, AllArgsConstructor, Debug, Clone)]
#[diesel(table_name = crate::database::schema::monsters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Monster {
    pub id: i32,
    pub name: String,
}

impl Hash for Monster {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Monster {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Monster {}

impl PartialOrd for Monster {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl Ord for Monster {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}
