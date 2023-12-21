use crate::vec_map::VecMap;
use crate::{GrammarError, Token};
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct InitialSet {
    name: String,
    accepts_empty: bool,
    accepted_tokens: VecMap<String>,
}

impl InitialSet {
    pub fn new(name: &str) -> InitialSet {
        InitialSet {
            name: name.to_owned(),
            accepts_empty: false,
            accepted_tokens: VecMap::new(),
        }
    }

    pub fn add_empty(&mut self) -> Result<(), GrammarError> {
        if self.accepts_empty {
            return Err(GrammarError::AmbiguityOnEmpty(self.name.to_owned()));
        }
        self.accepts_empty = true;
        Ok(())
    }

    pub fn add_token(&mut self, token: Token, pattern: String) -> Result<(), GrammarError> {
        if self.accepted_tokens.get(token).is_some() {
            return Err(GrammarError::AmbiguityOnFirstToken(
                self.name.to_owned(),
                token,
                pattern,
            ));
        }
        self.accepted_tokens.set(token, pattern);
        Ok(())
    }

    pub fn seq(&mut self, other: InitialSet) -> Result<(), GrammarError> {
        self.accepts_empty = self.accepts_empty && other.accepts_empty;
        if self.accepts_empty {
            for (token, pattern) in &other.accepted_tokens {
                self.add_token(token, pattern.to_owned())?;
            }
        }
        Ok(())
    }

    pub fn union(&mut self, other: InitialSet) -> Result<(), GrammarError> {
        if other.accepts_empty {
            self.add_empty()?;
        }
        for (token, pattern) in other.accepted_tokens {
            self.add_token(token, pattern.to_owned())?;
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
        let mut initial_set = InitialSet::new(name);

        for (i, set) in initial_sets.into_iter().enumerate() {
            if set.accepts_empty {
                initial_set.add_empty()?;
                choice_table.empty_index = Some(i);
            }
            for (token, pattern) in set.accepted_tokens {
                initial_set.add_token(token, pattern)?;
                choice_table.token_indices.set(token, i);
            }
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
