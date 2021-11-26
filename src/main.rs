use clap::{App, Arg, ArgMatches, SubCommand};

pub mod api;
pub mod commands;

mod ui;

fn main() {
  let matches = App::new("CLI Drink")
    .version("1.0.0")
    .author("Mary Strodl <mstrodl@csh.rit.edu>")
    .about("Drops drinks from CSH vending machines")
    .subcommand(
      SubCommand::with_name("list")
        .about("Display available slots")
        .arg(
          Arg::with_name("machine")
            .index(1)
            .help("Which machine should be listed?")
            .required(false),
        ),
    )
    .subcommand(
      SubCommand::with_name("drop")
        .about("Drops a drink")
        .arg(
          Arg::with_name("machine")
            .index(1)
            .help("Machine to drop from")
            .required(true),
        )
        .arg(
          Arg::with_name("slot")
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
    cli();
    return Ok(());
  }
}

fn cli() {

  let mut api = api::API::new(); // Cheetos.
  ui::ui_common::launch();
  ui::machine::pick_machine(&mut api);
  ui::ui_common::end();
}
