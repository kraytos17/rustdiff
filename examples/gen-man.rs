//! Generate the `rustdiff` man page.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example gen-man > rustdiff.1
//! ```

use clap::CommandFactory;
use clap_mangen::Man;
use rustdiff::cli::Cli;

fn main() {
    let man = Man::new(Cli::command());
    man.render(&mut std::io::stdout()).unwrap();
}
