extern crate nonogram;
extern crate postgres;
extern crate serde_json;

use nonogram::Nonogram;
use postgres::{Connection, TlsMode};

use std::error::Error;

pub fn push_to_postgres(puzzles: &[Nonogram], addr: &str) -> Result<(), Box<Error>> {
    let conn = Connection::connect(addr, TlsMode::None)?;

    println!("Creating the table if it doesn't already exist.");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS puzzle(
            id integer GENERATED ALWAYS AS IDENTITY,
            height integer NOT NULL CHECK (height > 0 AND height % 5 = 0),
            width integer NOT NULL CHECK (width > 0 AND width % 5 = 0),
            hash bytea NOT NULL,
            row_segments jsonb NOT NULL,
            column_segments jsonb NOT NULL,
            completed_grid jsonb NOT NULL,
            created timestamp without time zone default (now() at time zone 'utc'),
            PRIMARY KEY (id),
            UNIQUE (hash)
        );",
        &[],
    )?;

    println!("Inserting new puzzles into the table");
    let insert_stmt = conn.prepare(
        "INSERT INTO puzzle (
            height,
            width,
            hash,
            row_segments,
            column_segments,
            completed_grid
        ) VALUES ($1, $2, $3, $4, $5, $6)",
    )?;

    for p in puzzles {
        let height = p.height() as i32;
        let width = p.width() as i32;
        let hash = p.generate_checksum().to_be_bytes().to_vec();
        let row_segments =
            serde_json::to_value(p.row_segments.iter().cloned().collect::<Vec<Vec<usize>>>())?;
        let column_segments = serde_json::to_value(
            p.column_segments
                .iter()
                .cloned()
                .collect::<Vec<Vec<usize>>>(),
        )?;
        let completed_grid = serde_json::to_value(
            p.completed_grid
                .genrows()
                .into_iter()
                .map(|row| row.iter().cloned().collect())
                .collect::<Vec<Vec<u8>>>(),
        )?;
        insert_stmt.execute(&[
            &height,
            &width,
            &hash,
            &row_segments,
            &column_segments,
            &completed_grid,
        ])?;
    }

    Ok(())
}
