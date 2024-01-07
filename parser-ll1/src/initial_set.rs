use crate::vec_map::VecMap;
use crate::GrammarError;
use crate::Token;

#[derive(Debug, Clone)]
pub struct InitialSet {
    name: String,
    accepts_empty: bool,
    accepted_tokens: VecMap<String>,
}

impl InitialSet {
    pub fn void() -> InitialSet {
        InitialSet {
            name: "void".to_owned(), // unreachable
            accepts_empty: false,
            accepted_tokens: VecMap::new(),
        }
    }

    pub fn empty() -> InitialSet {
        InitialSet {
            name: "nothing".to_owned(),
            accepts_empty: true,
            accepted_tokens: VecMap::new(),
        }
    }

    pub fn token(name: String, token: Token) -> InitialSet {
        let mut accepted_tokens = VecMap::new();
        accepted_tokens.set(token, name.clone());
        InitialSet {
            name,
            accepts_empty: false,
            accepted_tokens,
        }
    }

    pub fn sequence(name: String, elems: Vec<InitialSet>) -> Result<InitialSet, GrammarError> {
        let mut accepts_empty = true;
        let mut accepted_tokens: VecMap<(String, usize)> = VecMap::new();
        for (i, init) in elems.iter().enumerate() {
            for (token, pattern) in &init.accepted_tokens {
                if let Some((_patt, j)) = accepted_tokens.get(token) {
                    return Err(GrammarError::AmbiguityOnFirstToken {
                        token: pattern.to_owned(),
                        name,
                        case_1: elems[*j].name.clone(),
                        case_2: elems[i].name.clone(),
                    });
                }
                accepted_tokens.set(token, (pattern.to_owned(), i));
            }
            if !init.accepts_empty {
                accepts_empty = false;
                break;
            }
        }

        Ok(InitialSet {
            name,
            accepts_empty,
            accepted_tokens: accepted_tokens.map(|(pattern, _)| pattern),
        })
    }

    pub fn choice(name: String, elems: Vec<InitialSet>) -> Result<InitialSet, GrammarError> {
        let mut accepts_empty: Option<usize> = None;
        let mut accepted_tokens: VecMap<(String, usize)> = VecMap::new();
        for (i, init) in elems.iter().enumerate() {
            if init.accepts_empty {
                if let Some(j) = accepts_empty {
                    return Err(GrammarError::AmbiguityOnEmpty {
                        name,
                        case_1: elems[j].name.clone(),
                        case_2: elems[i].name.clone(),
                    });
                }
                accepts_empty = Some(i);
            }
            for (token, pattern) in &init.accepted_tokens {
                if let Some((_patt, j)) = accepted_tokens.get(token) {
                    return Err(GrammarError::AmbiguityOnFirstToken {
                        token: pattern.to_owned(),
                        name,
                        case_1: elems[*j].name.clone(),
                        case_2: elems[i].name.clone(),
                    });
                }
                accepted_tokens.set(token, (pattern.to_owned(), i));
            }
        }

        Ok(InitialSet {
            name,
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

#[test]
fn test_initial_sets() {
    let name = String::new();

    let set_a = InitialSet::token("A".to_owned(), 65);
    let set_b = InitialSet::token("B".to_owned(), 66);
    let set_c = InitialSet::token("C".to_owned(), 67);
    let set_d = InitialSet::token("D".to_owned(), 68);
    let set_empty = InitialSet::empty();

    let set_a_empty =
        InitialSet::choice(name.clone(), vec![set_a.clone(), set_empty.clone()]).unwrap();
    assert!(
        InitialSet::choice(name.clone(), vec![set_a_empty.clone(), set_empty.clone()]).is_err()
    );
    assert!(set_a_empty.accepts_empty());
    assert!(set_a_empty.accepts_token(65));
    assert!(!set_a_empty.accepts_token(66));

    let set_bc = InitialSet::choice(name.clone(), vec![set_b.clone(), set_c.clone()]).unwrap();
    assert!(InitialSet::choice(name.clone(), vec![set_bc.clone(), set_c.clone()]).is_err());
    assert!(!set_bc.accepts_empty());
    assert!(!set_bc.accepts_token(65));
    assert!(set_bc.accepts_token(66));
    assert!(set_bc.accepts_token(67));

    let set_d_empty = InitialSet::choice(name.clone(), vec![set_d, InitialSet::empty()]).unwrap();
    assert!(set_d_empty.accepts_empty());
    assert!(set_d_empty.accepts_token(68));

    let set_seq =
        InitialSet::sequence(name.clone(), vec![set_d_empty, set_bc, set_a_empty]).unwrap();
    assert!(!set_seq.accepts_empty());
    assert!(!set_seq.accepts_token(65));
    assert!(set_seq.accepts_token(66));
    assert!(set_seq.accepts_token(67));
    assert!(set_seq.accepts_token(68));
}
