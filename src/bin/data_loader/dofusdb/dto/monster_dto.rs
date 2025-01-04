use serde::Deserialize;
use serde_flat_path::flat_path;

#[flat_path]
#[derive(Deserialize, Debug)]
pub struct MonsterDto {
    pub id: i32,
    #[flat_path("name.fr")]
    pub name: String,
    #[serde(rename = "gfxId")]
    pub gfx_id: i32,
}
