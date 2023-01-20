use solvomatic::Solvomatic;

fn main() {
    let mut solver = Solvomatic::new();
    solver.set_display(|f, mapping| {
        use std::fmt::Write;

        for letter in "ABCDE".chars() {
            if let Some(digit) = mapping.get(&letter) {
                write!(f, "{}", digit)?;
            } else {
                write!(f, "_")?;
            }
        }
        Ok(())
    });

    solver.var('A', 1..9);
    solver.var('B', 0..9);
    solver.var('C', 0..9);
    solver.var('D', 0..9);
    solver.var('E', 0..9);

    solver.simple_constraint("sumAB", ['A', 'B'], |args| args[0] + args[1] == 7);
    solver.simple_constraint("sumAC", ['A', 'C'], |args| args[0] + args[1] == 8);
    solver.simple_constraint("sumBC", ['B', 'C'], |args| args[0] + args[1] == 9);
    solver.simple_constraint("sumDE", ['D', 'E'], |args| args[0] + args[1] == 1);

    let assignment = solver.solve().unwrap();
    println!("{}", solver.display(&assignment).unwrap());
}
