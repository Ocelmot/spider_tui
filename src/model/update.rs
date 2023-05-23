use crossterm::event::Event;
use spider_client::{message::{UiPage, UiElementUpdate, AbsoluteDatasetPath, DatasetData}, SpiderId2048};




pub enum ModelUpdate{
	Event(Event),
	SetPages(Vec<UiPage>),
	SetPage(UiPage),
	UpdateElementsFor(SpiderId2048, Vec<UiElementUpdate>),
	UpdateDataset(AbsoluteDatasetPath, Vec<DatasetData>),
}