use super::{
    monsters_tab::MonstersTab, resources_tab::ResourcesTab, search_item_tabs::SearchItemTab,
    wish_list_tab::WishListTab,
};

pub enum ItemTabsData {
    SearchItem(SearchItemTab),
    WishList(WishListTab),
    Resources(ResourcesTab),
    Monsters(MonstersTab),
}
