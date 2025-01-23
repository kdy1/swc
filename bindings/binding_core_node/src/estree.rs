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
    pub fn stmts(&self, reference: Reference<JsProgram>, env: Env) -> Result<JsStmts> {
        Ok(JsStmts {
            inner: reference
                .share_with(env, |program| Ok(&*program.inner.as_script().unwrap().body))?,
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
