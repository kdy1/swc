use napi::bindgen_prelude::*;
use napi_derive::napi;
use swc_core::ecma::ast::{Program, Script, Stmt};

#[napi]
pub struct EstreeProgram {
    inner: Program,
}

#[napi]
pub struct EstreeScript {
    inner: SharedReference<EstreeProgram, Script>,
}

#[napi]
impl EstreeScript {
    #[napi(getter)]
    pub fn stmts(&self, env: Env) -> Result<EstreeStatements> {
        Ok(EstreeStatements {
            inner: self
                .inner
                .clone(env)?
                .share_with(env, |script| Ok(&*script.body))?,
        })
    }
}

#[napi]
pub struct EstreeStatements {
    inner: SharedReference<EstreeProgram, &'static [Stmt]>,
}

#[napi]
pub struct EstreeStatement {
    inner: SharedReference<EstreeProgram, Stmt>,
}
