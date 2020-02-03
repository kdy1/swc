#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(specialization)]
#![feature(try_blocks)]

pub use self::{
    bundler::Bundler,
    id::{Id, ModuleId, QualifiedId},
};

mod bundler;
mod chunk;
mod debug;
mod id;
pub mod load;
pub mod loaders;
mod normalize;
pub mod resolve;
mod util;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub tree_shake: bool,
}
