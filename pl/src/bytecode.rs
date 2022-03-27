use std::fmt;

pub type Register = u32;

#[derive(Debug)]
pub enum Value {
    Reg(u32),
    Int(i32),
}

#[derive(Debug)]
pub enum Instr {
    Push(Value),
    Add,
    GetReg,
    SetReg,
}

#[derive(Debug)]
pub struct Code(pub Vec<Instr>);

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Value::*;

        match self {
            Reg(i) => write!(f, "${}", i),
            Int(i) => write!(f, "{}", i),
        }
    }
}

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Instr::*;

        match self {
            Push(v) => write!(f, "{}", v),
            Add => write!(f, "+"),
            GetReg => write!(f, "get"),
            SetReg => write!(f, "set"),
        }
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut iter = self.0.iter();
        write!(f, "[")?;
        if let Some(first_instr) = iter.next() {
            write!(f, "{}", first_instr)?;
            for instr in iter {
                write!(f, " {}", instr)?;
            }
        }
        write!(f, "]")
    }
}
