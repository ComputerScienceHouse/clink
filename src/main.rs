use clap::{Arg, App, SubCommand, ArgMatches};

pub mod api;
pub mod commands;

fn main() {
  let matches = App::new("CLI Drink")
    .version("1.0.0")
    .author("Mary Strodl <mstrodl@csh.rit.edu>")
    .about("Drops drinks from CSH vending machines")
    .arg(Arg::with_name("machine")
         .short("m")
         .long("machine")
         .value_name("NAME")
         .help("Selects machine to perform operation on")
         .takes_value(true))
    .subcommand(SubCommand::with_name("list")
                .about("Display available slots")).get_matches();
  match process_command(matches) {
    Ok(_) => {}
    Err(err) => println!("{}", err),
  }
}

fn process_command(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
  let mut api = api::API::new();
  if let Some(matches) = matches.subcommand_matches("list") {
    return commands::list::list(matches, &mut api);
  } else {
    // TODO: Call out to willard's GUI!
    panic!("cry about it bitch");
  }
}
