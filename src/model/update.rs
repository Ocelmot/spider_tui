use crossterm::event::Event;
use spider_client::{
    message::{AbsoluteDatasetPath, DatasetData, UiElementUpdate, UiPage},
    SpiderId2048,
};

pub enum ModelUpdate {
    Event(Event),
    SetPages(Vec<UiPage>),
    SetPage(UiPage),
    UpdateElementsFor(SpiderId2048, Vec<UiElementUpdate>),
    UpdateDataset(AbsoluteDatasetPath, Vec<DatasetData>),
}
