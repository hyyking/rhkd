use futures::stream::Stream;

use std::collections::{HashMap, HashSet};

struct Client {}

enum Interest {}

struct Engine {
    clients: HashMap<u64, Client>,
}
