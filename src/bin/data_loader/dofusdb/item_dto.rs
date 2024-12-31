use serde::Deserialize;
use serde_flat_path::flat_path;

#[flat_path]
#[derive(Deserialize, Debug)]
pub struct ItemDto {
    pub id: i32,
    #[flat_path("name.fr")]
    pub name: String,
    #[flat_path("type.categoryId")]
    pub category: i16,
    #[serde(rename = "iconId")]
    pub icon: i32,
}
