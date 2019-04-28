extern crate futures;
extern crate nonogram;
extern crate rusoto_core;
extern crate rusoto_dynamodb;

use futures::future::{join_all, Future};
use nonogram::Nonogram;
use rusoto_core::Region;
use rusoto_dynamodb::{
    AttributeValue, BatchWriteItemInput, DynamoDb, DynamoDbClient, PutRequest, WriteRequest,
};
use std::collections::HashMap;
use std::default::Default;
use std::error::Error;

const CHUNK_SIZE: usize = 25;

fn two_dimensional_array_to_attribute_value<N>(array: &nonogram::Array2<N>) -> Vec<AttributeValue>
where
    N: ToString,
{
    array
        .genrows()
        .into_iter()
        .map(|row| AttributeValue {
            l: Some(
                row.iter()
                    .map(|num| AttributeValue {
                        n: Some(num.to_string()),
                        ..Default::default()
                    })
                    .collect::<Vec<AttributeValue>>(),
            ),
            ..Default::default()
        })
        .collect()
}

fn nested_vec_to_attribute_value<N>(nested: &nonogram::Array1<Vec<N>>) -> Vec<AttributeValue>
where
    N: ToString,
{
    nested
        .iter()
        .map(|row| AttributeValue {
            l: Some(
                row.iter()
                    .map(|num| AttributeValue {
                        n: Some(num.to_string()),
                        ..Default::default()
                    })
                    .collect(),
            ),
            ..Default::default()
        })
        .collect()
}

fn puzzle_to_request(puzzle: &Nonogram) -> WriteRequest {
    let mut attribute_map: HashMap<String, AttributeValue> = HashMap::new();

    attribute_map.insert(
        "checksum".to_owned(),
        AttributeValue {
            s: Some(puzzle.generate_checksum().to_string()),
            ..Default::default()
        },
    );

    attribute_map.insert(
        "height".to_owned(),
        AttributeValue {
            n: Some(puzzle.height().to_string()),
            ..Default::default()
        },
    );

    attribute_map.insert(
        "width".to_owned(),
        AttributeValue {
            n: Some(puzzle.width().to_string()),
            ..Default::default()
        },
    );

    attribute_map.insert(
        "row_segments".to_owned(),
        AttributeValue {
            l: Some(nested_vec_to_attribute_value(&puzzle.row_segments)),
            ..Default::default()
        },
    );

    attribute_map.insert(
        "column_segments".to_owned(),
        AttributeValue {
            l: Some(nested_vec_to_attribute_value(&puzzle.column_segments)),
            ..Default::default()
        },
    );

    attribute_map.insert(
        "completed_grid".to_owned(),
        AttributeValue {
            l: Some(two_dimensional_array_to_attribute_value(
                &puzzle.completed_grid,
            )),
            ..Default::default()
        },
    );

    WriteRequest {
        put_request: Some(PutRequest {
            item: attribute_map,
        }),
        ..Default::default()
    }
}

fn get_batch_from_chunk(table: String) -> impl FnMut(Vec<Nonogram>) -> BatchWriteItemInput {
    move |chunk| {
        let mut map: HashMap<String, Vec<WriteRequest>> = HashMap::new();

        map.insert(table.clone(), chunk.iter().map(puzzle_to_request).collect());

        BatchWriteItemInput {
            request_items: map,
            ..Default::default()
        }
    }
}

pub fn push_to_dynamo(puzzles: &[Nonogram], table: &str) -> Result<(), Box<Error>> {
    let client = DynamoDbClient::new(Region::UsEast2);
    let chunks = puzzles
        .chunks(CHUNK_SIZE)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<Vec<Nonogram>>>();
    let batches: Vec<
        rusoto_core::RusotoFuture<
            rusoto_dynamodb::BatchWriteItemOutput,
            rusoto_dynamodb::BatchWriteItemError,
        >,
    > = chunks
        .iter()
        .cloned()
        .map(get_batch_from_chunk(table.to_owned()))
        .map(move |batch| client.batch_write_item(batch))
        .collect();

    tokio::run(join_all(batches).map(|_| ()).map_err(|e| panic!(e)));

    Ok(())
}
