use crate::ability::prelude::*;

#[derive(Clone, Default, Builder, Debug, PartialEq, Getters)]
pub struct GxAssert {
    value: String,
    expect: String,
    result: bool,
    error: Option<String>,
}
impl GxAssert {
    pub fn expect_eq(&mut self, val: &str, expect: &str) {
        self.value = val.to_string();
        self.expect = expect.to_string();
        self.result = true;
    }
    pub fn expect_no_eq(&mut self, val: &str, expect: &str) {
        self.value = val.to_string();
        self.expect = expect.to_string();
        self.result = false;
    }
    pub fn from_diy_error<S: Into<String>>(err: S) -> Self {
        Self {
            error: Some(err.into()),
            ..Default::default()
        }
    }
}

//impl DefaultDTO for RgAssert {}

#[async_trait]
impl AsyncRunnableTrait for GxAssert {
    async fn async_exec(&self, mut ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        ctx.append("assert");
        let exp = EnvExpress::from_env_mix(vars_dict.global().clone());
        let value = exp.eval(&self.value)?;
        let expect = exp.eval(&self.expect)?;
        debug!(target: ctx.path(), "value  {} :{}", self.value, value);
        debug!(target: ctx.path(), "expect {} :{}", self.expect, expect);

        match value == expect {
            true if self.result => {
                if !ctx.quiet() {
                    eprintln!("assert true : {value}");
                }
                info!(target: ctx.path(), "value {value} match exprect");
                Ok(TaskValue::from((vars_dict, ExecOut::Ignore)))
            }
            false if !self.result => {
                if !ctx.quiet() {
                    eprintln!("assert true : {value}");
                }
                info!(target: ctx.path(), "value {value} match exprect");
                Ok(TaskValue::from((vars_dict, ExecOut::Ignore)))
            }
            _ => {
                let mut err_msg = format!(
                    "assert fail! result: [{}],\n expect: [{}],\n value:  [{}]",
                    self.result, expect, value
                );
                if let Some(msg) = self.error.clone() {
                    err_msg = exp.eval(&msg)?;
                }
                if !ctx.quiet() {
                    eprintln!("{err_msg}");
                }
                Err(ExecReason::Assert.to_err().with_detail(format!(
                    "assert fail! [{}], expect: {},\n value {}",
                    self.result, expect, value
                )))
            }
        }
    }
}
impl ComponentMeta for GxAssert {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.assert")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn assert_test() {
        let mut assert = GxAssert::default();
        let ctx = ExecContext::default();
        let def = VarSpace::default();
        assert.expect_eq("hello", "hello");
        assert.async_exec(ctx.clone(), def.clone()).await.unwrap();
        assert.expect_eq("${HOME}", "${HOME}");
        assert.async_exec(ctx.clone(), def.clone()).await.unwrap();
        assert.expect_no_eq("${HOME}", "xxxx");
        assert.async_exec(ctx.clone(), def.clone()).await.unwrap();
    }
}
