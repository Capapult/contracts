pub mod contract;
pub mod msg;
pub mod state;
pub mod deposit;
pub mod math;
pub mod querier;

#[cfg(test)]
mod testing;

#[cfg(all(target_arch = "wasm32", not(feature = "library")))]
cosmwasm_std::create_entry_points!(contract);
