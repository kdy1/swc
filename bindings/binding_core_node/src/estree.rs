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
    #[napi]
    pub fn get_body(&self, env: Env) -> Result<EstreeStatements> {
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
impl EstreeStatements {
    #[napi]
    pub fn len(&self) -> u32 {
        self.inner.len() as u32
    }

    #[napi]
    pub fn get(&self, index: u32, env: Env) -> Result<EstreeStatement> {
        Ok(EstreeStatement {
            inner: self
                .inner
                .clone(env)?
                .share_with(env, |stmts| Ok(&stmts[index as usize]))?,
        })
    }
}

#[napi]
pub struct EstreeStatement {
    inner: SharedReference<EstreeProgram, &'static Stmt>,
}
