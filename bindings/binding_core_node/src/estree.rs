use napi::bindgen_prelude::*;
use napi_derive::napi;
use swc_core::ecma::ast::{Program, Script, Stmt};

#[napi]
pub struct JsProgram {
    program: Program,
}

#[napi]
pub struct JsScript {
    inner: SharedReference<JsProgram, Script>,
}

#[napi]
pub struct JsStmt {
    inner: SharedReference<JsProgram, Stmt>,
}
