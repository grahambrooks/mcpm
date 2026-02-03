pub mod app;
pub mod error;
pub mod ide;
pub mod message;
pub mod model;
pub mod registry;
pub mod services;
pub mod view;

pub use app::App;
pub use error::{McpmError, Result};
