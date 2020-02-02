#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(specialization)]

pub use self::{
    bundler::Bundler,
    id::{Id, ModuleId, QualifiedId},
};

mod bundler;
pub mod chunk;
mod id;
pub mod load;
pub mod loaders;
pub mod resolve;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub tree_shake: bool,
}
