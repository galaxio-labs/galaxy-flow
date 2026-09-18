use crate::ability::prelude::*;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct GxEcho {
    value: String,
}
impl GxEcho {
    pub fn new<S: Into<String>>(val: S) -> Self {
        Self { value: val.into() }
    }
    pub fn set(&mut self, val: &str) {
        self.value = val.to_string();
    }
}

// impl DefaultDTO for RgEcho {}

#[async_trait]
impl AsyncRunnableTrait for GxEcho {
    async fn async_exec(&self, ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        let ex = EnvExpress::from_env_mix(vars_dict.global().clone());
        let out = ex.eval(&self.value)?;
        info!(target: ctx.path(), "{} :{}", self.value, out);
        println!("{out}");
        Ok(TaskValue::from((vars_dict, ExecOut::Ignore)))
    }
}

impl ComponentMeta for GxEcho {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.echo")
    }
}

#[cfg(test)]
mod tests {

    use crate::traits::Setter;

    use super::*;

    #[tokio::test]
    async fn echo_test() {
        let mut watcher = GxEcho::default();
        watcher.set("${HOME}");
        let ctx = ExecContext::default();
        let mut def = VarSpace::default();
        watcher.async_exec(ctx.clone(), def.clone()).await.unwrap();
        def.global_mut().set("HOME", "/root");
        watcher.async_exec(ctx.clone(), def).await.unwrap();
    }
}
