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
    // TODO: delete
    /// Inefficient! Just for testing.
    fn new(source: &'s str, start: usize, end: usize) -> Srcloc<'s> {
        Srcloc {
            line: source[start..].lines().count(),
            column: source[start..].lines().last().unwrap().len(),
            source: &source[start..end],
        }
    }

    pub fn as_str(&self) -> &str {
        self.source
    }
}

impl<'s> Src<'s> {
    /// Inefficient! Just for testing.
    pub fn new(
        source: &'s str,
        start: usize,
        end: usize,
        construct: &'s str,
        args: &'s [Src<'s>],
    ) -> Src<'s> {
        Src {
            loc: Srcloc::new(source, start, end),
            construct,
            args,
        }
    }

    pub fn loc(&self) -> Srcloc<'s> {
        self.loc
    }

    pub fn as_str(&self) -> &str {
        self.loc.as_str()
    }

    pub fn construct(&self) -> &str {
        self.construct
    }

    pub fn args(&self) -> &[Src] {
        self.args
    }
}
