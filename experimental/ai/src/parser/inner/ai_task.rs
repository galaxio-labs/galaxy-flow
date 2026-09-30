use orion_variate::opt::OptionFrom;

use crate::ability::ai::ai_task::AiTaskExecutor as GxAIFun;
use crate::parser::inner::prelude::*;

pub fn gal_ai_task(input: &mut &str) -> Result<GxAIFun> {
    let mut ai_fun = GxAIFun::default();
    gal_keyword("gx.ai_task", input)?;
    let props = action_call_args.parse_next(input)?;
    for one in props {
        let key = one.0.to_lowercase();
        if key == "role" {
            ai_fun.set_role(one.1.to_opt());
        } else if key == "task" {
            ai_fun.set_task(one.1.to_opt());
        } else if key == "tools" {
            let tools: Vec<String> = one.1.split(",").map(String::from).collect();
            ai_fun.set_tools(tools);
        }
    }
    Ok(ai_fun)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ai_fun_with_role() {
        let mut input = r#"gx.ai_task(
            role: "developer",
            task: "检查代码"
        );"#;

        let ai_fun = gal_ai_task(&mut input).unwrap();

        assert_eq!(ai_fun.role(), &Some("developer".to_string()));
        assert_eq!(ai_fun.task(), &Some("检查代码".to_string()));
    }

    #[test]
    fn test_parse_ai_fun_with_tools_list() {
        let mut input = r#"gx.ai_task(
            role: "developer",
            task: "使用指定的 Git 工具",
            tools: "git-status,git-add"
        );"#;

        let ai_fun = gal_ai_task(&mut input).unwrap();
        assert_eq!(ai_fun.role(), &Some("developer".to_string()));
        assert_eq!(ai_fun.task(), &Some("使用指定的 Git 工具".to_string()));
        assert_eq!(
            ai_fun.tools(),
            &vec!["git-status".to_string(), "git-add".to_string()]
        );
    }

    #[test]
    fn test_parse_ai_fun_with_max_rounds_default() {
        let mut input = r#"gx.ai_task(
            role: "developer",
            task: "执行任务"
        );"#;

        let ai_fun = gal_ai_task(&mut input).unwrap();
        assert_eq!(ai_fun.role(), &Some("developer".to_string()));
        assert_eq!(ai_fun.task(), &Some("执行任务".to_string()));
        assert_eq!(*ai_fun.max_rounds(), 3); // 默认值
    }

    #[test]
    fn test_parse_ai_fun_with_all_params() {
        let mut input = r#"gx.ai_task(
            role: "developer",
            task: "执行完整Git工作流",
            tools: "git-status,git-add,git-commit",
        );"#;

        let ai_fun = gal_ai_task(&mut input).unwrap();
        assert_eq!(ai_fun.role(), &Some("developer".to_string()));
        assert_eq!(ai_fun.task(), &Some("执行完整Git工作流".to_string()));
        assert_eq!(
            ai_fun.tools(),
            &vec![
                "git-status".to_string(),
                "git-add".to_string(),
                "git-commit".to_string()
            ]
        );
        assert_eq!(*ai_fun.max_rounds(), 3);
    }
}
