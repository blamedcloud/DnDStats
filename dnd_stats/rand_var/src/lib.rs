
pub mod rand_var;
pub mod num_rand_var;
pub mod vec_rand_var;
pub mod map_rand_var;

#[derive(Debug, Clone)]
pub enum RVError {
    InvalidBounds,
    CDFNotOne,
    NegProb,
    NoRound,
    Other(String),
}
