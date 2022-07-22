use clap::{Parser, Subcommand};
use std::process::ExitCode;

pub mod api;
pub mod commands;

mod ui;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
  #[clap(subcommand)]
  command: Option<Subcommands>,
  /// API base URL to use
  #[clap(value_parser, default_value = "https://drink.csh.rit.edu", long)]
  api: String,
}

#[derive(Subcommand)]
enum Subcommands {
  /// Drops a drink
  Drop {
    /// Machine to drop from
    #[clap(value_parser)]
    machine: String,
    /// Slot to drop from
    #[clap(value_parser)]
    slot: u8,
  },
  /// Lists available drinks
  List {
    /// Machine whose contents should be shown (if not specified, all will be shown)
    #[clap(value_parser)]
    machine: Option<String>,
  },
  /// Prints the number of credits in your account
  Credits,
  /// Generates an API token (Plumbing)
  Token,
}

use crate::Subcommands::*;

fn main() -> ExitCode {
  let cli = Cli::parse();
  let result = process_command(cli);
  match result {
    Ok(_) => 0,
    Err(err) => {
      eprintln!("Error: {}", err);
      1
    }
  }
  .into()
}

fn process_command(cli: Cli) -> Result<(), api::APIError> {
  let mut api = api::API::new(cli.api);
  match cli.command {
    Some(Drop { machine, slot }) => commands::drop::drop(&mut api, machine, slot),
    Some(List { machine }) => commands::list::list(&mut api, machine),
    Some(Credits) => commands::credits::credits(&mut api),
    Some(Token) => commands::token::token(&mut api),
    None => ui::ui_common::launch(api),
  }
}
