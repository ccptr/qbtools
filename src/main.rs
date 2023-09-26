pub use quickbooks_ureq;

#[cfg(feature = "interactive")]
use std::ffi::OsStr;
use std::ffi::OsString;

fn main() {
    env_logger::init();
    #[allow(unused_mut)]
    let mut args: Vec<OsString> = std::env::args_os().collect();

    // TODO: add to --help output
    #[cfg(feature = "interactive")]
    let interactive = if let Some(index) = args.iter().position(|arg| arg == OsStr::new("-i")) {
        args.remove(index);
        true
    } else {
        false
    };

    let args = args.into_iter();

    #[cfg(feature = "interactive")]
    if interactive {
        qbtools::main_interactive(args);
    } else {
        #[cfg(feature = "cmdline")]
        qbtools::main_cmdline(args);
    }

    #[cfg(all(feature = "cmdline", not(feature = "interactive")))]
    qbtools::main_cmdline(args);
}
