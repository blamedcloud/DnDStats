use num::rational::Rational64;
use rand_var::{NumRandVar, RandomVariable};

fn main() {
    let one_sixth = Rational64::new(1,6);
    let rv = RandomVariable::build(1,6,|_x| {one_sixth});
    let cdf3 = rv.cdf(3);
    println!("cdf(3): {}", cdf3);

    let ev = rv.expected_value();
    println!("EV[X] = {}", ev);
}
