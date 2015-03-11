use std::collections::{HashSet, HashMap};
use std::hash::Hash;

pub trait Automaton {
    type State;
    type Alphabet;

    fn run(&self, Vec<Self::Alphabet>) -> bool;
}

pub struct DFA<S = usize, I = char> {
    start: S,
    accept_states: HashSet<S>,
    transitions: HashMap<(S, I), S>
}

impl<S, I> DFA<S, I> {
    pub fn new(start: S, accept_states: HashSet<S>, transitions: HashMap<(S, I), S>) -> DFA<S, I> {
        DFA { start: start, accept_states: accept_states, transitions: transitions }
    }
}

impl<S, I> Automaton for DFA<S, I> where S: Hash + Eq + Copy, I: Hash + Eq + Copy {
    type State = S;
    type Alphabet = I;

    fn run(&self, s: Vec<I>) -> bool {
        let mut cur_state = self.start;
        for c in s {
            match self.transitions.get(&(cur_state, c)) {
                Some(s) => cur_state = *s,
                None => return false
            }
        }
        self.accept_states.contains(&cur_state)
    }
}

#[cfg(test)]
mod test {
    use dfa::{DFA, Automaton};

    macro_rules! set {
        ($($elem:expr),*) => ({
            let mut s = ::std::collections::HashSet::new();
            $(s.insert($elem);)*
            s
        })
    }

    macro_rules! map {
        ($($key:expr => $val:expr),*) => ({
            let mut h = ::std::collections::HashMap::new();
            $(h.insert($key, $val);)*
            h
        })
    }


    #[test]
    fn test_dfa() {
        let transitions = map!((0, 'a') => 0, (0, 'b') => 1, (1, 'a') => 0, (1, 'b') => 2);
        let dfa = DFA::new(1, set!(2), transitions);
        assert_eq!(dfa.run("aaaaa".chars().collect()), false);
        assert_eq!(dfa.run("aabaa".chars().collect()), false);
        assert_eq!(dfa.run("aababbb".chars().collect()), false);
        assert_eq!(dfa.run("aababb".chars().collect()), true);
        assert_eq!(dfa.run("aabb".chars().collect()), true);
    }
}