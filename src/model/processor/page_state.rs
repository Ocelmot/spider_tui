use std::collections::HashMap;

use spider_client::message::{UiElement, UiPageManager, UiElementKind};





pub struct PageState{
    selected: Option<String>, // Element id for selected element
    uncommited_inputs: HashMap<String, String>, // Map from element id to contents
}

impl Default for PageState{
    fn default() -> Self {
        Self{
            selected: Default::default(),
            uncommited_inputs: Default::default()
        }
    }
}

impl PageState{
    // uncommited input
    pub fn get_uncommited_input(&self, id: &String) -> Option<&String>{
        self.uncommited_inputs.get(id)
    }
    pub fn get_uncommited_input_mut(&mut self, id: &String) -> Option<&mut String>{
        self.uncommited_inputs.get_mut(id)
    }
    pub fn clear_uncommitted_input(&mut self, id: &String){
        self.uncommited_inputs.remove(id);
    }
    pub fn set_uncommited_input(&mut self, id: String, value: String){
        self.uncommited_inputs.insert(id, value);
    }
    pub fn get_selected_uncommited_input_mut(&mut self) -> Option<&mut String>{
        match &self.selected{
            Some(id) => {
                self.uncommited_inputs.get_mut(id)
            },
            None => None,
        }
    }
    pub fn take_selected_uncommited_input_mut(&mut self) -> Option<String>{
        match &self.selected{
            Some(id) => {
                self.uncommited_inputs.remove(id)
            },
            None => None,
        }
    }
    pub fn set_selected_uncommited_input(&mut self,  value: String){
        match &self.selected{
            Some(id) => {
                self.uncommited_inputs.insert(id.to_string(), value);
            },
            None => {},
        };
    }

    // Selected element management
    pub fn get_selected_id(&self) -> Option<&String>{
        self.selected.as_ref()
    }

    pub fn select_next(&mut self, mgr: &UiPageManager, direction: SelectDirection) {
        let path = self.get_selected_id().and_then(|s| mgr.get_path(s));
        let mut selected_path = match path{
            // get selected path (string id -> get path from page)
            // traverse to that node
            Some(path) => {
                let mut v = Vec::new();
                let mut last_elem = mgr.get_page().root();
                for i in path.iter(){
                    v.push((*i, last_elem));
                    match last_elem.get_child(*i) {
                        Some(child) => last_elem = child,
                        None => break,
                    }
                }
                v
            },
            // if no element is selected, navigate to firstmost leaf, and try to select from there.
            None => {
                let mut v = Vec::new();
                v.push((0, mgr.get_page().root()));
                loop {
                    let (_, prev) = v.last().unwrap();
                    match prev.get_child(0) {
                        Some(child) => v.push((0, child)),
                        None => break,
                    }
                }
                // if firstmost element is selectable, select that element
                if let Some((_, firstmost)) = v.last(){
                    if let Some(id) = firstmost.id() {
                        if firstmost.selectable() {
                            self.selected = Some(id.clone());
                            return;
                        }
                    }
                }
                v
            },
        };

        // query node for next selectable element with direction
        // if successful, that is new selected element
        // else try next parent
        // if at root, selection cant move, make no change
        selected_path.reverse();
        for (index, element) in selected_path{
            match elem_select_next(element, index, direction) {
                Some(string) => {
                    self.selected = Some(string);
                    break;
                },
                None => {}, // continue up path
            }
        }
    }
}

fn elem_select_next(elem: &UiElement, index: usize, direction: SelectDirection) -> Option<String>{
    match elem.kind(){
        UiElementKind::Columns => {
            match direction{
                SelectDirection::Up => None,
                SelectDirection::Down => None,
                SelectDirection::Left => {
                    for i in elem.children().take(index).rev(){
                        let res = elem_select_enter_towards(i, direction);
                        if res.is_some(){
                            return res;
                        }
                    }
                    None
                },
                SelectDirection::Right => {
                    for i in elem.children().skip(index+1){
                        let res = elem_select_enter_towards(i, direction);
                        if res.is_some(){
                            return res;
                        }
                    }
                    None
                },
            }
        },
        UiElementKind::Rows => {
            match direction{
                SelectDirection::Up => {
                    for i in elem.children().take(index).rev(){
                        let res = elem_select_enter_towards(i, direction);
                        if res.is_some(){
                            return res;
                        }
                    }
                    None
                },
                SelectDirection::Down => {
                    for i in elem.children().skip(index+1){
                        let res = elem_select_enter_towards(i, direction);
                        if res.is_some(){
                            return res;
                        }
                    }
                    None
                },
                SelectDirection::Left => None,
                SelectDirection::Right => None,
            }
        },
        UiElementKind::Grid(_, _) => todo!(),
        _ => None,
    }
}

fn elem_select_enter_towards(elem: &UiElement, direction: SelectDirection) -> Option<String> {
    match elem.kind(){
        UiElementKind::Columns => {
            match direction{
                SelectDirection::Left => { // reverse order
                    for child in elem.children().rev(){
                        let ret = elem_select_enter_towards(child, direction);
                        if ret.is_some() {
                            return ret;
                        }
                    }
                    None
                }, 
                _ => { // get first valid
                    for child in elem.children(){
                        let ret = elem_select_enter_towards(child, direction);
                        if ret.is_some() {
                            return ret;
                        }
                    }
                    None
                },
            }
        },
        UiElementKind::Rows => {
            match direction{
                SelectDirection::Up => { // reverse order
                    for child in elem.children().rev(){
                        let ret = elem_select_enter_towards(child, direction);
                        if ret.is_some() {
                            return ret;
                        }
                    }
                    None
                }, 
                _ => { // get first valid
                    for child in elem.children(){
                        let ret = elem_select_enter_towards(child, direction);
                        if ret.is_some() {
                            return ret;
                        }
                    }
                    None
                },
            }
        },
        UiElementKind::Grid(_, _) => todo!(),
        _ => {
            if elem.selectable(){
                elem.id().cloned()
            }else{
                None
            }
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub enum SelectDirection{
	Up,
	Down,
	Left,
	Right,
}

