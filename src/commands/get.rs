use crate::config::get_authorized_qb;

use super::{CommandError, OutputFormat};

use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct GetArgs {
    pub format: Option<OutputFormat>,
    pub output_path: Option<PathBuf>,
    pub quiet: bool,

    pub id: String,
}

pub fn customer(args: &GetArgs) -> Result<(), CommandError> {
    let qb = get_authorized_qb(args.quiet)?;

    let customer = qb.read("customer", "27")?;

    println!("{:#?}", customer);

    Ok(())
}
