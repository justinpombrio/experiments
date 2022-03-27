use super::bytecode::Register;
use super::expr::{Compiled, Type};
use super::source::Src;
use std::collections::HashMap;

#[derive(Debug)]
pub struct TypeError;

pub type Sort = String;
pub type Construct = String;

pub type Fragment =
    for<'s> fn(&mut Compiler<'s, '_>, src: Src<'s>) -> Result<Compiled<'s>, TypeError>;

pub struct Registry {
    fragments: HashMap<(Sort, Construct), Fragment>,
}

impl Registry {
    pub fn new() -> Registry {
        Registry {
            fragments: HashMap::new(),
        }
    }

    pub fn add_fragment(&mut self, sort: &str, con: &str, fragment: Fragment) {
        self.fragments
            .insert((sort.to_owned(), con.to_owned()), fragment);
    }

    fn get_fragment<'s>(&self, sort: &str, con: &str) -> Option<Fragment> {
        self.fragments
            .get(&(sort.to_owned(), con.to_owned()))
            .copied()
    }
}

pub struct Compiler<'s, 'r> {
    register: Register,
    environment: Vec<(&'s str, Type, Register)>,
    registry: &'r Registry,
}

impl<'s, 'r> Compiler<'s, 'r> {
    pub fn new(registry: &'r Registry) -> Compiler<'s, 'r> {
        Compiler {
            register: 0,
            environment: vec![],
            registry,
        }
    }

    /// Must be followed by pop_var!
    pub fn push_var(&mut self, name: &'s str, typ: Type) -> Register {
        let register = self.register;
        self.environment.push((name, typ, register));
        self.register += 1;
        register
    }

    pub fn pop_var(&mut self, name: &'s str, register: Register) {
        let binding = self.environment.pop().unwrap();
        assert_eq!((name, register), (binding.0, binding.2));
        self.register -= 1;
    }

    pub fn lookup_var(&self, name: &'s str) -> Option<(Type, Register)> {
        for (var_name, var_typ, var_reg) in self.environment.iter().rev() {
            if *var_name == name {
                return Some((*var_typ, *var_reg));
            }
        }
        None
    }

    pub fn compile(&mut self, sort: &str, src: Src<'s>) -> Result<Compiled<'s>, TypeError> {
        let fragment = match self.registry.get_fragment(sort, src.construct()) {
            Some(fragment) => fragment,
            None => panic!("missing compiler fragment for {}", src.construct()),
        };
        fragment(self, src)
    }
}
