use num::rational::Rational64;
use num::ToPrimitive;
use rand_var::RandomVariable;
use rand_var::rv_traits::{NumRandVar, RandVar};

fn main() {
    let rv: RandomVariable<Rational64> = RandomVariable::new_dice(6).unwrap();
    let cdf3 = rv.cdf(3);
    println!("cdf(3): {}", cdf3);

    let ev = rv.expected_value();
    println!("EV[X] = {}", ev);

    let d10r2: RandomVariable<Rational64> = RandomVariable::new_dice_reroll(10,2).unwrap();
    println!("EV[d10r2] = {}", d10r2.expected_value());

    d10r2.print_pdf(|x| x.to_f64().unwrap());
    d10r2.print_stats_convert(|x| x.to_f64().unwrap());
}
