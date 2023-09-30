use std::{fs, process};
use clap::Parser;
use combat_sim::{CombatSimulator, CSBig};
use combat_sim::serialization::PlayerDescription;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    player_file: String,

    #[arg(long = "ac")]
    armor_class: isize,

    #[arg(short, long, default_value_t = 1)]
    num_rounds: u8,
}


fn main() {
    let args = Args::parse();

    println!("Player file: {}", args.player_file);
    println!("armor class: {}", args.armor_class);
    println!("num rounds: {}", args.num_rounds);

    let player_file = fs::File::open(args.player_file).unwrap_or_else(|err| {
        eprintln!("Problem opening player file: {}", err);
        process::exit(1);
    });
    let player_desc: PlayerDescription = serde_json::from_reader(player_file).unwrap_or_else(|err| {
        eprintln!("Problem serializing player: {}", err);
        process::exit(2);
    });
    let character = player_desc.get_player().unwrap_or_else(|err| {
        eprintln!("Problem creating character: {:?}", err);
        process::exit(3);
    });

    let combat_sim: CSBig = CombatSimulator::dmg_sponge(character, player_desc.get_str_bldr().clone(), args.armor_class, args.num_rounds).unwrap_or_else(|err| {
        eprintln!("Problem running combat simulation: {:?}", err);
        process::exit(4);
    });

    combat_sim.describe();
}
