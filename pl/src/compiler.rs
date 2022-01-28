use super::expr::Compiled;
use super::source::Src;
use std::collections::HashMap;

#[derive(Debug)]
pub struct TypeError;

pub type Fragment = for<'s> fn(&mut Compiler, src: Src<'s>) -> Result<Compiled<'s>, TypeError>;

pub struct Registry {
    fragments: HashMap<String, Fragment>,
}

impl Registry {
    pub fn new() -> Registry {
        Registry {
            fragments: HashMap::new(),
        }
    }

    pub fn add_fragment(&mut self, con: &str, fragment: Fragment) {
        self.fragments.insert(con.to_owned(), fragment);
    }

    fn get_fragment<'s>(&self, con: &str) -> Option<Fragment> {
        self.fragments.get(con).copied()
    }
}

pub struct Compiler<'r> {
    registry: &'r Registry,
}

impl<'r> Compiler<'r> {
    pub fn new(registry: &'r Registry) -> Compiler<'r> {
        Compiler { registry }
    }

    pub fn compile<'s>(&mut self, src: Src<'s>) -> Result<Compiled<'s>, TypeError> {
        let fragment = match self.registry.get_fragment(src.construct()) {
            Some(fragment) => fragment,
            None => panic!("missing compiler fragment for {}", src.construct()),
        };
        let mut args = vec![];
        for arg in src.args() {
            args.push(self.compile(*arg));
        }
        fragment(self, src)
    }
}
