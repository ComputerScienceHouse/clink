use clap::{command, Arg, ArgMatches, Command};

pub mod api;
pub mod commands;

mod ui;

fn main() {
  let matches = command!("CLI Drink")
    .about("Drops drinks from CSH vending machines")
    .subcommand(
      Command::new("list")
        .about("Display available slots")
        .arg(
          Arg::new("machine")
            .index(1)
            .help("Which machine should be listed?")
            .required(false),
        ),
    )
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
  match process_command(matches) {
    Ok(_) => {}
    Err(err) => println!("{}", err),
  }
}

fn process_command(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
  let mut api = api::API::new();
  if let Some(matches) = matches.subcommand_matches("list") {
    return commands::list::list(matches, &mut api);
  } else if let Some(matches) = matches.subcommand_matches("drop") {
    return commands::drop::drop(matches, &mut api);
  } else {
    cli(&mut api);
    return Ok(());
  }
}

fn cli(api: &mut api::API) {
  ui::ui_common::launch();
  match ui::machine::pick_machine(api) {
    Ok(_) => {
      ui::ui_common::end();
    }
    Err(err) => {
      ui::ui_common::end();
      eprintln!("{}", err);
    }
  };
}
