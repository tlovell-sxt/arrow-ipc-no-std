use std::io::{read_to_string, stdin};

use arrow_ipc_no_std::{finish, single_batch_stream_parser, SingleBatchStream};

fn main() {
    let hex_encoded = read_to_string(stdin()).unwrap();

    let hex_minus_prefix = hex_encoded.strip_prefix("0x").unwrap();

    let bytes = hex::decode(hex_minus_prefix).unwrap();

    let SingleBatchStream {
        schema_message,
        batch_message,
    } = finish(single_batch_stream_parser)(&bytes).unwrap();

    let column_count = schema_message
        .header_as_schema()
        .unwrap()
        .fields()
        .unwrap()
        .len();

    let row_count = batch_message.header_as_record_batch().unwrap().length();

    dbg!(column_count, row_count);
}
