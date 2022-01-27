pub type Code = Vec<Instr>;

#[derive(Debug)]
pub enum Value {
    Int(i32),
}

#[derive(Debug)]
pub enum Instr {
    Push(Value),
    Add,
}
