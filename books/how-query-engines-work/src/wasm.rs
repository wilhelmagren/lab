use bytes::Bytes;
use wasm_bindgen::prelude::*;

use arrow::{
    array::RecordBatch,
    util::pretty::pretty_format_batches,
};

use crate::{
    context::SessionContext,
    logical::expr::{avg, col, max, min},
};

#[wasm_bindgen]
pub fn run_demo(parquet: &[u8]) -> Result<String, JsValue> {
    console_error_panic_hook::set_once();

    let ctx = SessionContext::new();

    let df = ctx
        .parquet_bytes(
            "weather_stations",
            Bytes::copy_from_slice(parquet),
        )
        .agg(
            vec![col("station_name")],
            vec![
                min(col("measurement")),
                max(col("measurement")),
                avg(col("measurement")),
            ],
        );

    let results: Vec<RecordBatch> =
        ctx.execute(&df).collect();

    pretty_format_batches(&results)
        .map(|x| x.to_string())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
