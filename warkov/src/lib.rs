extern crate rand;

use std::collections::{HashMap, BTreeMap};
use std::hash::Hash;
use std::fmt::Debug;
pub use rand::Rng;

/// A Markov Chain.
#[derive(Default)]
pub struct MarkovChain<T, R>
    where T: Hash + Eq + Clone + Default + Ord + Debug,
          R: Rng,
{
    size: usize,
    rng: R,
    stages: HashMap<Vec<Option<T>>, (usize, BTreeMap<Option<T>, usize>)>,
    alphabet: (usize, BTreeMap<T, usize>),
}

impl<T> MarkovChain<T, rand::ThreadRng>
    where T: Hash + Eq + Clone + Default + Ord + Debug,
{
    /// Creates a MarkovChain with the max look a head size.
    /// Uses the stock thread local random number generator
    ///
    /// # Panics
    /// If size is 0
    pub fn new(size: usize) -> Self {
        MarkovChain::new_with_rng(size, rand::thread_rng())
    }
}

impl<T, R> MarkovChain<T, R>
    where T: Hash + Eq + Clone + Default + Ord + Debug,
          R: Rng,
{
    /// Creates a MarkovChain with the specified random number generator, and max look a head size
    ///
    /// # Panics
    /// If size is 0
    pub fn new_with_rng(size: usize, rng: R) -> Self {
        assert!(size > 0);
        MarkovChain{ size, rng: rng,  stages: HashMap::new(), alphabet: (0, BTreeMap::new()) }
    }

    /// Change the random number generation for this object to `rng`.
    pub fn set_rng(&mut self, rng: R) {
        self.rng = rng
    }

    fn record_occurance(&mut self, mut stage: &[Option<T>], next: Option<T>) {
        while !stage.is_empty() {
            let stage_stat = self.stages.entry(stage.to_vec()).or_default();
            stage_stat.0 += 1;
            *stage_stat.1.entry(next.clone()).or_default() += 1;

            stage = &stage[1..];
        }
    }

    /// Teach the markov chain `term`.
    pub fn train(&mut self, term: impl Iterator<Item=T>) {
        let term: Vec<T> = term.into_iter().collect();
        self.alphabet.0 += term.len();
        for t in term.iter() {
            *self.alphabet.1.entry(t.clone()).or_default() += 1;
        }
        let mut term: Vec<Option<T>> = term.into_iter().map(|s| Some(s)).collect();
        term.insert(0, None);
        term.push(None);

        for idx in 1..term.len() {

            for len in 1..(self.size+1) {
                if len <= idx {
                    self.record_occurance(&term[idx-len..idx], term[idx].clone());
                }
            }
        }

    }

    /// Generates a term using the max look ahead
    pub fn generate(&mut self) -> Vec<T> {
        let curr_size = self.size;
        self.generate_max_look(curr_size)
    }

    pub fn generate_max_look(&mut self, max_lookbehind: usize) -> Vec<T> {
        assert!(max_lookbehind >= 1 && max_lookbehind <= self.size);

        let mut result = Vec::new();
        let mut curr: Vec<Option<T>> = vec![None];
        let mut next: Option<T>;


        loop {
            loop {
                match self.stages.get(&curr) {
                    None => {
                        if curr.len() == 1 {
                            next = Some(weighted_choice(&mut self.rng, &self.alphabet));
                            break;
                        } else {
                            curr.remove(0);
                            continue;
                        }
                    },
                    Some(stats) => {
                        next = weighted_choice(&mut self.rng, stats);
                        break;
                    }
                }

            }

            if next == None {
                // we're at end
                break;
            } else {
                curr.push(next.clone());
                while curr.len() > max_lookbehind {
                    curr.remove(0);
                }
                result.push(next.clone().unwrap());
            }
        }

        result
    }

}

fn weighted_choice<'a, T: Debug+Clone+Default, R: Rng>(rng: &mut R, options: &'a (usize, BTreeMap<T, usize>)) -> T {
    debug_assert_eq!(options.0, options.1.values().sum());
    let random_number = rng.gen_range(0, options.0);
    let mut curr_value = 0;
    let mut last_key = &T::default();
    for (key, value) in options.1.iter() {
        last_key = key;
        if random_number >= curr_value && random_number < curr_value+value {
            return key.clone();
        }
        curr_value += value;
    }

    return last_key.to_owned();
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn has_key<R: Rng>(mc: &MarkovChain<char, R>, k: &str) -> bool {
        let k: Vec<Option<char>> = k.chars().map(|s| Some(s.clone())).collect();
        mc.stages.contains_key(&k)
    }

    fn has_key_w_none_prefix<R: Rng>(mc: &MarkovChain<char, R>, k: &str) -> bool {
        let mut k: Vec<Option<char>> = k.chars().map(|s| Some(s.clone())).collect();
        k.insert(0, None);
        mc.stages.contains_key(&k)
    }

    fn has_key_w_none_predict<R: Rng>(mc: &MarkovChain<char, R>, k: &str) -> bool {
        let k: Vec<Option<char>> = k.chars().map(|s| Some(s.clone())).collect();
        mc.stages.get(&k).map(|stats| stats.1.contains_key(&None)).unwrap_or(false)
    }

    #[test]
    fn simple1() {
        let mut mc = MarkovChain::new(2);

        mc.train("abc".chars());
        //assert_eq!(mc.stages.len(), 4, "{:?}", mc.stages);
        assert!(mc.stages.contains_key(&vec![None]));
        assert!(has_key_w_none_prefix(&mc, "a"));
        assert!(!has_key_w_none_prefix(&mc, "ab"));
        assert!(!has_key_w_none_prefix(&mc, "abc"));

        assert!(has_key_w_none_predict(&mc, "c"));
        assert!(has_key_w_none_predict(&mc, "bc"));
        assert!(!has_key_w_none_predict(&mc, "abc"));

        assert!(has_key(&mc, "ab"));
        assert!(has_key(&mc, "ab"));
        assert!(has_key(&mc, "bc"));
        assert!(!has_key(&mc, "ac"));

        assert!(has_key(&mc, "a"));
        assert!(has_key(&mc, "b"));
        assert!(has_key(&mc, "c"));
        assert!(!has_key(&mc, "d"));

    }

    #[test]
    fn simple2() {
        let mut mc = MarkovChain::new(3);

        mc.train("abc".chars());
        assert!(mc.stages.contains_key(&vec![None]));
        assert!(has_key_w_none_prefix(&mc, "a"));
        assert!(has_key_w_none_prefix(&mc, "ab"));
        assert!(!has_key_w_none_prefix(&mc, "abc"));

        assert!(has_key(&mc, "abc"));

        assert!(has_key(&mc, "ab"));
        assert!(has_key(&mc, "ab"));
        assert!(has_key(&mc, "bc"));
        assert!(!has_key(&mc, "ac"));

        assert!(has_key(&mc, "a"));
        assert!(has_key(&mc, "b"));
        assert!(has_key(&mc, "c"));
        assert!(!has_key(&mc, "d"));

        mc.train("rust".chars());
        assert!(mc.stages.contains_key(&vec![None]));
        assert!(has_key_w_none_prefix(&mc, "r"));
        assert!(has_key_w_none_prefix(&mc, "ru"));
        assert!(!has_key_w_none_prefix(&mc, "rus"));
        assert!(!has_key_w_none_prefix(&mc, "rust"));
        assert!(has_key(&mc, "r"));
        assert!(has_key(&mc, "u"));
        assert!(has_key(&mc, "s"));
        assert!(has_key(&mc, "t"));
        assert!(has_key(&mc, "ru"));
        assert!(has_key(&mc, "us"));
        assert!(has_key(&mc, "st"));
        assert!(has_key(&mc, "rus"));
        assert!(has_key(&mc, "ust"));
        assert!(!has_key(&mc, "rust"));

    }


    fn easy_rng() -> impl Rng {
        rand::prng::XorShiftRng::from_seed([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16])
    }

    #[test]
    fn weighted_choice1() {
        let mut rng = easy_rng();
        let mut data = BTreeMap::new();
        data.insert(Some('a'), 1);
        data.insert(Some('b'), 2);
        let data = (data.values().sum(), data);
        let mut stats: HashMap<Option<char>, usize> = HashMap::new();

        for _ in 0..1000 {
            let choice = weighted_choice(&mut rng, &data);
            *stats.entry(choice).or_default() += 1;
        }
        assert_eq!(stats[&Some('a')], 328);
        assert_eq!(stats[&Some('b')], 672);
        
    }

    #[test]
    fn weighted_choice2() {
        let mut rng = easy_rng();
        let mut data = BTreeMap::new();
        data.insert(Some('a'), 1);
        data.insert(Some('b'), 200);
        data.insert(Some('c'), 200);
        data.insert(Some('d'), 200);
        let data = (data.values().sum(), data);
        let mut stats: HashMap<Option<char>, usize> = HashMap::new();

        for _ in 0..10_000 {
            let choice = weighted_choice(&mut rng, &data);
            *stats.entry(choice).or_default() += 1;
        }
        assert_eq!(stats[&Some('a')], 15);
        assert_eq!(stats[&Some('b')], 3293);
        assert_eq!(stats[&Some('c')], 3405);
        assert_eq!(stats[&Some('d')], 3287);
        
    }

    fn prediction_result<R: Rng>(mc: &mut MarkovChain<char, R>) -> String {
        mc.generate().into_iter().map(|c| c.to_string()).collect::<Vec<String>>().join("")
    }

    fn prediction_result_size<R: Rng>(mc: &mut MarkovChain<char, R>, size: usize) -> String {
        mc.generate_max_look(size).into_iter().map(|c| c.to_string()).collect::<Vec<String>>().join("")
    }

    #[test]
    fn predict1() {
        let mut mc = MarkovChain::new_with_rng(2, easy_rng());

        mc.train("abc".chars());
        mc.train("bbc".chars());
        mc.train("acb".chars());

        assert_eq!(prediction_result(&mut mc), "abc");
        assert_eq!(prediction_result(&mut mc), "bbc");
        assert_eq!(prediction_result(&mut mc), "bbc");

        assert_eq!(prediction_result_size(&mut mc, 1), "abbbc");
        assert_eq!(prediction_result_size(&mut mc, 1), "bc");
        assert_eq!(prediction_result_size(&mut mc, 1), "acbc");
        assert_eq!(prediction_result_size(&mut mc, 1), "ac");

    }
}
