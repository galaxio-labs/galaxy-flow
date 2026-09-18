use super::abilities::define::gal_gxl_object;
use super::inner::funs::gal_defined;
use super::prelude::*;
use orion_parse::define::take_var_ref_name;
use orion_parse::symbol::{
    LogicSymbol, symbol_bracket_beg, symbol_bracket_end, symbol_cmp, symbol_logic_and,
    symbol_logic_not, symbol_logic_or,
};
use winnow::combinator::repeat;

use crate::calculate::cond::IFExpress;
use crate::calculate::logic::LogicExpress;
use crate::calculate::{CmpExpress, ExpressEnum};
use crate::components::gxl_cond::GxlCond;
use crate::parser::atom::spaced;
use crate::parser::domain::gal_keyword;
use crate::parser::stc_blk::gal_block;
use crate::primitive::GxlObject;

pub fn gal_else_if(input: &mut &str) -> Result<GxlCond> {
    skip_spaces_block(input)?;
    gal_keyword("else", input)?;
    gal_keyword("if", input)?;
    let (name, cmp, value) = (
        spaced(take_var_ref_name).context(wn_desc("<env-var>")),
        spaced(symbol_cmp).context(wn_desc("operator")),
        spaced(gal_gxl_object).context(wn_desc("<value-str>")),
    )
        .parse_next(input)?;
    let true_block = gal_block.parse_next(input)?;
    skip_spaces_block(input)?;
    let ctrl_express = IFExpress::new(
        ExpressEnum::Cmp(CmpExpress::from_op(cmp, GxlObject::VarRef(name), value)),
        true_block,
        Vec::new(),
        None,
    );
    Ok(GxlCond::new(ctrl_express))
}

pub fn gal_cmp_exp(input: &mut &str) -> Result<ExpressEnum> {
    let (name, cmp, value) = (
        spaced(take_var_ref_name).context(wn_desc("<env-var>")),
        spaced(symbol_cmp).context(wn_desc("operator")),
        spaced(gal_gxl_object).context(wn_desc("<value-str>")),
    )
        .parse_next(input)?;
    Ok(ExpressEnum::Cmp(CmpExpress::from_op(
        cmp,
        GxlObject::VarRef(name),
        value,
    )))
}

pub fn gal_cmp_fun(input: &mut &str) -> Result<ExpressEnum> {
    let defined = gal_defined.parse_next(input)?;
    Ok(ExpressEnum::from(defined))
}

pub fn gal_logic_not(input: &mut &str) -> Result<ExpressEnum> {
    symbol_logic_not.parse_next(input)?;
    multispace0.parse_next(input)?;
    let first = alt((gal_exp_cmp, gal_exp_gourp)).parse_next(input)?;
    Ok(ExpressEnum::Logic(Box::new(LogicExpress::not_exp(first))))
}

pub fn gal_logic_bin(input: &mut &str) -> Result<(LogicSymbol, ExpressEnum)> {
    let logic = alt((symbol_logic_and, symbol_logic_or)).parse_next(input)?;
    multispace0.parse_next(input)?;
    let second = gal_exp.parse_next(input)?;
    Ok((logic, second))
}

pub fn gal_exp_cmp(input: &mut &str) -> Result<ExpressEnum> {
    alt((gal_cmp_exp, gal_cmp_fun)).parse_next(input)
}

pub fn gal_exp_gourp(input: &mut &str) -> Result<ExpressEnum> {
    multispace0.parse_next(input)?;
    symbol_bracket_beg.parse_next(input)?;
    multispace0.parse_next(input)?;
    let first = alt((gal_cmp_exp, gal_cmp_fun, gal_logic_not)).parse_next(input)?;
    let second = opt(gal_logic_bin).parse_next(input)?;
    multispace0.parse_next(input)?;
    symbol_bracket_end.parse_next(input)?;
    if let Some((logic, exp)) = second {
        Ok(match logic {
            LogicSymbol::And => ExpressEnum::Logic(Box::new(LogicExpress::and_exp(first, exp))),
            LogicSymbol::Or => ExpressEnum::Logic(Box::new(LogicExpress::or_exp(first, exp))),
            _ => unreachable!(),
        })
    } else {
        Ok(first)
    }
}

pub fn gal_exp(input: &mut &str) -> Result<ExpressEnum> {
    let first = alt((gal_cmp_exp, gal_cmp_fun, gal_logic_not, gal_exp_gourp)).parse_next(input)?;
    let second = opt(gal_logic_bin).parse_next(input)?;
    if let Some((logic, exp)) = second {
        Ok(match logic {
            LogicSymbol::And => ExpressEnum::Logic(Box::new(LogicExpress::and_exp(first, exp))),
            LogicSymbol::Or => ExpressEnum::Logic(Box::new(LogicExpress::or_exp(first, exp))),
            _ => unreachable!(),
        })
    } else {
        Ok(first)
    }
}

pub fn gal_cond(input: &mut &str) -> Result<GxlCond> {
    skip_spaces_block(input)?;
    gal_keyword("if", input)?;

    let exp = gal_exp.parse_next(input)?;
    let true_block = gal_block.parse_next(input)?;
    skip_spaces_block(input)?;
    let elseif_conds: Vec<GxlCond> = repeat(0.., gal_else_if).parse_next(input)?;

    multispace0(input)?;
    let false_block = if starts_with("else", input) {
        gal_keyword("else", input)?;
        Some(gal_block.parse_next(input)?)
    } else {
        None
    };
    let ctrl_express = IFExpress::new(exp, true_block, elseif_conds, false_block);
    Ok(GxlCond::new(ctrl_express))
}

#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;
    use orion_sec::sec::{SecFrom, SecValueType};

    use crate::{
        calculate::Evaluation,
        context::ExecContext,
        execution::VarSpace,
        parser::{
            cond::{gal_cond, gal_exp},
            inner::run_gxl,
            stc_blk::gal_block,
        },
        traits::Setter,
    };

    #[test]
    fn test_exp() {
        let mut dict = VarSpace::default();
        dict.global_mut().set("val", SecValueType::nor_from(1));
        dict.global_mut().set("val_1", SecValueType::nor_from(1));
        dict.global_mut().set("val2", SecValueType::nor_from(2));
        dict.global_mut().set("val_f", SecValueType::nor_from(1.14));
        dict.global_mut()
            .set("val_s", SecValueType::nor_from("1".to_string()));
        let mut data = r#" defined(${val})"#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(exp.decide(ExecContext::default(), &dict).assert());

        let mut data = r#" defined(${val_not_exists})"#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(!exp.decide(ExecContext::default(), &dict).assert());

        let mut data = r#" ${val} == 1"#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(exp.decide(ExecContext::default(), &dict).assert());

        let mut data = r#" ${val} == ${val_1}"#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(exp.decide(ExecContext::default(), &dict).assert());

        let mut data = r#" ${val_s} =* "1""#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(exp.decide(ExecContext::default(), &dict).assert());

        let mut data = r#" ${val} != 1"#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(!exp.decide(ExecContext::default(), &dict).assert());

        let mut data = r#" ${val2} > 1"#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(exp.decide(ExecContext::default(), &dict).assert());

        let mut data = r#" ${val2} >= 1"#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(exp.decide(ExecContext::default(), &dict).assert());
        assert_eq!(data, "");

        let mut data = r#" ${val_f} >= 1.11"#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(exp.decide(ExecContext::default(), &dict).assert());
        assert_eq!(data, "");

        let mut data = r#"  ${val} == 1 && ${val} == 1"#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(exp.decide(ExecContext::default(), &dict).assert());
        assert_eq!(data, "");

        let mut data = r#"  defined(${val}) && ${val} == 1"#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(exp.decide(ExecContext::default(), &dict).assert());
        assert_eq!(data, "");

        let mut data = r#"  defined(${val_not}) || ${val} == 1"#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(exp.decide(ExecContext::default(), &dict).assert());
        assert_eq!(data, "");

        let mut data = r#"  !defined(${val_not}) || ${val_not} == 1"#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(exp.decide(ExecContext::default(), &dict).assert());
        assert_eq!(data, "");

        let mut data = r#"  !defined(${val_not}) || ${val_not} == 1"#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(exp.decide(ExecContext::default(), &dict).assert());
        assert_eq!(data, "");

        let mut data = r#"  ${val} == 1 && ${val} == 2"#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(!exp.decide(ExecContext::default(), &dict).assert());
        assert_eq!(data, "");

        let mut data = r#"  ${val} == 1 || ${val} == 2"#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(exp.decide(ExecContext::default(), &dict).assert());
        assert_eq!(data, "");

        let mut data = r#" ! ${val} == 1"#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(!exp.decide(ExecContext::default(), &dict).assert());
        assert_eq!(data, "");

        let mut data = r#" ( ${val} == 1)"#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(exp.decide(ExecContext::default(), &dict).assert());
        assert_eq!(data, "");

        let mut data = r#"  (${val} == 1) || (${val} == 2)"#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(exp.decide(ExecContext::default(), &dict).assert());
        assert_eq!(data, "");

        let mut data = r#"  (defined(${val_not}) && ${val_not} == 1) || ${val} == 1"#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(exp.decide(ExecContext::default(), &dict).assert());
        assert_eq!(data, "");

        let mut data = r#"  (defined(${val}) && ${val} == 2) || ${val} == 1"#;
        let exp = run_gxl(gal_exp, &mut data).assert();
        assert!(exp.decide(ExecContext::default(), &dict).assert());
        assert_eq!(data, "");
    }

    #[test]
    fn test_block_if() {
        let mut data = r#"
            if defined(${val}) && ${val} == "1" {
                gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
             }"#;
        let _ = run_gxl(gal_cond, &mut data).assert();
        assert_eq!(data, "");

        let mut data = r#"
            if ${var} == 2 || ${val} == "1" {
                gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
             }"#;
        let _ = run_gxl(gal_cond, &mut data).assert();
        assert_eq!(data, "");
        //if !defined($CUR.ENABLE) || ${CUR.ENABLE} == true

        let mut data = r#"
            if !defined(${CUR.ENABLE}) || ${CUR.ENABLE} == true {
                gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
             }"#;
        let _ = run_gxl(gal_cond, &mut data).assert();
        assert_eq!(data, "");

        let mut data = r#"
            if ${val} == "1" {
                gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
             }
             else {
                gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
             }"#;
        let _ = run_gxl(gal_cond, &mut data).assert();
        assert_eq!(data, "");

        let mut data = r#"
            if ${val} == "1" {
                gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
             } else if ${val} == "2" {
                gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
             }
             else {
                gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
             }"#;
        let _ = run_gxl(gal_cond, &mut data).assert();
        assert_eq!(data, "");
    }
    #[test]
    fn test_block_3() {
        let mut data = r#"
        {
            if defined(${val}) && ${val} == "1" {
                gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
             }
             else {
                gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
             }
        }"#;
        let _ = run_gxl(gal_block, &mut data).assert();
        assert_eq!(data, "");
    }

    #[test]
    fn test_block_if_4() {
        let mut data = r#"
        {
            if defined(${val}) {
                gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
             }
             else {
                gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
             }
        }"#;
        let _ = run_gxl(gal_block, &mut data).assert();
        assert_eq!(data, "");
    }

    #[test]
    fn test_if_for() {
        let mut data = r#"
            {
            if ${val} == 1 {
                for  ${CUR} in ${DATA} {
                    gx.echo ( value  : "${cur}/test/main.py"  );
                }
             }
             }
        "#;
        let _ = run_gxl(gal_block, &mut data).assert();
        assert_eq!(data, "");
    }
    #[test]
    fn test_if_for2() {
        let mut data = r#"
            if ${val} =* "hello*" {
                for  ${CUR} in ${DATA} {
                    gx.echo ( value  : "${cur}/test/main.py"  );
                }
             }
        "#;
        let _cond = run_gxl(gal_cond, &mut data).assert();
        assert_eq!(data, "");
    }
}
