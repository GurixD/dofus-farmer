use super::{search_item_tabs::SearchItemTab, wish_list_tab::WishListTab};

pub enum ItemTabsData {
    SearchItem(SearchItemTab),
    WishList(WishListTab),
}
