#[derive(Debug, Clone, Copy)]
pub struct Srcloc<'s> {
    pub line: usize,
    pub column: usize,
    pub source: &'s str,
}

#[derive(Debug, Clone, Copy)]
pub struct Src<'s> {
    pub loc: Srcloc<'s>,
    pub construct: &'s str,
    pub args: &'s [Src<'s>],
}

impl<'s> Srcloc<'s> {
    pub fn as_str(&self) -> &str {
        self.source
    }
}

impl<'s> Src<'s> {
    pub fn loc(&self) -> Srcloc<'s> {
        self.loc
    }

    pub fn as_str(&self) -> &str {
        self.loc.as_str()
    }

    pub fn construct(&self) -> &str {
        self.construct
    }

    pub fn args(&self) -> &'s [Src<'s>] {
        self.args
    }
}
