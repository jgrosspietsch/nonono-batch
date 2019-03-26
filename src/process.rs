extern crate hashbrown;
extern crate nonogram;
extern crate num_cpus;
extern crate pbr;

use hashbrown::HashSet;
use nonogram::Nonogram;
use pbr::ProgressBar;
use std::sync::{Arc, Mutex};
use std::thread;

fn generate_dimension_puzzles(dimension: (usize, usize), number: usize) -> Vec<Nonogram> {
    println!("Height: {}, Width: {}", dimension.0, dimension.1);

    let results = Arc::new(Mutex::new((
        HashSet::new(),
        ProgressBar::new(number as u64),
    )));
    let mut handles = vec![];

    for _ in 0..num_cpus::get() {
        let results_clone = Arc::clone(&results);
        let handle = thread::spawn(move || loop {
            let puzzle = loop {
                let current = Nonogram::generate(dimension.1, dimension.0);

                if current.solvable() {
                    break current;
                }
            };

            let mut progress = results_clone.lock().unwrap();

            if !progress.0.contains(&puzzle) && progress.0.len() < number {
                progress.0.insert(puzzle);
                progress.1.inc();
            }

            if progress.0.len() >= number {
                break;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let puzzles = &results.lock().unwrap().0;
    puzzles.iter().cloned().collect()
}

pub fn build_puzzles(dimensions: Vec<(usize, usize)>, number: usize) -> Vec<Nonogram> {
    dimensions
        .iter()
        .cloned()
        .map(|dim| generate_dimension_puzzles(dim, number))
        .flatten()
        .collect()
}
