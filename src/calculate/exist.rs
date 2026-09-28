use orion_sec::sec::SecValueType;

use crate::{
    calculate::traits::{DecideResult, Evaluation},
    const_val::gxl_const::full_flow_name,
    context::ExecContext,
    evaluator::{EnvExpress, VarParser},
    execution::VarSpace,
    primitive::GxlObject,
};

/// 存在性判定表达式：`gx.exists(flow: "localize")`（也接受裸流程名 `gx.exists("localize")`
/// 或变量 `gx.exists(${P})`）。
///
/// 用于「有则调用、无则跳过」，配合条件块：
///
/// ```gxl
/// if gx.exists(flow: "localize") {
///     gx.run(local: ".", env: "default", flow: "localize") ;
/// }
/// ```
///
/// 判定依据是当前已装配空间的流程名集合（`ExecContext::flow_names`），
/// **不解析 extern、不联网、无副作用**；不存在返回 `false`（不是错误）。
#[derive(Clone, Debug, PartialEq)]
pub struct FnExists {
    flow: GxlObject,
}

impl FnExists {
    pub fn new(flow: GxlObject) -> Self {
        Self { flow }
    }
    pub fn flow(&self) -> &GxlObject {
        &self.flow
    }
}

/// 由参数对象求出流程名：字面量直接取用；变量 `${P}` 先从变量空间解析，
/// 未命中再回退进程环境变量（与 `defined(...)` 一致）；最后统一做 `${}` 插值
/// （与 `gx.run(flow: "${P}")` 一致）。
fn resolve_flow_name(obj: &GxlObject, args: &VarSpace) -> Option<String> {
    let raw = match obj {
        GxlObject::Value(SecValueType::String(s)) => s.value().to_string(),
        GxlObject::VarRef(name) => match args.get(name) {
            Some(SecValueType::String(s)) => s.value().to_string(),
            Some(_) => return None,
            None => std::env::var(name).ok()?,
        },
        _ => return None,
    };
    EnvExpress::from_env_mix(args.global().clone())
        .eval(&raw)
        .ok()
}

impl Evaluation for FnExists {
    fn decide(&self, ctx: ExecContext, args: &VarSpace) -> DecideResult {
        let Some(name) = resolve_flow_name(&self.flow, args) else {
            return Ok(false);
        };
        Ok(ctx.flow_names().contains(&full_flow_name(&name)))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::Arc;

    use orion_sec::sec::SecFrom;

    use crate::loader::GxLoader;
    use crate::traits::Setter;
    use crate::util::path::WorkDirWithLock;

    use super::*;

    fn ctx_with(names: &[&str]) -> ExecContext {
        let set: HashSet<String> = names.iter().map(|s| s.to_string()).collect();
        ExecContext::default().with_flow_names(Arc::new(set))
    }

    #[test]
    fn exists_literal_hit_and_miss() {
        // 集合与运行期一致，只放限定名；查询侧用 full_flow_name 归一化
        let ctx = ctx_with(&["main.localize", "ops.install"]);
        assert!(
            FnExists::new(GxlObject::from_val("localize"))
                .decide(ctx.clone(), &VarSpace::default())
                .unwrap()
        );
        assert!(
            !FnExists::new(GxlObject::from_val("no_such_flow"))
                .decide(ctx, &VarSpace::default())
                .unwrap()
        );
    }

    #[test]
    fn exists_qualified_name() {
        let ctx = ctx_with(&["ops.install"]);
        assert!(
            FnExists::new(GxlObject::from_val("ops.install"))
                .decide(ctx, &VarSpace::default())
                .unwrap()
        );
    }

    #[test]
    fn exists_dotted_main_name_stays_consistent_with_runtime() {
        // main 里名为 `a.b` 的流程，限定名是 `main.a.b`；查询 `a.b` 应视为 mod=a/flow=b
        // 而不是命中 —— 与运行期 `gx run a.b` 的解析一致（都为「不存在」）。
        let ctx = ctx_with(&["main.a.b"]);
        assert!(
            FnExists::new(GxlObject::from_val("main.a.b"))
                .decide(ctx.clone(), &VarSpace::default())
                .unwrap()
        );
        assert!(
            !FnExists::new(GxlObject::from_val("a.b"))
                .decide(ctx, &VarSpace::default())
                .unwrap()
        );
    }

    #[test]
    fn exists_var_ref_resolved() {
        let mut vars = VarSpace::default();
        vars.global_mut()
            .set("P", SecValueType::nor_from("localize".to_string()));
        let ctx = ctx_with(&["main.localize"]);
        let f = FnExists::new(GxlObject::from_ref("P"));
        assert!(f.decide(ctx, &vars).unwrap());
    }

    #[test]
    fn exists_string_literal_is_interpolated() {
        let mut vars = VarSpace::default();
        vars.global_mut()
            .set("CAND", SecValueType::nor_from("localize".to_string()));
        let ctx = ctx_with(&["main.localize"]);
        let f = FnExists::new(GxlObject::from_val("${CAND}"));
        assert!(f.decide(ctx, &vars).unwrap());
    }

    #[test]
    fn exists_missing_var_is_false_not_error() {
        let ctx = ctx_with(&["main.localize"]);
        let f = FnExists::new(GxlObject::from_ref("UNDEFINED_P_XYZ"));
        assert!(!f.decide(ctx, &VarSpace::default()).unwrap());
    }

    /// 用真实 `GxlSpace` 校验：`flow_names()`（判定来源）与 `has_flow()`（CLI/运行期）
    /// 语义一致，且集合只含限定名。
    #[tokio::test]
    async fn has_flow_agrees_with_flow_names() {
        let dir = tempfile::tempdir().unwrap();
        let _wd = WorkDirWithLock::change(dir.path()).unwrap();
        let loader = GxLoader::new();
        let code = "mod envs { env default {} }\n\
                    mod ops { flow install {} }\n\
                    mod main { flow localize {} }\n";
        let vars = VarSpace::sys_init().unwrap();
        let spc = loader
            .parse_code(code, false, &vars, None)
            .await
            .unwrap()
            .assemble()
            .unwrap();

        let names = spc.flow_names();
        assert!(names.contains("main.localize"));
        assert!(names.contains("ops.install"));
        assert!(!names.contains("localize"), "集合应为限定名，不含裸名");

        assert!(spc.has_flow("localize"));
        assert!(spc.has_flow("ops.install"));
        assert!(!spc.has_flow("nope"));
        // 两条判定路径一致
        assert_eq!(
            spc.has_flow("localize"),
            names.contains(&full_flow_name("localize"))
        );
    }
}
