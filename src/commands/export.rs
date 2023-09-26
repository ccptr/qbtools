use super::{get_desired_array, to_output_path, we_do_a_bit_of_logging, CommandError};
use crate::args::{ExportCustomerArgs, OutputFormat};

use quickbooks_ureq::config::QueryConfig;

use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct ExportArgs {
    pub format: Option<OutputFormat>,
    pub output_path: Option<PathBuf>,
    pub pretty: bool,
    pub quiet: bool,
    pub verbose: bool,
}

pub fn customers(
    args: &ExportArgs,
    customer_args: &ExportCustomerArgs,
) -> Result<(), CommandError> {
    let options = QueryConfig {
        r#where: customer_args.r#where.as_deref(),
        ..Default::default()
    };

    let customers = get_desired_array(args.quiet, "Customer", &options)?;
    let customers = customers
        .as_array()
        .expect("to be guaranteed by the QB API");

    we_do_a_bit_of_logging(customers, "customers");

    Ok(to_output_path(
        customers,
        &args.output_path,
        &args.format.clone().unwrap_or_default(),
        args.pretty,
    )?)
}

pub fn items(args: &ExportArgs) -> Result<(), CommandError> {
    let items = get_desired_array(args.quiet, "Item", &Default::default())?;
    let items = items.as_array().expect("to be guaranteed by the QB API");

    we_do_a_bit_of_logging(items, "items");

    Ok(to_output_path(
        items,
        &args.output_path,
        &args.format.clone().unwrap_or_default(),
        args.pretty,
    )?)
}
