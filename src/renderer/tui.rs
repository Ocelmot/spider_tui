use std::{io::{self, Stdout}, collections::HashMap};
use crossterm::{
	event::{
		EnableMouseCapture,
		DisableMouseCapture,
		EnableBracketedPaste,
		DisableBracketedPaste
	},
	terminal::{
		enable_raw_mode,
		disable_raw_mode,
		EnterAlternateScreen,
		LeaveAlternateScreen
	},
	execute
};

use spider_client::message::{UiPage, UiElement, DatasetData, AbsoluteDatasetPath, UiElementKind};
use tui::{
	Terminal,
	widgets::{Block, Borders, Paragraph, BorderType, List, ListItem},
	layout::{Layout, Direction, Constraint, Rect},
	backend::Backend, Frame, style::{Color, Style, Modifier}, text::Text
};

use tui::backend::CrosstermBackend;


use crate::model::processor::PageState;

use super::Renderer;

pub struct TUI{
	
	term: Terminal<CrosstermBackend<Stdout>>,
}

impl TUI{
	pub fn new() -> Self{

		let mut stdout = io::stdout();
		execute!(
			stdout,
			EnterAlternateScreen,
			// EnableMouseCapture,
			EnableBracketedPaste
		).unwrap();
		let backend = CrosstermBackend::new(stdout);
		let terminal = Terminal::new(backend).expect("able to create a terminal");

		Self {
			term: terminal
		}
	}
}

impl Renderer for TUI{
	fn startup(&mut self) {
		enable_raw_mode();
	}

	fn render_menu(&mut self) {
		todo!()
	}

	fn render_page(&mut self, page: &UiPage, state: &PageState, data_map: &HashMap<AbsoluteDatasetPath, Vec<DatasetData>>) {
		self.term.draw(|frame|{
			let constraints = vec![Constraint::Min(5), Constraint::Length(1)];
			let areas = Layout::default()
				.constraints(constraints)
				.direction(Direction::Vertical)
				.split(frame.size());

			let b = Block::default()
				.title(format!("{} (esc=Menu)", page.name()))
				.borders(Borders::TOP)
				.border_style(Style::default().fg(Color::White))
				.border_type(BorderType::Double)
				.style(Style::default().bg(Color::Black));
			let inner_size = b.inner(areas[0]);
			frame.render_widget(b, areas[0]);

			// debug area
			let default = String::from("-");
			let id_text = state.get_selected_id().unwrap_or(&default);
			let indexes = state.get_selected_datasets();
			let widget = Paragraph::new(format!("{} | {:?}", id_text, indexes));
			frame.render_widget(widget, areas[1]);

			draw_elem(frame, state, inner_size, page.root(), &None, data_map, &Vec::new());		

		}).unwrap();
	}

	fn render_page_list(&mut self, list: &Vec<&UiPage>, highlight_index: usize) {
		self.term.draw(|frame|{

			let b = Block::default()
				.title("Select Page (q=Quit)")
				.borders(Borders::all())
				.border_style(Style::default().fg(Color::White))
				.border_type(BorderType::Rounded)
				.style(Style::default().bg(Color::Black));

			// frame.render_widget(w, frame.size());

			let mut list_items = Vec::new();
			for (i, item) in list.iter().enumerate(){
				let mut list_item: ListItem =  ListItem::new(item.name().clone());
				if i == highlight_index{
					let style = Style::default()
						.bg(Color::LightGreen)
						.add_modifier(Modifier::BOLD);

					list_item = list_item.style(style);
				}
				list_items.push(list_item);
			}
			let list = List::new(list_items).block(b);
			
			frame.render_widget(list, frame.size());
			
		}).unwrap();
	}

	fn shutdown(mut self) {
		// cleanup
		disable_raw_mode();
		execute!(
			self.term.backend_mut(),
			DisableBracketedPaste,
			// DisableMouseCapture,
			LeaveAlternateScreen,
		).unwrap();
		//terminal.show_cursor();
	}




}


fn draw_elem<B: Backend>(frame: &mut Frame<B>, state: &PageState, rect: Rect, elem: &UiElement, data: &Option<&DatasetData>, data_map: &HashMap<AbsoluteDatasetPath, Vec<DatasetData>>, dataset_indices: &Vec<usize>){
	let content = match data {
		Some(data) => elem.render_content(data),
		None => elem.text(),
	};
	let mut elem_kind = elem.kind().clone();
	elem_kind = elem_kind.resolve(data);


	match elem_kind{
		UiElementKind::None =>{}
		UiElementKind::Spacer =>{}
		spider_client::message::UiElementKind::Columns => {
			// calc constraints
			let mut constraints = Vec::new();
			for (_, child, datum) in elem.children_dataset(data, data_map){
				if let UiElementKind::Spacer = child.kind(){
					constraints.push(Constraint::Min(0))
				}else{
					constraints.push(Constraint::Length(elem_calc_width(child, &datum, data_map)));
				}
			}
			let areas = Layout::default()
				.constraints(constraints)
				.direction(Direction::Horizontal)
				.split(rect);
			let mut areas = areas.iter();
			// render children
			let mut v: Vec<usize>;
			for (cdi, child, datum) in elem.children_dataset(data, data_map){
				let area = areas.next().expect("areas should be dataset * children in length");
				let child_dataset_indices = match cdi{
					Some(cdi) => {
						v = dataset_indices.clone();
						v.push(cdi);
						&v
					},
					None => dataset_indices,
				};
				draw_elem(frame, state, *area, child, &datum, data_map, child_dataset_indices);
			}
		},
		spider_client::message::UiElementKind::Rows => {
			// calc constraints
			let mut constraints = Vec::new();
			for (_, child, datum) in elem.children_dataset(data, data_map){
				if let UiElementKind::Spacer = child.kind(){
					constraints.push(Constraint::Min(0))
				}else{
					constraints.push(Constraint::Length(elem_calc_height(child, &datum, data_map)));
				}
			}
			constraints.push(Constraint::Min(0));
			let areas = Layout::default()
				.constraints(constraints)
				.direction(Direction::Vertical)
				.split(rect);
			let mut areas = areas.iter();
			// render children
			let mut v: Vec<usize>;
			for (cdi, child, datum) in elem.children_dataset(data, data_map){
				let area = areas.next().expect("areas should be dataset * children in length");
				let child_dataset_indices = match cdi{
					Some(cdi) => {
						v = dataset_indices.clone();
						v.push(cdi);
						&v
					},
					None => dataset_indices,
				};
				draw_elem(frame, state, *area, child, &datum, data_map, child_dataset_indices);
			}
		},
		spider_client::message::UiElementKind::Grid(_, _) => todo!(),
		spider_client::message::UiElementKind::Text => {
			let mut w = Paragraph::new(content);
			w = w.wrap(tui::widgets::Wrap { trim: false });
			frame.render_widget(w, rect);
		},
		spider_client::message::UiElementKind::TextEntry => {
			let b = Block::default()
				.title(content)
				.borders(Borders::all())
				// .border_style(Style::default().fg(Color::White))
				.style(Style::default().bg(Color::Black));
			let input_text = match elem.id(){
				Some(id) => {
					match state.get_uncommited_input(id) {
						Some(text) => text,
						None => "",
					}
				},
				None => "",
			};
			let mut w = Paragraph::new(input_text);
			if state.get_selected_id() == elem.id() && state.get_selected_datasets() == dataset_indices{
				w = w.style(Style::default().add_modifier(Modifier::BOLD));
			}
			frame.render_widget(w.block(b), rect);
		},
		spider_client::message::UiElementKind::Button => {
			let b = Block::default().borders(Borders::ALL);
			let mut w = Paragraph::new(content);
			if state.get_selected_id() == elem.id() && state.get_selected_datasets() == dataset_indices{
				w = w.style(Style::default().add_modifier(Modifier::BOLD));
			}
			frame.render_widget(w.block(b), rect);
		},
		UiElementKind::Variable(content_part) => { // If part could not have been resolved
			let w = Paragraph::new("e".to_owned() + &content_part.to_string());
			frame.render_widget(w, rect);
		} 
	}
}


fn elem_calc_height(elem: &UiElement, data: &Option<&DatasetData>, data_map: &HashMap<AbsoluteDatasetPath, Vec<DatasetData>>) -> u16{
	let mut elem_kind = elem.kind().clone();
	elem_kind = elem_kind.resolve(data);

	match elem_kind{
		UiElementKind::None => 0,
		UiElementKind::Spacer => 0,
		spider_client::message::UiElementKind::Columns => {
			let mut height = 0;
			for (_, child, data) in elem.children_dataset(data, data_map){
				let child_height = elem_calc_height(child, &data, data_map);
				if child_height > height{
					height = child_height; 
				}
			}
			height
		},
		spider_client::message::UiElementKind::Rows => {
			let mut height = 0;
			for (_, child, data) in elem.children_dataset(data, data_map){
				height += elem_calc_height(child, &data, data_map);
			}
			height
		},
		spider_client::message::UiElementKind::Grid(_, _) => todo!(),
		spider_client::message::UiElementKind::Text => {
			let t = elem.render_content_opt(data);
			// let text = Text::from(t);
			// TryInto::<u16>::try_into(text.height()).unwrap() + 1
			let len = t.chars().count();
			TryInto::<u16>::try_into((len / 80)).unwrap() + 1
		},
		spider_client::message::UiElementKind::TextEntry => 3,
		spider_client::message::UiElementKind::Button => 3,
		UiElementKind::Variable(_) => 1,
	}
}

fn elem_calc_width(elem: &UiElement, data: &Option<&DatasetData>, data_map: &HashMap<AbsoluteDatasetPath, Vec<DatasetData>>) -> u16{
	let mut elem_kind = elem.kind().clone();
	elem_kind = elem_kind.resolve(data);

	match elem_kind{
		UiElementKind::None => 0,
		UiElementKind::Spacer => 0,
		spider_client::message::UiElementKind::Columns => {
			let mut width = 0;
			for (_, child, data) in elem.children_dataset(data, data_map){
				width += elem_calc_width(child, &data, data_map);
			}
			width
		},
		spider_client::message::UiElementKind::Rows => {
			let mut width = 0;
			for (_, child, data) in elem.children_dataset(data, data_map){
				let child_width = elem_calc_width(child, &data, data_map);
				if child_width > width{
					width = child_width; 
				}
			}
			width
		},
		spider_client::message::UiElementKind::Grid(_, _) => todo!(),
		spider_client::message::UiElementKind::Text => {
			let text = Text::from(elem.render_content_opt(data));
			TryInto::<u16>::try_into(text.width()).unwrap()
		},
		spider_client::message::UiElementKind::TextEntry => 35,
		spider_client::message::UiElementKind::Button => (elem.render_content_opt(data).chars().count() + 2) as u16,
		UiElementKind::Variable(_) => {
			let text = Text::from(elem.render_content_opt(data));
			TryInto::<u16>::try_into(text.width()).unwrap()
		},
	}
}