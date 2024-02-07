// use std::{collections::HashSet, hash::Hash};

// pub struct Pool<T, S>
// where
//     T: Eq + Hash,
//     S: Spawner<T>,
// {
//     data: HashSet<T>,
//     spawner: S,
// }

// impl<T, S> Pool<T, S>
// where
//     T: Eq + Hash,
//     S: Spawner<T>,
// {
//     pub fn new(spawner: S) -> Pool<T, S> {
//         Pool {
//             data: HashSet::new(),
//             spawner: spawner,
//         }
//     }

//     pub fn get(&mut self) -> T {
//         for item in self.data {
//             return item;
//         }
//         let item = self.spawner.spawn();

//         self.data.insert(item);
//         return item;
//     }

//     fn put(&mut self, item: T) {
//         self.data.insert(item);
//     }
// }
