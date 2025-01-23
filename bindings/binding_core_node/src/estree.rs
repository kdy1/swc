use napi::bindgen_prelude::*;
use napi_derive::napi;
use swc_core::ecma::ast::{Program, Script, Stmt};

#[napi]
pub struct JsProgram {
    inner: Program,
}

#[napi]
pub struct JsScript {
    inner: SharedReference<JsProgram, Script>,
}

#[napi]
impl JsScript {
    #[napi]
    pub fn stmts(&self, env: Env) -> Result<JsStmts> {
        Ok(JsStmts {
            inner: self
                .inner
                .clone(env)?
                .share_with(env, |script| Ok(&*script.body))?,
        })
    }
}

#[napi]
pub struct JsStmts {
    inner: SharedReference<JsProgram, &'static [Stmt]>,
}

#[napi]
pub struct JsStmt {
    inner: SharedReference<JsProgram, Stmt>,
}
