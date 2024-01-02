use crate::vec_map::VecMap;
use crate::{GrammarError, Token};

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

    pub fn new_token(name: String, token: Token) -> InitialSet {
        let mut accepted_tokens = VecMap::new();
        accepted_tokens.set(token, name.clone());
        InitialSet {
            name,
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

    pub fn accepted_tokens(&self) -> VecMap<()> {
        self.accepted_tokens.map(|_| ())
    }

    #[cfg(test)]
    fn accepts_empty(&self) -> bool {
        self.accepts_empty
    }

    #[cfg(test)]
    fn accepts_token(&self, token: Token) -> bool {
        self.accepted_tokens.get(token).is_some()
    }
}

#[test]
fn test_initial_sets() {
    let set_a = InitialSet::new_token("A".to_owned(), 65);
    let set_b = InitialSet::new_token("B".to_owned(), 66);
    let set_c = InitialSet::new_token("C".to_owned(), 67);
    let set_d = InitialSet::new_token("D".to_owned(), 68);
    let set_empty = InitialSet::new_empty("e");

    let mut set_a_empty = set_empty.clone();
    assert!(set_a_empty.union("Ae", set_a.clone()).is_ok());
    assert!(set_a_empty.union("Aee", set_empty.clone()).is_err());
    assert!(set_a_empty.accepts_empty());
    assert!(set_a_empty.accepts_token(65));
    assert!(!set_a_empty.accepts_token(66));

    let mut set_bc = set_c.clone();
    assert!(set_bc.union("BC", set_b.clone()).is_ok());
    assert!(set_bc.union("BCC", set_c.clone()).is_err());
    assert!(!set_bc.accepts_empty());
    assert!(!set_bc.accepts_token(65));
    assert!(set_bc.accepts_token(66));
    assert!(set_bc.accepts_token(67));

    let mut set_d_empty = set_d.clone();
    assert!(set_d_empty.union("De", set_empty.clone()).is_ok());
    assert!(set_d_empty.accepts_empty());
    assert!(set_d_empty.accepts_token(68));

    let mut set_seq = set_d_empty.clone();
    assert!(set_seq.seq(set_bc.clone()).is_ok());
    assert!(set_seq.seq(set_a_empty.clone()).is_ok());
    assert!(!set_seq.accepts_empty());
    assert!(!set_seq.accepts_token(65));
    assert!(set_seq.accepts_token(66));
    assert!(set_seq.accepts_token(67));
    assert!(set_seq.accepts_token(68));
}
