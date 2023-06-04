use std::collections::HashMap;

use spider_client::message::{UiElement, UiPageManager, UiElementKind, DatasetData, AbsoluteDatasetPath};





pub struct PageState{
    selected: Option<String>, // Element id for selected element
    selected_datasets: Vec<usize>,
    selected_datum: Option<DatasetData>,

    uncommited_inputs: HashMap<(String, Vec<usize>), String>, // Map from element id to contents
}

impl Default for PageState{
    fn default() -> Self {
        Self{
            selected: Default::default(),
            selected_datasets: Default::default(),
            selected_datum: None,

            uncommited_inputs: Default::default()
        }
    }
}

impl PageState{
    // uncommited input
    pub fn get_uncommited_input(&self, id: &String, dataset_indices: &Vec<usize>) -> Option<&String>{
        let key = (id.clone(), dataset_indices.clone());
        self.uncommited_inputs.get(&key)
    }
    pub fn get_uncommited_input_mut(&mut self, id: &String, dataset_indices: &Vec<usize>) -> Option<&mut String>{
        let key = (id.clone(), dataset_indices.clone());
        self.uncommited_inputs.get_mut(&key)
    }
    pub fn clear_uncommitted_input(&mut self, id: &String, dataset_indices: &Vec<usize>){
        let key = (id.clone(), dataset_indices.clone());
        self.uncommited_inputs.remove(&key);
    }
    pub fn set_uncommited_input(&mut self, id: String, dataset_indices: &Vec<usize>, value: String){
        let key = (id.clone(), dataset_indices.clone());
        self.uncommited_inputs.insert(key, value);
    }
    pub fn get_selected_uncommited_input_mut(&mut self) -> Option<&mut String>{
        match &self.selected{
            Some(id) => {
                let key = (id.clone(), self.selected_datasets.clone());
                self.uncommited_inputs.get_mut(&key)
            },
            None => None,
        }
    }
    pub fn take_selected_uncommited_input_mut(&mut self) -> Option<String>{
        match &self.selected{
            Some(id) => {
                let key = (id.clone(), self.selected_datasets.clone());
                self.uncommited_inputs.remove(&key)
            },
            None => None,
        }
    }
    pub fn set_selected_uncommited_input(&mut self,  value: String){
        match &self.selected{
            Some(id) => {
                let key = (id.clone(), self.selected_datasets.clone());
                self.uncommited_inputs.insert(key, value);
            },
            None => {},
        };
    }

    // Selected element management
    pub fn get_selected_id(&self) -> Option<&String>{
        self.selected.as_ref()
    }
    pub fn get_selected_datasets(&self) -> &Vec<usize>{
        &self.selected_datasets
    }
    pub fn get_selected_datum(&self) -> &Option<DatasetData>{
        &self.selected_datum
    }

    pub fn select_next(&mut self, mgr: &UiPageManager, data_map: &HashMap<AbsoluteDatasetPath, Vec<DatasetData>>, direction: SelectDirection) {
        let path = self.get_selected_id().and_then(|s| mgr.get_path(s));
        let mut selected_path = match path{
            // get selected path (string id -> get path from page)
            // traverse to that node
            Some(path) => {
                let mut v = Vec::new();
                let mut dataset_indices = self.get_selected_datasets().clone();
                let mut datum = None;
                let mut recent_dataset_indices = vec![];
                let mut last_elem = mgr.get_page().root();
                for i in path.iter(){
                    let mut idx = *i;
                    let rdi = recent_dataset_indices.clone();
                    let d = datum.clone();
                    
                    if let Some(path) = last_elem.dataset(){
                        // if there is a dataset, include next element from dataset_indices
                        let index = dataset_indices.remove(0);
                        idx = (index * last_elem.children().len()) + idx; // convert idx to use with dataset children
                        recent_dataset_indices.push(index);

                        // try to get data, replace datum with data.
                        datum = match data_map.get(path){
                            Some(d) => d.get(index), 
                            None => None,
                        };   
                    }
                    v.push((idx, last_elem, rdi, d));
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
                let mut datum = None;
                let mut recent_dataset_indices = vec![];
                v.push((0, mgr.get_page().root(), recent_dataset_indices.clone(), datum.clone()));
                loop {
                    let (_, prev, _, _) = v.last().unwrap();
                    if let Some(path) = prev.dataset(){
                        // if there is a dataset, select the first element of
                        recent_dataset_indices.push(0usize);
                        // try to get data, replace datum with data.
                        datum = match data_map.get(path){
                            Some(d) => d.get(0), // always choose first item
                            None => None,
                        };
                    }
                    match prev.get_child(0) {
                        Some(child) => v.push((0, child, recent_dataset_indices.clone(), datum.clone())),
                        None => break,
                    }
                }
                // if firstmost element is selectable, select that element
                if let Some((_, firstmost, dataset_indices, datum)) = v.last(){
                    if let Some(id) = firstmost.id() {
                        if firstmost.selectable() {
                            self.selected = Some(id.clone());
                            self.selected_datasets = dataset_indices.to_vec();
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
        for (index, element, mut dataset_indices, datum) in selected_path{
            match elem_select_next(element, index, &datum, data_map, direction) {
                Some((string, mut dataset_indices_tail, selected_datum)) => {
                    self.selected = Some(string);
                    dataset_indices.append(&mut dataset_indices_tail);
                    self.selected_datasets = dataset_indices;
                    self.selected_datum = selected_datum;
                    break;
                },
                None => {}, // continue up path
            }
        }
    }
}

fn elem_select_next(elem: &UiElement, index: usize, data: &Option<&DatasetData>, data_map: &HashMap<AbsoluteDatasetPath, Vec<DatasetData>>, direction: SelectDirection) -> Option<(String, Vec<usize>, Option<DatasetData>)>{
    match elem.kind(){
        UiElementKind::Columns => {
            match direction{
                SelectDirection::Up => None,
                SelectDirection::Down => None,
                SelectDirection::Left => {
                    for (i, child, datum) in elem.children_dataset(data, data_map).take(index).rev(){
                        let res = elem_select_enter_towards(child, &datum, data_map, direction);
                        if let Some((id, mut dataset_tail, selected_datum)) = res{
                            // need to prepend to the data index tail if this node had a dataset
                            return match i{
                                Some(dataset_index) => {
                                    let mut tail = vec![dataset_index];
                                    tail.append(&mut dataset_tail);
                                    Some((id, tail, selected_datum))
                                },
                                None => Some((id, dataset_tail, selected_datum)),
                            };
                        }
                    }
                    None
                },
                SelectDirection::Right => {
                    for (i, child, datum) in elem.children_dataset(data, data_map).skip(index+1){
                        let res = elem_select_enter_towards(child, &datum, data_map, direction);
                        if let Some((id, mut dataset_tail, selected_datum)) = res{
                            // need to prepend to the data index tail if this node had a dataset
                            return match i{
                                Some(dataset_index) => {
                                    let mut tail = vec![dataset_index];
                                    tail.append(&mut dataset_tail);
                                    Some((id, tail, selected_datum))
                                },
                                None => Some((id, dataset_tail, selected_datum)),
                            };
                        }
                    }
                    None
                },
            }
        },
        UiElementKind::Rows => {
            match direction{
                SelectDirection::Up => {
                    for (i, child, datum) in elem.children_dataset(data, data_map).take(index).rev(){
                        let res = elem_select_enter_towards(child, &datum, data_map, direction);
                        if let Some((id, mut dataset_tail, selected_datum)) = res{
                            // need to prepend to the data index tail if this node had a dataset
                            return match i{
                                Some(dataset_index) => {
                                    let mut tail = vec![dataset_index];
                                    tail.append(&mut dataset_tail);
                                    Some((id, tail, selected_datum))
                                },
                                None => Some((id, dataset_tail, selected_datum)),
                            };
                        }
                    }
                    None
                },
                SelectDirection::Down => {
                    for (i, child, datum) in elem.children_dataset(data, data_map).skip(index+1){
                        let res = elem_select_enter_towards(child, &datum, data_map, direction);
                        if let Some((id, mut dataset_tail, selected_datum)) = res{
                            // need to prepend to the data index tail if this node had a dataset
                            return match i{
                                Some(dataset_index) => {
                                    let mut tail = vec![dataset_index];
                                    tail.append(&mut dataset_tail);
                                    Some((id, tail, selected_datum))
                                },
                                None => Some((id, dataset_tail, selected_datum)),
                            };
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

fn elem_select_enter_towards(elem: &UiElement, data: &Option<&DatasetData>, data_map: &HashMap<AbsoluteDatasetPath, Vec<DatasetData>>, direction: SelectDirection) -> Option<(String, Vec<usize>, Option<DatasetData>)> {
    match elem.kind(){
        UiElementKind::Columns => {
            match direction{
                SelectDirection::Left => { // reverse order
                    for (i, child, datum) in elem.children_dataset(data, data_map).rev(){
                        let ret = elem_select_enter_towards(child, &datum, data_map, direction);
                        if let Some((id, mut dataset_tail, selected_datum)) = ret{
                            // need to prepend to the data index tail if this node had a dataset
                            return match i{
                                Some(dataset_index) => {
                                    let mut tail = vec![dataset_index];
                                    tail.append(&mut dataset_tail);
                                    Some((id, tail, selected_datum))
                                },
                                None => Some((id, dataset_tail, selected_datum)),
                            };
                        }
                    }
                    None
                }, 
                _ => { // get first valid
                    for (i, child, datum) in elem.children_dataset(data, data_map){
                        let ret = elem_select_enter_towards(child, &datum, data_map, direction);
                        if let Some((id, mut dataset_tail, selected_datum)) = ret{
                            // need to prepend to the data index tail if this node had a dataset
                            return match i{
                                Some(dataset_index) => {
                                    let mut tail = vec![dataset_index];
                                    tail.append(&mut dataset_tail);
                                    Some((id, tail, selected_datum))
                                },
                                None => Some((id, dataset_tail, selected_datum)),
                            };
                        }
                    }
                    None
                },
            }
        },
        UiElementKind::Rows => {
            match direction{
                SelectDirection::Up => { // reverse order
                    for (i, child, datum) in elem.children_dataset(data, data_map).rev(){
                        let ret = elem_select_enter_towards(child, &datum, data_map, direction);
                        if let Some((id, mut dataset_tail, selected_datum)) = ret{
                            // need to prepend to the data index tail if this node had a dataset
                            return match i{
                                Some(dataset_index) => {
                                    let mut tail = vec![dataset_index];
                                    tail.append(&mut dataset_tail);
                                    Some((id, tail, selected_datum))
                                },
                                None => Some((id, dataset_tail, selected_datum)),
                            };
                        }
                    }
                    None
                }, 
                _ => { // get first valid
                    for (i, child, datum) in elem.children_dataset(data, data_map){
                        let ret = elem_select_enter_towards(child, &datum, data_map, direction);
                        if let Some((id, mut dataset_tail, selected_datum)) = ret{
                            // need to prepend to the data index tail if this node had a dataset
                            return match i{
                                Some(dataset_index) => {
                                    let mut tail = vec![dataset_index];
                                    tail.append(&mut dataset_tail);
                                    Some((id, tail, selected_datum))
                                },
                                None => Some((id, dataset_tail, selected_datum)),
                            };
                        }
                    }
                    None
                },
            }
        },
        UiElementKind::Grid(_, _) => todo!(),
        _ => {
            let not_none = elem.kind().clone().resolve(data) != UiElementKind::None;
            if elem.selectable() && not_none{
                match elem.id(){
                    Some(id) => {
                        Some((id.clone(), Vec::new(), data.cloned()))
                    },
                    None => None,
                }
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

