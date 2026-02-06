use std::io::{read_to_string, stdin};

use fbs::Message::{root_as_message, Message};

const CONTINUATION_MARKER: [u8; 4] = [255, 255, 255, 255];

const END_OF_STREAM_MARKER: [u8; 8] = [255, 255, 255, 255, 0, 0, 0, 0];

struct Continuation;

fn continuation_parser(input: &[u8]) -> Result<(&[u8], Continuation), ()> {
    let (continuation_bytes, input) = input.split_at_checked(4).ok_or(())?;

    if continuation_bytes == CONTINUATION_MARKER {
        Ok((input, Continuation))
    } else {
        Err(())
    }
}

fn u32_parser(input: &[u8]) -> Result<(&[u8], u32), ()> {
    let (u32_bytes, input) = input.split_at_checked(4).ok_or(())?;

    let u32_array: [u8; 4] = u32_bytes.try_into().map_err(|_| ())?;

    Ok((input, u32::from_le_bytes(u32_array)))
}

fn message_parser<'a>(input: &'a [u8]) -> Result<(&'a [u8], Message<'a>), ()> {
    let (input, _) = continuation_parser(input)?;
    let (input, metadata_len) = u32_parser(input)?;
    let message = root_as_message(input).map_err(|_| ())?;

    let (_, input) = input
        .split_at_checked(metadata_len as usize + message.bodyLength() as usize)
        .ok_or(())?;
    Ok((input, message))
}

struct EndOfStream;

fn end_of_stream_parser(input: &[u8]) -> Result<(&[u8], EndOfStream), ()> {
    let (end_of_stream_bytes, input) = input.split_at_checked(8).ok_or(())?;

    if end_of_stream_bytes == END_OF_STREAM_MARKER {
        Ok((input, EndOfStream))
    } else {
        Err(())
    }
}

fn maybe_parser<'a, T>(
    f: impl Fn(&'a [u8]) -> Result<(&'a [u8], T), ()>,
) -> impl Fn(&'a [u8]) -> Result<(&'a [u8], Option<T>), ()> {
    move |input| {
        f(input)
            .map(|(input, t)| (input, Some(t)))
            .or_else(|_| Ok((input, None)))
    }
}

struct SingleBatchStream<'a> {
    schema_message: Message<'a>,
    batch_message: Message<'a>,
}

fn single_batch_stream_parser<'a>(
    input: &'a [u8],
) -> Result<(&'a [u8], SingleBatchStream<'a>), ()> {
    let (input, schema_message) = message_parser(input)?;
    let (input, batch_message) = message_parser(input)?;
    let (input, _) = maybe_parser(end_of_stream_parser)(input)?;

    Ok((
        input,
        SingleBatchStream {
            schema_message,
            batch_message,
        },
    ))
}

fn finish<'a, T>(
    f: impl Fn(&'a [u8]) -> Result<(&'a [u8], T), ()>,
) -> impl Fn(&'a [u8]) -> Result<T, ()> {
    move |input| {
        let (input, t) = f(input)?;

        if input.is_empty() {
            Ok(t)
        } else {
            Err(())
        }
    }
}

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
