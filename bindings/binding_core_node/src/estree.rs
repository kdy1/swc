use napi::bindgen_prelude::*;
use napi_derive::napi;
use swc_core::ecma::ast::{ExprStmt, Program, Script, Stmt};

#[napi]
pub struct EstreeProgram {
    inner: Program,
}

#[napi]
impl EstreeProgram {
    #[napi]
    pub fn as_script(
        &self,
        env: Env,
        reference: Reference<EstreeProgram>,
    ) -> Result<Option<EstreeScript>> {
        let Some(..) = self.inner.as_script() else {
            return Ok(None);
        };

        let inner = reference.share_with(env, |program| Ok(program.inner.as_script().unwrap()))?;

        Ok(Some(EstreeScript { inner }))
    }
}

#[napi]
pub struct EstreeScript {
    inner: SharedReference<EstreeProgram, &'static Script>,
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
    #[napi(getter)]
    pub fn length(&self) -> u32 {
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

#[napi]
impl EstreeStatement {
    #[napi]
    pub fn as_expression_statement(&self, env: Env) -> Result<Option<EstreeExpressionStatement>> {
        if !self.inner.is_expr() {
            return Ok(None);
        }

        let inner = self
            .inner
            .clone(env)?
            .share_with(env, |stmt| Ok(stmt.as_expr().unwrap()))?;

        Ok(Some(EstreeExpressionStatement { inner }))
    }
}

#[napi]
pub struct EstreeExpressionStatement {
    inner: SharedReference<EstreeProgram, &'static ExprStmt>,
}
