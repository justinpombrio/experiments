use crate::vec_map::VecMap;
use crate::{GrammarError, Token};
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct InitialSet {
    accepts_empty: bool,
    accepted_tokens: VecMap<String>,
}

impl InitialSet {
    pub fn new() -> InitialSet {
        InitialSet {
            accepts_empty: false,
            accepted_tokens: VecMap::new(),
        }
    }

    pub fn new_singleton(token: Token, pattern: String) -> InitialSet {
        let mut accepted_tokens = VecMap::new();
        accepted_tokens.set(token, pattern);
        InitialSet {
            accepts_empty: false,
            accepted_tokens,
        }
    }

    pub fn add_empty(&mut self, label: &str) -> Result<(), GrammarError> {
        if self.accepts_empty {
            return Err(GrammarError::AmbiguityOnEmpty(label.to_owned()));
        }
        self.accepts_empty = true;
        Ok(())
    }

    pub fn add_token(
        &mut self,
        label: &str,
        token: Token,
        pattern: String,
    ) -> Result<(), GrammarError> {
        if self.accepted_tokens.get(token).is_some() {
            return Err(GrammarError::AmbiguityOnFirstToken(
                label.to_owned(),
                token,
                pattern,
            ));
        }
        self.accepted_tokens.set(token, pattern);
        Ok(())
    }

    pub fn seq(mut self, label: &str, other: InitialSet) -> Result<InitialSet, GrammarError> {
        self.accepts_empty = self.accepts_empty && other.accepts_empty;
        if self.accepts_empty {
            for (token, pattern) in other.accepted_tokens.iter() {
                self.add_token(label, token, pattern.to_owned())?;
            }
        }
        Ok(self)
    }

    pub fn union(mut self, label: &str, other: InitialSet) -> Result<InitialSet, GrammarError> {
        if other.accepts_empty {
            self.add_empty(label)?;
        }
        for (token, pattern) in other.accepted_tokens.into_iter() {
            self.add_token(label, token, pattern.to_owned())?;
        }
        Ok(self)
    }
}

#[derive(Debug, Clone)]
pub struct ChoiceTable {
    empty_index: Option<usize>,
    token_indices: VecMap<usize>,
}

impl ChoiceTable {
    pub fn new(
        label: &str,
        initial_sets: Vec<InitialSet>,
    ) -> Result<(ChoiceTable, InitialSet), GrammarError> {
        let mut choice_table = ChoiceTable {
            empty_index: None,
            token_indices: VecMap::new(),
        };
        let mut initial_set = InitialSet::new();

        for (i, set) in initial_sets.into_iter().enumerate() {
            if set.accepts_empty {
                initial_set.add_empty(label)?;
                choice_table.empty_index = Some(i);
            }
            for (token, pattern) in set.accepted_tokens.into_iter() {
                initial_set.add_token(label, token, pattern)?;
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
