use std::{path::PathBuf, process::exit};

use crate::{args::OutputFormat, config::get_authorized_qb};

use super::to_output_path;

#[derive(Debug, PartialEq)]
pub struct ExportItemsArgs {
    pub quiet: bool,
    pub verbose: bool,
    pub output_path: Option<PathBuf>,
    pub format: OutputFormat,
}

pub fn items(args: &ExportItemsArgs) {
    let qb = get_authorized_qb(args.quiet);

    match qb.query_items(Default::default()) {
        Ok(response) => {
            let response: serde_json::Value = response
                .into_json()
                .expect("failed to serialize response into a serde_json::Value");

            let items = response
                .get("QueryResponse")
                .expect("unexpected response reveived from QuickBooks API")
                .get("Item")
                .expect("unexpected response reveived from QuickBooks API");

            let items = items
                .as_array()
                .expect("unexpected response reveived from QuickBooks API");

            if args.verbose {
                println!("Number of items: {}", items.len());
            }

            if items.len() >= 1000 {
                log::error!("Number of items is equal to 1,000; if you have more than 1,000 items, not all of them have been written to --output-path");
            }

            to_output_path(items, &args.output_path, &args.format)
        }
        Err(err) => {
            log::error!("failed to fetch items (products & services): {}", err);
            exit(1);
        }
    };
}
