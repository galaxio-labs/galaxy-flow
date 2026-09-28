use derive_more::From;

use crate::{
    calculate::traits::{DecideResult, Evaluation},
    context::ExecContext,
    execution::VarSpace,
};
use std::env;

#[derive(Clone, Debug, From)]
pub enum BoolBinFn {
    Defined(FnDefined),
    Exists(crate::calculate::exist::FnExists),
}

#[derive(Clone, Default, Builder, Debug, PartialEq, Getters)]
pub struct FnDefined {
    name: String,
}
impl FnDefined {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }
}

impl Evaluation for FnDefined {
    fn decide(&self, _ctx: ExecContext, args: &VarSpace) -> DecideResult {
        if args.get(self.name()).is_some() {
            return Ok(true);
        }
        if env::vars().any(|x| x.0.as_str() == self.name()) {
            return Ok(true);
        }
        Ok(false)
    }
}

impl Evaluation for BoolBinFn {
    fn decide(&self, ctx: ExecContext, args: &VarSpace) -> DecideResult {
        match self {
            BoolBinFn::Defined(f) => f.decide(ctx, args),
            BoolBinFn::Exists(f) => f.decide(ctx, args),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{context::ExecContext, execution::VarSpace, traits::Setter};

    #[test]
    fn test_defined_in_global_vars() {
        let mut var_space = VarSpace::default();
        var_space.global_mut().set("TEST_VAR", "value");

        let fn_defined = FnDefined::new("TEST_VAR");
        let result = fn_defined
            .decide(ExecContext::default(), &var_space)
            .unwrap();

        assert!(result, "Expected TEST_VAR to be defined in global vars");
    }

    #[test]
    fn test_defined_in_env_vars() {
        unsafe {
            std::env::set_var("ENV_VAR", "value");
        }

        let fn_defined = FnDefined::new("ENV_VAR");
        let result = fn_defined
            .decide(ExecContext::default(), &VarSpace::default())
            .unwrap();

        assert!(result, "Expected ENV_VAR to be defined in environment vars");
    }

    #[test]
    fn test_not_defined() {
        let fn_defined = FnDefined::new("NON_EXISTENT_VAR");
        let result = fn_defined
            .decide(ExecContext::default(), &VarSpace::default())
            .unwrap();

        assert!(!result, "Expected NON_EXISTENT_VAR to not be defined");
    }
}
