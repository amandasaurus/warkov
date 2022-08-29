extern crate anyhow;
extern crate clap;
extern crate warkov;
use clap::Parser;

use std::path::PathBuf;

use anyhow::Result;

use warkov::MarkovChain;

#[derive(Parser, Debug)]
#[clap(
    author,
    version,
    about = "Generate words from a file of existing words"
)]
struct Args {
    #[clap(short, long, default_value = "10", value_name="N")]
    /// The number of new words to generate (unless `min-look` is specified)
    num: usize,

    #[clap(long = "max-look", default_value = "3", value_name="N")]
    /// The max lookbehind to use when generating
    max_look: usize,

    #[clap(long = "min-look", value_name="N")]
    /// If specified, `num` items will be generated from every lookbehind value from `min-look` to
    /// `max-look` inclusive.
    min_look: Option<usize>,

    #[clap(parse(from_os_str))]
    /// Filename to read example words from, e.g. /usr/share/dict/words
    filename: PathBuf,
}

fn generate(markov: &mut MarkovChain<char, impl warkov::Rng>, len: usize) -> String {
    markov.generate_max_look(len).into_iter().collect()
}

fn main() -> Result<()> {
    let args = Args::parse();
    let file = std::fs::read_to_string(args.filename)?;
    let mut markov = MarkovChain::new(args.max_look);
    for line in file.lines() {
        markov.train(line.to_lowercase().chars());
    }

    match args.min_look {
        None => {
            for _ in 0..args.num {
                println!("{}", generate(&mut markov, args.max_look))
            }
        }
        Some(min_look) => {
            for len in (min_look..=args.max_look).rev() {
                for _ in 0..args.num {
                    println!("{} {}", len, generate(&mut markov, len))
                }
            }
        }
    }

    Ok(())
}
