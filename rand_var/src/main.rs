use num::rational::Rational64;
use rand_var::{NumRandVar, RandomVariable};

fn main() {
    let rv: RandomVariable<Rational64> = RandomVariable::new_dice(6);
    let cdf3 = rv.cdf(3);
    println!("cdf(3): {}", cdf3);

    let ev = rv.expected_value();
    println!("EV[X] = {}", ev);

    let d10r2: RandomVariable<Rational64> = RandomVariable::new_dice_reroll(10,2);
    println!("EV[d10r2] = {}", d10r2.expected_value());
}
