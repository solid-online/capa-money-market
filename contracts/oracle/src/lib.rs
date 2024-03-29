#[cfg(not(feature = "library"))]
pub mod contract;
mod error;
mod execute;
mod functions;
mod query;
mod state;

#[cfg(test)]
mod testing;
