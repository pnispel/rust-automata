#![feature(std_misc)]

use std::fmt::Display;

pub mod dfa;
pub mod nfa;

pub use nfa::{NFA, Transition};
pub use dfa::DFA;

pub trait Automaton {
    type State;
    type Alphabet;

    fn run(&self, Vec<Self::Alphabet>) -> Option<Vec<Self::State>>;
}

pub mod automaton {
    #[macro_export]
    macro_rules! map {
        ($($key:expr => $val:expr),*) => ({
            let mut h = ::std::collections::HashMap::new();
            $(h.insert($key, $val);)*
            h
        })
    }

    #[macro_export]
    macro_rules! set {
        ($($elem:expr),*) => ({
            let mut s = ::std::collections::HashSet::new();
            $(s.insert($elem);)*
            s
        })
    }
}
