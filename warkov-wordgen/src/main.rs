extern crate warkov;

#[macro_use]
extern crate quicli;

use std::path::PathBuf;

use quicli::prelude::*;
use warkov::MarkovChain;

#[derive(Debug, StructOpt)]
#[structopt(about = "Generate words from a file of existing words", author="")]
struct Cli {

    #[structopt(short="n", long="num", default_value="10")]
    /// The number of new words to generate (unless `min_look` is specified)
    num: usize,

    #[structopt(long="max-look", default_value="3")]
    /// The lookahead to use when generating
    max_look: usize,

    #[structopt(long="min-look")]
    /// If specified, `num` items will be generated from every lookahead value from `min_look` to
    /// `max_look` inclusive.
    min_look: Option<usize>,

    #[structopt(parse(from_os_str))]
    /// Filename to read example words from
    filename: PathBuf,
}

fn generate(markov: &mut MarkovChain<char, impl warkov::Rng>, len: usize) -> String {
    markov.generate_max_look(len).into_iter().collect()
}

main!(|args: Cli| {
    let file = read_file(args.filename)?;
    let mut markov = MarkovChain::new(args.max_look);
    for line in file.lines() {
        markov.train(line.to_lowercase().chars());
    }

    match args.min_look {
        None => {
            for _ in 0..args.num {
                println!("{}", generate(&mut markov, args.max_look))
            }
        },
        Some(min_look) => {
            for len in (min_look..=args.max_look).rev() {
                for _ in 0..args.num {
                    println!("{} {}", len, generate(&mut markov, len))
                }
            }
        },
    }

});
