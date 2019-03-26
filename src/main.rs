extern crate clap;
extern crate serde_json;

mod db;
mod file;
mod process;

use clap::{App, Arg};
use process::build_puzzles;
use std::error::Error;

fn is_number(n: String) -> Result<(), String> {
    if n.parse::<usize>().is_ok() {
        return Ok(());
    }
    Err(String::from(
        "Number of puzzles, heights, and widths arguments must be numbers",
    ))
}

fn permute_dimensions(heights: &[usize], widths: &[usize]) -> Vec<(usize, usize)> {
    heights
        .iter()
        .cloned()
        .map(|h| {
            widths
                .iter()
                .cloned()
                .map(|w| (h, w))
                .collect::<Vec<(usize, usize)>>()
        })
        .flatten()
        .collect()
}

fn main() -> Result<(), Box<Error>> {
    let matches = App::new("Nonogram Batch Runner")
        .version("0.1.0")
        .author("Joe Grosspietsch <joe.grosspietsch@gmail.com>")
        .about("Builds a number of nonograms of the given dimensions, then pushes it to PostgreSQL and/or a file")
        .arg(Arg::with_name("number")
            .help("Number of puzzles to create for each dimension permutation")
            .long("number")
            .short("n")
            .takes_value(true)
            .multiple(false)
            .default_value("10000")
        )
        .arg(Arg::with_name("heights")
            .help("One or more heights that are used for building ")
            .long("heights")
            .short("h")
            .takes_value(true)
            .multiple(true)
            .default_value("5")
            .validator(is_number)
            .possible_values(&["5", "10", "15", "20"])
        )
        .arg(Arg::with_name("widths")
            .help("One or more widths that are used for building ")
            .long("widths")
            .short("w")
            .takes_value(true)
            .multiple(true)
            .default_value("5")
            .validator(is_number)
            .possible_values(&["5", "10", "15", "20"])
        )
        .arg(Arg::with_name("outfile")
            .help("Path for output file")
            .long("out-file")
            .short("o")
            .takes_value(true)
            .multiple(false)
            .required(false)
        )
        .arg(Arg::with_name("connection")
            .help("PostgreSQL connection address")
            .long("conn")
            .short("c")
            .takes_value(true)
            .multiple(false)
            .required(false)
        )
        .get_matches();

    let number: usize = match matches.value_of("number").unwrap().parse() {
        Ok(n) => n,
        Err(_) => {
            println!("Desired number of generated puzzles is invalid.");
            println!(
                "Number value passed in \"{}\"",
                matches.value_of("number").unwrap()
            );
            10_000
        }
    };

    let mut heights: Vec<usize> = matches
        .values_of("heights")
        .unwrap()
        .map(|n| n.parse::<usize>().unwrap())
        .collect();
    let mut widths: Vec<usize> = matches
        .values_of("widths")
        .unwrap()
        .map(|n| n.parse::<usize>().unwrap())
        .collect();

    heights.dedup();
    widths.dedup();

    let dimensions = permute_dimensions(&heights, &widths);

    let outfile = matches.value_of("outfile");
    let addr = matches.value_of("connection");

    println!("Number of puzzles: {}", number);
    println!("Dimension permutations: {:?}", dimensions);
    if let Some(path) = outfile {
        println!("Outfile path: {}", path)
    }
    if let Some(addr) = addr {
        println!("PostgreSQL addr: {}", addr);
    }

    let puzzles = build_puzzles(dimensions, number);

    println!("Number of puzzles generated! {}", puzzles.len());

    if outfile.is_some() {
        file::write_to_file(&puzzles, outfile.unwrap())?;
    }

    if addr.is_some() {
        db::push_to_postgres(&puzzles, addr.unwrap())?;
    }

    Ok(())
}
