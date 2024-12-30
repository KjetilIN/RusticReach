use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex}};

pub type Rooms = Arc<Mutex<HashMap<String, HashSet<String>>>>;