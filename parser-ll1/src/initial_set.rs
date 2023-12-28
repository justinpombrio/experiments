use crate::vec_map::VecMap;
use crate::{GrammarError, Token};
use std::ops::{Index, IndexMut};

// TODO: think about ideal names & error messages

#[derive(Debug, Clone)]
pub struct InitialSet {
    name: String,
    accepts_empty: bool,
    accepted_tokens: VecMap<String>,
}

impl InitialSet {
    pub fn new_void(name: &str) -> InitialSet {
        InitialSet {
            name: name.to_owned(),
            accepts_empty: false,
            accepted_tokens: VecMap::new(),
        }
    }

    pub fn new_empty(name: &str) -> InitialSet {
        InitialSet {
            name: name.to_owned(),
            accepts_empty: true,
            accepted_tokens: VecMap::new(),
        }
    }

    pub fn new_token(name: &str, token: Token) -> InitialSet {
        let mut accepted_tokens = VecMap::new();
        accepted_tokens.set(token, name.to_owned());
        InitialSet {
            name: name.to_owned(),
            accepts_empty: false,
            accepted_tokens,
        }
    }

    pub fn seq(&mut self, other: InitialSet) -> Result<(), GrammarError> {
        let accepts_empty = self.accepts_empty;
        self.accepts_empty = self.accepts_empty && other.accepts_empty;
        if accepts_empty {
            for (token, pattern) in other.accepted_tokens {
                if self.accepted_tokens.get(token).is_some() {
                    return Err(GrammarError::AmbiguityOnFirstToken {
                        start: "sequence".to_owned(),
                        case_1: self.name.clone(),
                        case_2: other.name,
                        pattern: pattern,
                    });
                }
                self.accepted_tokens.set(token, pattern);
            }
        }
        Ok(())
    }

    pub fn union(&mut self, parent_name: &str, other: InitialSet) -> Result<(), GrammarError> {
        if other.accepts_empty {
            if self.accepts_empty {
                return Err(GrammarError::AmbiguityOnEmpty {
                    start: parent_name.to_owned(),
                    case_1: self.name.clone(),
                    case_2: other.name,
                });
            }
            self.accepts_empty = true;
        }
        for (token, pattern) in other.accepted_tokens {
            if self.accepted_tokens.get(token).is_some() {
                return Err(GrammarError::AmbiguityOnFirstToken {
                    start: parent_name.to_owned(),
                    case_1: self.name.clone(),
                    case_2: other.name,
                    pattern: pattern,
                });
            }
            self.accepted_tokens.set(token, pattern);
        }
        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct ChoiceTable {
    empty_index: Option<usize>,
    token_indices: VecMap<usize>,
}

impl ChoiceTable {
    pub fn new(
        name: &str,
        initial_sets: Vec<InitialSet>,
    ) -> Result<(ChoiceTable, InitialSet), GrammarError> {
        let mut choice_table = ChoiceTable {
            empty_index: None,
            token_indices: VecMap::new(),
        };
        let mut initial_set = InitialSet::new_void(name);

        for (i, set) in initial_sets.into_iter().enumerate() {
            if set.accepts_empty {
                choice_table.empty_index = Some(i);
            }
            for (token, _) in &set.accepted_tokens {
                choice_table.token_indices.set(token, i);
            }
            initial_set.union(name, set)?;
        }

        Ok((choice_table, initial_set))
    }

    pub fn lookup(&self, token: Option<Token>) -> Option<usize> {
        match token {
            None => self.empty_index,
            Some(token) => self.token_indices.get(token).copied(),
        }
    }
}

#[test]
fn test_initial_sets() {
    let mut set_2 = InitialSet::new("testing");
    assert!(set_2.add_token(2, "two".to_owned()).is_ok());
    let mut set_empty_2 = InitialSet::new("testing");
    assert!(set_empty_2.add_empty().is_ok());
    assert!(set_empty_2.union(set_2).is_ok());

    let mut set_5 = InitialSet::new("testing");
    assert!(set_5.add_token(5, "five".to_owned()).is_ok());

    let mut set_4 = InitialSet::new("testing");
    assert!(set_4.add_token(4, "four".to_owned()).is_ok());
    let mut set_14 = InitialSet::new("testing");
    assert!(set_14.add_empty().is_ok());
    assert!(set_14.add_token(1, "one".to_owned()).is_ok());
    assert!(set_14.seq(set_4).is_ok());

    let (table, set) = ChoiceTable::new("testing", vec![set_empty_2, set_5, set_14]).unwrap();
    assert_eq!(table.lookup(None), Some(0));
    assert_eq!(table.lookup(Some(1)), Some(2));
    assert_eq!(table.lookup(Some(2)), Some(0));
    assert_eq!(table.lookup(Some(3)), None);
    assert_eq!(table.lookup(Some(4)), Some(2));
    assert_eq!(table.lookup(Some(5)), Some(1));
    assert_eq!(table.lookup(Some(6)), None);
}
