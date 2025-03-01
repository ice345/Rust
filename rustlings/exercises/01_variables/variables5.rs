fn main() {
    let number = "T-H-R-E-E"; // Don't change this line
    println!("Spell a number: {}", number);

    // TODO: Fix the compiler error by changing the line below without renaming the variable.
    // number = 3;
    let number: i32 = match number {
        "T-H-R-E-E" => 3,
        _ => 0, // default case
    };
    println!("Number plus two is: {}", number + 2);
}
