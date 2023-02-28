use std::io::{self, Stdout};
use crossterm::{
	event::{
		Event,
		self,
		EnableMouseCapture,
		DisableMouseCapture
	},
	terminal::{
		enable_raw_mode,
		disable_raw_mode,
		EnterAlternateScreen,
		LeaveAlternateScreen
	},
	execute
};

use spider_client::message::{UiPage, UiElement};
use tui::{
	Terminal,
	widgets::{Block, Borders, Paragraph, BorderType, List, ListItem},
	layout::{Layout, Direction, Constraint, Rect},
	backend::Backend, Frame, style::{Color, Style, Modifier}
};

use tui::backend::CrosstermBackend;


use super::Renderer;

pub struct TUI{
	
	term: Terminal<CrosstermBackend<Stdout>>,
}

impl TUI{
	pub fn new() -> Self{

		let mut stdout = io::stdout();
		execute!(stdout, EnterAlternateScreen, EnableMouseCapture);
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

	fn render_page(&mut self, page: &UiPage) {
		
		self.term.draw(|frame|{

			let b = Block::default()
				.title(format!("{} (esc=Menu)", page.name()))
				.borders(Borders::all())
				.border_style(Style::default().fg(Color::White))
				.border_type(BorderType::Rounded)
				.style(Style::default().bg(Color::Black));
			let inner_size = b.inner(frame.size());
			frame.render_widget(b, frame.size());

			draw_elem(frame, &page, inner_size, page.root());			

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
			DisableMouseCapture,
			LeaveAlternateScreen,
		).unwrap();
		//terminal.show_cursor();
	}




}


fn draw_elem<B: Backend>(frame: &mut Frame<B>, page: &UiPage, rect: Rect, elem: &UiElement){
	let widget = match elem.kind(){
		spider_client::message::UiElementKind::Columns => todo!(),
		spider_client::message::UiElementKind::Rows => {
			let mut constraints = Vec::new();
			for child in elem.children(){
				constraints.push(Constraint::Min(1));
			}
			let areas = Layout::default()
				.constraints(constraints)
				.direction(Direction::Vertical)
				.split(rect);
			for (child, area) in elem.children().zip(areas.iter()){
				// frame.render_widget(widget, area);
				draw_elem(frame, page, *area, child);
			}
		},
		spider_client::message::UiElementKind::Grid(_, _) => todo!(),
		spider_client::message::UiElementKind::Text => {
			let w = Paragraph::new(elem.text().clone());
			frame.render_widget(w, rect);
		},
		spider_client::message::UiElementKind::TextEntry => todo!(),
		spider_client::message::UiElementKind::Button => todo!(),
	};
}
