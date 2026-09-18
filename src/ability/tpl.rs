use crate::ability::prelude::*;
use crate::execution::action::Action;
use handlebars::{Handlebars, to_json};
use orion_error::OperationContext;
use orion_error::conversion::{SourceErr, SourceRawErr};
use serde::Serialize;
use std::fmt::Display;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Default, Debug, PartialEq, Getters, Clone)]
pub struct GxTpl {
    dto: TplDTO,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum TPlEngineType {
    #[default]
    Handlebars,
    Helm,
}
impl FromStr for TPlEngineType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "handlebars" => Ok(Self::Handlebars),
            "helm" => Ok(Self::Helm),
            _ => Err("unknow engine".to_string()),
        }
    }
}
impl Display for TPlEngineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            TPlEngineType::Handlebars => "handlebars",
            TPlEngineType::Helm => "helm",
        };
        write!(f, "{msg}")
    }
}

#[derive(Clone, Debug, PartialEq, Builder, Default)]
pub struct TplDTO {
    pub tpl: String,
    pub dst: String,
    #[builder(default = "None")]
    pub data: Option<String>,
    #[builder(default = "None")]
    pub file: Option<String>,
    #[builder(default = "TPlEngineType::Handlebars")]
    pub engine: TPlEngineType,
}
impl From<TplDTO> for GxTpl {
    fn from(value: TplDTO) -> Self {
        Self { dto: value }
    }
}
impl Display for TplDTO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[tpl] {},", self.tpl)?;
        write!(f, "[dst] {},", self.dst)?;
        write!(f, "[engine] {},", self.engine)?;
        write!(
            f,
            "[data|file|env]: {},",
            self.data
                .clone()
                .or(self.file.clone())
                .unwrap_or("env".to_string())
        )?;
        Ok(())
    }
}

impl GxTpl {
    pub fn render_path(&self, ctx: ExecContext, dto: &TplDTO, dict: VarSpace) -> ExecResult<()> {
        info!(target: ctx.path(), "gx.tpl : {dto}" );
        let exp = EnvExpress::from_env_mix(dict.global().clone());
        let tpl = PathBuf::from(exp.eval(dto.tpl.as_str())?);
        let dst = PathBuf::from(exp.eval(dto.dst.as_str())?);

        let mut err_ctx = OperationContext::doing("render tpl path").with_auto_log();
        // 处理目录模板
        if dto.engine != TPlEngineType::Handlebars {
            return Err(ExecReason::Args
                .to_err()
                .with_detail(format!("Only support Handlebars Engine {:?}", dto.engine)));
        }
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        // 准备数据

        let data = if let Some(json_file) = &dto.file {
            let json_file = exp.eval(json_file.as_str())?;
            err_ctx.record("file", json_file.as_str());
            let content = std::fs::read_to_string(json_file.as_str())
                .source_err(UvsReason::data_error().into(), "source error")
                .with_context(&err_ctx)?;
            err_ctx.record("need-fmt", "json");
            serde_json::from_str(content.as_str())
                .source_raw_err(ExecReason::data_error(), "parse json data file")
                .with_context(&err_ctx)?
        } else if let Some(data_str) = &dto.data {
            serde_json::from_str(data_str.as_str())
                .source_raw_err(ExecReason::data_error(), "parse inline json data")
                .with_context(&err_ctx)?
        } else {
            to_json(dict.global().export())
        };
        let res = if tpl.is_dir() {
            self.render_dir_impl(ctx, &handlebars, &tpl, &dst, &data)
        } else {
            self.render_file_impl(ctx, &handlebars, &tpl, &dst, &data)
        };
        if res.is_ok() {
            err_ctx.mark_suc();
        }
        res
    }
    fn render_dir_impl<T: Serialize>(
        &self,
        ctx: ExecContext,
        handlebars: &Handlebars,
        tpl_dir: &PathBuf,
        dst: &PathBuf,
        data: &T,
    ) -> ExecResult<()> {
        debug!(target: ctx.path(), "tpl dir: {}", tpl_dir.display());
        for entry in walkdir::WalkDir::new(tpl_dir) {
            let entry = entry.source_raw_err(ExecReason::data_error(), "walk template dir")?;
            let entry_path = entry.path();
            let relative_path = entry_path
                .strip_prefix(tpl_dir)
                .source_raw_err(ExecReason::data_error(), "strip template dir prefix")?;
            let dst_path = Path::new(dst).join(relative_path);

            if entry_path.is_dir() {
                // 如果是目录，确保在目标位置创建对应的目录
                std::fs::create_dir_all(&dst_path)
                    .source_err(UvsReason::system_error().into(), "source error")?;
                debug!(target: ctx.path(), "created dir: {}", dst_path.display());
            } else if entry_path.is_file() {
                // 如果是文件，则渲染模板
                self.render_file_impl(
                    ctx.clone(),
                    handlebars,
                    &entry_path.to_path_buf(),
                    &dst_path,
                    &data,
                )?;
            }
            // 忽略其他类型（如符号链接等）
        }
        Ok(())
    }

    fn render_file_impl<T: Serialize>(
        &self,
        ctx: ExecContext,
        handlebars: &Handlebars,
        tpl: &PathBuf,
        dst: &PathBuf,
        data: &T,
    ) -> ExecResult<()> {
        debug!(target: ctx.path(), "tpl:{}", tpl.display());
        debug!(target: ctx.path(),  "dst:{}", dst.display());

        let mut err_ctx = OperationContext::doing("render tpl").with_auto_log();
        err_ctx.record("tpl", tpl.display());
        // 2. 验证模板文件
        let tpl_path = Path::new(&tpl);
        if !tpl_path.exists() {
            return Err(ExecReason::Args
                .to_err()
                .with_detail(format!("Template file not found: {}", tpl.display())));
        }
        if !tpl_path.is_file() {
            return Err(ExecReason::Args
                .to_err()
                .with_detail(format!("Template path is not a file: {}", tpl.display())));
        }
        err_ctx.record("dst", dst.display());
        // 3. 准备目标文件
        let dst_path = Path::new(&dst);
        if let Some(parent) = dst_path.parent() {
            std::fs::create_dir_all(parent)
                .source_err(UvsReason::system_error().into(), "source error")?;
        }
        if dst_path.exists() {
            std::fs::remove_file(dst)
                .source_err(UvsReason::system_error().into(), "source error")?;
        }

        // 4. 日志记录
        debug!(target: ctx.path(), "Processing template: {} → {}", tpl.display(), dst.display());

        // 5. 读取模板内容
        let template = std::fs::read_to_string(tpl)
            .source_err(UvsReason::data_error().into(), "source error")
            .with_context(&err_ctx)?;

        let mut dst_file = File::create(dst).source_err(
            ExecReason::Io,
            format!("create output file: {}", dst.display()),
        )?;

        handlebars
            .render_template_to_write(&template, data, &mut dst_file)
            .source_raw_err(ExecReason::business_error(), "render handlebars template")
            .with_context(&err_ctx)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o644); // rw-r--r--
            std::fs::set_permissions(dst, perms)
                .source_err(UvsReason::system_error().into(), "source error")
                .with_context(&err_ctx)?;
        }
        if !ctx.quiet() {
            eprintln!("render {:30} ---> {}", tpl.display(), dst.display());
        }

        debug!(target: ctx.path(), "Successfully generated: {}", dst.display());
        err_ctx.mark_suc();
        Ok(())
    }
}

#[async_trait]
impl AsyncRunnableTrait for GxTpl {
    async fn async_exec(&self, ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        let mut action = Action::from("build tpl file");
        self.render_path(ctx, &self.dto, vars_dict.clone())?;
        action.finish();
        Ok(TaskValue::from((vars_dict, ExecOut::Action(action))))
    }
}

impl ComponentMeta for GxTpl {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.tpl")
    }
}

impl GxTpl {
    pub fn new(tpl: String, dst: String) -> Self {
        let obj = TplDTO {
            tpl,
            dst,
            ..Default::default()
        };
        GxTpl { dto: obj }
    }
    pub fn new_by_dto(dto: TplDTO) -> Self {
        GxTpl { dto }
    }
}

#[cfg(test)]
mod tests {

    use crate::{ability::ability_env_init, const_val::gxl_const, traits::Setter};

    use super::*;
    fn files_identical(path1: &str, path2: &str) -> std::io::Result<bool> {
        let content1 = std::fs::read(path1)?;
        let content2 = std::fs::read(path2)?;
        Ok(content1 == content2)
    }

    #[tokio::test]
    async fn tpl_test_by_envars() {
        let (mut context, mut def) = ability_env_init();
        let tpl = format!(
            "{}/examples/template/conf/tpls/nginx.conf",
            context.cur_path()
        );
        let dst = format!(
            "{}/examples/template/conf/used/nginx_env.conf",
            context.cur_path()
        );
        let conf_tpl = GxTpl::new(tpl.clone(), dst.clone());
        context.append("RG");
        def.global_mut().set(gxl_const::PRJ_ROOT, "/home/galaxy");
        def.global_mut().set("DOMAIN", "www.galaxio-labs.org");
        def.global_mut().set("SOCK_FILE", "galaxy.socket");
        conf_tpl.async_exec(context.clone(), def).await.unwrap();

        let ngx_dst = format!(
            "{}/examples/template/conf/used/nginx_env.conf",
            context.cur_path()
        );
        let ngx_xpt = format!(
            "{}/examples/template/conf/expect/nginx.conf",
            context.cur_path()
        );
        assert!(files_identical(ngx_dst.as_str(), ngx_xpt.as_str()).unwrap());
    }
    #[tokio::test]
    async fn tpl_test_by_json() {
        //use ValueDict
        let (context, def) = ability_env_init();
        let root = format!("{}/examples/template/conf", context.cur_path());
        let tpl = format!("{root}/tpls",);
        let dst = format!("{root}/used",);
        let file = format!("{root}/value.json",);

        let dto = TplDTO {
            tpl: tpl.clone(),
            dst: dst.clone(),
            file: Some(file.clone()),
            ..Default::default()
        };

        let conf_tpl = GxTpl::new_by_dto(dto);
        conf_tpl.async_exec(context.clone(), def).await.unwrap();

        let ngx_dst = format!("{root}/used/nginx.conf",);
        let ngx_xpt = format!("{root}/expect/nginx.conf",);
        assert!(files_identical(ngx_dst.as_str(), ngx_xpt.as_str()).unwrap());

        let sys_dst = format!("{root}/used/sys.toml",);
        let sys_tpl = format!("{root}/tpls/sys.toml",);
        assert!(files_identical(sys_dst.as_str(), sys_tpl.as_str()).unwrap());
    }

    #[test]
    fn handlebar_test() {
        let mut handlebars = Handlebars::new();
        let source = "hello {{WORLD}} {{PRJ_HOME}}!";
        assert!(handlebars.register_template_string("t1", source).is_ok());
        let mut def = VarSpace::default();
        def.global_mut().set("world", "世界!".to_string());
        def.global_mut().set("PRJ_HOME", "home".to_string());
        let data = def.global().export();
        assert_eq!(handlebars.render("t1", &data).unwrap(), "hello 世界! home!");

        let data = r#"
        {
            "WORLD": "John",
            "PRJ_HOME": "home/bj",
            "AGE": 43
        }"#;
        let v: serde_json::Value = serde_json::from_str(data).unwrap();
        assert_eq!(handlebars.render("t1", &v).unwrap(), "hello John home/bj!");
    }
    #[test]
    fn handlebar_test1() {
        let mut handlebars = Handlebars::new();
        let source = "hello {{PRJ.HOME}}!";
        assert!(handlebars.register_template_string("t1", source).is_ok());
        let data = r#"
        {
            "PRJ" : { "HOME": "home" }
        }"#;
        let v: serde_json::Value = serde_json::from_str(data).unwrap();
        assert_eq!(handlebars.render("t1", &v).unwrap(), "hello home!");
    }
}
