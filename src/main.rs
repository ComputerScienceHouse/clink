use clap::{command, Arg, ArgMatches, Command};
use std::process::ExitCode;

pub mod api;
pub mod commands;

mod ui;

fn main() -> ExitCode {
  let matches = command!("clink")
    .about("Drops drinks from CSH vending machines")
    .subcommand(
      Command::new("list").about("Display available slots").arg(
        Arg::new("machine")
          .index(1)
          .help("Which machine should be listed?")
          .required(false),
      ),
    )
    .subcommand(Command::new("credits").about("Prints the number of credits in your account"))
    .subcommand(Command::new("token").about("Generates an API token (Plumbing)"))
    .subcommand(
      Command::new("drop")
        .about("Drops a drink")
        .arg(
          Arg::new("machine")
            .index(1)
            .help("Machine to drop from")
            .required(true),
        )
        .arg(
          Arg::new("slot")
            .index(2)
            .help("Slot to drop from")
            .required(true),
        ),
    )
    .get_matches();
  let result = process_command(matches);
  match result {
    Ok(_) => 0,
    Err(err) => {
      eprintln!("Error: {}", err);
      1
    }
  }
  .into()
}

fn process_command(matches: ArgMatches) -> Result<(), api::APIError> {
  let mut api = api::API::new();
  if let Some(matches) = matches.subcommand_matches("list") {
    commands::list::list(matches, &mut api)
  } else if let Some(matches) = matches.subcommand_matches("drop") {
    commands::drop::drop(matches, &mut api)
  } else if let Some(matches) = matches.subcommand_matches("credits") {
    commands::credits::credits(matches, &mut api)
  } else if let Some(matches) = matches.subcommand_matches("token") {
    commands::token::token(matches, &mut api)
  } else {
    ui::ui_common::launch(api)
  }
}
