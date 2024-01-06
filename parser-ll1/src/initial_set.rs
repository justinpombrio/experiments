use crate::vec_map::VecMap;
use crate::GrammarError;
use crate::Token;

// TODO: think about ideal names & error messages

#[derive(Debug, Clone)]
pub struct InitialSet {
    empty_name: String,
    nonempty_name: String,
    accepts_empty: bool,
    accepted_tokens: VecMap<String>,
}

impl InitialSet {
    pub fn new_void() -> InitialSet {
        InitialSet {
            empty_name: "void".to_owned(),    // unreachable
            nonempty_name: "void".to_owned(), // unreachable
            accepts_empty: false,
            accepted_tokens: VecMap::new(),
        }
    }

    pub fn new_empty() -> InitialSet {
        InitialSet {
            empty_name: "nothing".to_owned(),
            nonempty_name: "nothing".to_owned(),
            accepts_empty: true,
            accepted_tokens: VecMap::new(),
        }
    }

    pub fn new_token(name: String, token: Token) -> InitialSet {
        let mut accepted_tokens = VecMap::new();
        accepted_tokens.set(token, name.clone());
        InitialSet {
            empty_name: format!("empty {}", name),
            nonempty_name: name,
            accepts_empty: false,
            accepted_tokens,
        }
    }

    pub fn sequence(
        names: (String, String, String),
        elems: Vec<InitialSet>,
    ) -> Result<InitialSet, GrammarError> {
        let (name, empty_name, nonempty_name) = names;

        let mut accepts_empty = true;
        let mut accepted_tokens: VecMap<(String, usize)> = VecMap::new();
        for (i, init) in elems.iter().enumerate() {
            if !init.accepts_empty {
                accepts_empty = false;
                break;
            }
            for (token, pattern) in &init.accepted_tokens {
                if let Some((_patt, j)) = accepted_tokens.get(token) {
                    return Err(GrammarError::AmbiguityOnFirstToken {
                        token: pattern.to_owned(),
                        name,
                        case_1: elems[*j].nonempty_name.to_owned(),
                        case_2: elems[i].nonempty_name.to_owned(),
                    });
                } else {
                    accepted_tokens.set(token, (pattern.to_owned(), i));
                }
            }
        }

        Ok(InitialSet {
            empty_name,
            nonempty_name,
            accepts_empty,
            accepted_tokens: accepted_tokens.map(|(pattern, _)| pattern),
        })
    }

    pub fn choice(
        names: (String, String, String),
        elems: Vec<InitialSet>,
    ) -> Result<InitialSet, GrammarError> {
        let (name, empty_name, nonempty_name) = names;

        let mut accepts_empty: Option<usize> = None;
        let mut accepted_tokens: VecMap<(String, usize)> = VecMap::new();
        for (i, init) in elems.iter().enumerate() {
            if init.accepts_empty {
                if let Some(j) = accepts_empty {
                    return Err(GrammarError::AmbiguityOnEmpty {
                        name,
                        case_1: elems[j].empty_name.to_owned(),
                        case_2: elems[i].empty_name.to_owned(),
                    });
                }
                accepts_empty = Some(i);
            }
            for (token, pattern) in &init.accepted_tokens {
                if let Some((_patt, j)) = accepted_tokens.get(token) {
                    return Err(GrammarError::AmbiguityOnFirstToken {
                        token: pattern.to_owned(),
                        name,
                        case_1: elems[*j].nonempty_name.to_owned(),
                        case_2: elems[i].nonempty_name.to_owned(),
                    });
                } else {
                    accepted_tokens.set(token, (pattern.to_owned(), i));
                }
            }
        }

        Ok(InitialSet {
            empty_name,
            nonempty_name,
            accepts_empty: accepts_empty.is_some(),
            accepted_tokens: accepted_tokens.map(|(pattern, _)| pattern),
        })
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

/* TODO
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
*/
