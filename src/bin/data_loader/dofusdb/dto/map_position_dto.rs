use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MapPositionDto {
    pub id: i32,
    #[serde(rename = "posX")]
    pub x: i16,
    #[serde(rename = "posY")]
    pub y: i16,
    #[serde(rename = "subAreaId")]
    pub sub_area_id: i32,
    #[serde(rename = "hasPriorityOnWorldmap")]
    pub has_priority: bool,
}
