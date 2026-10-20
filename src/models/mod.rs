pub mod token;
pub mod error;
pub mod users;

pub use self::{
    error::{ModelError, ModelResult},
    users::{LoginUser, RegisterUser, User},
};