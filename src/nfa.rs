use {Automaton, DFA};
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::Write;
use std::collections::hash_map::Entry::Vacant;
use std::collections::{HashSet, HashMap, VecDeque, BTreeSet};
use std::hash::Hash;
use nfa::Transition::{Input, Epsilon, Anything};

#[macro_export]
macro_rules! set {
    ($($elem:expr),*) => ({
        let mut s = ::std::collections::HashSet::new();
        $(s.insert($elem);)*
        s
    })
}

#[derive(Debug, Clone)]
pub struct NFA<S: Eq + Hash = usize, I: Eq + Hash = char> {
    start: S,
    accept_states: HashSet<S>,
    transitions: HashMap<(S, Transition<I>), HashSet<S>>
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum Transition<I> {
    Anything,
    Epsilon,
    Input(I)
}

pub struct NFAIter<'a, S: 'a, I: 'a> {
    queue: VecDeque<(&'a S, usize)>,
    input: Vec<I>,
    transitions: &'a HashMap<(S, Transition<I>), HashSet<S>>
}

impl<'a, S: 'a + Hash + Eq + Copy, I: Hash + Eq + Copy> Iterator for NFAIter<'a, S, I> {
    type Item = &'a S;

    fn next(&mut self) -> Option<&'a S> {
        if self.queue.is_empty() {
            None
        } else {
            let (state, pos) = match self.queue.pop_front() {
                Some(s) => s,
                None => panic!("Shouldn't happen")
            };

            if pos < self.input.len() {
                if let Some(set) = self.transitions.get(&(*state, Input(self.input[pos]))) {
                    for item in set {
                        self.queue.push_back((item, pos + 1))
                    }
                } else if let Some(set) = self.transitions.get(&(*state, Anything)) {
                    for item in set {
                        self.queue.push_back((item, pos + 1))
                    }
                }
            }
            if let Some(set) = self.transitions.get(&(*state, Epsilon)) {
                for item in set {
                    self.queue.push_back((item, pos))
                }
            }

            Some(state)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>){
        (0, None) // TODO
    }
}

impl<S: Clone + Eq + Hash = usize, I: Eq + Hash + Copy = char> NFA<S, I> {
    pub fn new(start: S, accept_states: HashSet<S>,
               transitions: HashMap<(S, Transition<I>), HashSet<S>>) -> NFA<S, I> {
        NFA { start: start, accept_states: accept_states, transitions: transitions }
    }

    pub fn get_accept_states(&self) -> &HashSet<S> {
        &self.accept_states
    }

    pub fn get_start_state(&self) -> &S {
        &self.start
    }

    pub fn get_transitions(&self) -> &HashMap<(S, Transition<I>), HashSet<S>> {
        &self.transitions
    }

    pub fn iter(&self, input: Vec<I>) -> NFAIter<S, I> {
        let mut queue = VecDeque::new();
        queue.push_back((&self.start, 0));
        NFAIter { queue: queue, input: input, transitions: &self.transitions }
    }

    pub fn into_dfa(&self) -> DFA<usize, I> where S: Ord {
        let clone = self.clone();
        let mut alphabet = HashSet::new();
        for (trans, _) in clone.transitions.iter() {
            // Don't add epsilon
            match trans.1 {
                Input(c) => alphabet.insert(Input(c)),
                Anything => alphabet.insert(Anything),
                _ => {false},
            };
        }

        let mut states = HashMap::new();
        let mut accept_states = HashSet::new();
        let mut transitions = HashMap::new();
        let mut id = 0;
        let mut get_id = || { let ret = id; id += 1; ret };
        let mut queue = VecDeque::new();

        let mut init_state = set!(clone.start.clone());
        clone.epsilon_closure(&mut init_state);
        queue.push_back((get_id(), init_state.clone()));
        states.insert(init_state.into_iter().collect(), 0);
        while let Some((cur_id, cur_state)) = queue.pop_front() {
            for a in alphabet.iter() {
                let mut new_state = clone.reachable_states(&cur_state, *a);
                clone.epsilon_closure(&mut new_state);

                if let Anything = *a {
                    for b in alphabet.iter() {
                        let mut extra_states = clone.reachable_states(&new_state, *b);
                        new_state.extend(extra_states);
                    }
                }

                let new_state_set: BTreeSet<_> = new_state.clone().into_iter().collect();
                if new_state.len() > 0 {
                    if let Vacant(entry) = states.entry(new_state_set.clone()) {
                        let id = get_id();
                        if let Some(s) = clone.get_accept(&new_state) {
                            accept_states.insert(id);
                        }
                        queue.push_back((id, new_state));
                        entry.insert(id);
                    }
                    // TODO: Find a way to not requery
                    let id = states.get(&new_state_set).unwrap();
                    transitions.insert((cur_id, *a), *id);
                }
            }
        }

        DFA::new(0, accept_states, transitions)
    }

    fn get_accept(&self, states: &HashSet<S>) -> Option<S> {
        let (set, other) = if states.len() < self.accept_states.len() {
            (states, &self.accept_states) }
        else {
            (&self.accept_states, states)
        };

        for s in set {
            if other.contains(&s) {
                return Some(s.clone())
            }
        }
        None
    }

    fn reachable_states(&self, states: &HashSet<S>, input: Transition<I>) -> HashSet<S> {
        let mut reachable_states = HashSet::new();
        for s in states {
            if let Some(next_states) = self.transitions.get(&(s.clone(), input)) {
                for ns in next_states {
                    reachable_states.insert(ns.clone());
                }
            }
        }
        reachable_states
    }

    fn epsilon_closure(&self, states: &mut HashSet<S>) {
        loop {
            let new_states = self.reachable_states(states, Epsilon);
            let old_len = states.len();
            states.extend(new_states);
            if old_len == states.len() {
                break;
            }
        }
    }
}

impl<S, I> Automaton for NFA<S, I> where S: Hash + Eq + Copy, I: Hash + Eq + Copy {
    type State = S;
    type Alphabet = I;

    fn run(&self, s: Vec<I>) -> Option<Vec<I>> {
        let mut queue = VecDeque::new();
        let mut path = Vec::<I>::new();

        queue.push_back((self.start, 0));
        while let Some((state, pos)) = queue.pop_front() {
            if let Some(set) = self.transitions.get(&(state, Epsilon)) {
                for item in set {
                    queue.push_back((*item, pos))
                }
            }

            if let Some(set) = self.transitions.get(&(state, Anything)) {
                for item in set {
                    queue.push_back((*item, pos + 1))
                }
            }

            path.push(s[pos]);
            if pos == s.len() {
                if self.accept_states.contains(&state) {
                    return Some(path)
                }
            } else {
                if let Some(set) = self.transitions.get(&(state, Input(s[pos]))) {
                    for item in set {
                        queue.push_back((*item, pos + 1))
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use {Automaton, NFA};
    use nfa::Transition::Input;
    use std::collections::HashSet;

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
    fn test_nfa() {
        let transitions = map!((0, Input('a')) => set!(0, 1),
                               (0, Input('b')) => set!(1),
                               (1, Input('a')) => set!(0, 1),
                               (1, Input('b')) => set!(2));
        let nfa = NFA::new(0, set!(2), transitions);
        assert_eq!(nfa.run("aaaaa".chars().collect()), None);
        assert_eq!(nfa.run("aabaa".chars().collect()), None);
        assert_eq!(nfa.run("aababbb".chars().collect()), None);
        assert_eq!(nfa.run("aababb".chars().collect()), Some(2));
        assert_eq!(nfa.run("aabb".chars().collect()), Some(2));
    }

    #[ignore] // We need to check for isomorphism, not equality
    #[test]
    fn test_into_dfa() {
        // let transitions = map!((0, Input('a')) => set!(0, 1),
        //                        (0, Input('b')) => set!(1),
        //                        (1, Input('a')) => set!(0, 1),
        //                        (1, Input('b')) => set!(2));
        // let nfa = NFA::new(0, set!(2), transitions);
        // let dfa1 = nfa.into_dfa();

        // let transitions = map!((0, 'a') => 1,
        //                        (0, 'b') => 2,
        //                        (1, 'a') => 1,
        //                        (1, 'b') => 3,
        //                        (2, 'a') => 1,
        //                        (2, 'b') => 4,
        //                        (3, 'a') => 1,
        //                        (3, 'b') => 4);
        // let dfa2 = DFA::new(0, set!(3, 4), transitions);
        // assert_eq!(dfa1, dfa2)
    }

    #[test]
    fn test_reachable() {
        let transitions = map!((0, Input('a')) => set!(0, 1),
                               (0, Input('b')) => set!(1),
                               (1, Input('a')) => set!(0, 1),
                               (1, Input('b')) => set!(2));
        let nfa = NFA::new(0, set!(2), transitions);
        let s = nfa.reachable_states(&set!(0), Input('a'));

        assert_eq!(s, set!(0, 1));

        let s = nfa.reachable_states(&set!(0), Input('b'));
        assert_eq!(s, set!(1));

        let s = nfa.reachable_states(&set!(1), Input('a'));
        assert_eq!(s, set!(0, 1));

        let s = nfa.reachable_states(&set!(1), Input('b'));
        assert_eq!(s, set!(2));

        let s = nfa.reachable_states(&set!(2), Input('a'));
        assert_eq!(s, HashSet::new());

        let s = nfa.reachable_states(&set!(2), Input('b'));
        assert_eq!(s, HashSet::new());

    }

    #[test]
    fn test_epsilon_closure() {
        let transitions = map!((0, Input('a')) => set!(0, 1),
                               (0, Input('b')) => set!(1),
                               (1, Input('a')) => set!(0, 1),
                               (1, Input('b')) => set!(2));
        let nfa = NFA::new(1, set!(2), transitions);

        let mut s = set!(0);
        nfa.epsilon_closure(&mut s);
        assert_eq!(s, set!(0));

        let mut s = set!(1);
        nfa.epsilon_closure(&mut s);
        assert_eq!(s, set!(1));

        let mut s = set!(2);
        nfa.epsilon_closure(&mut s);
        assert_eq!(s, set!(2));
    }
}
