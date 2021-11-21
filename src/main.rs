use clap::{Arg, App, SubCommand};

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
}

fn process_command(ArgMatches matches) -> Result<(), ()> {
  if let Some(matches) = matches.subcommand_matches("list") {
    return commands::list::list(matches);
  }
}
