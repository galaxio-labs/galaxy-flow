use orion_error::conversion::{SourceErr, ToStructError};

use crate::ability::prelude::*;

use crate::execution::runnable::ComponentMeta;
use crate::parser::domain::take_version;
use crate::traits::Setter;
use std::cmp::Ordering;
use std::fmt;
use std::fs::{self, File};
use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub struct Version {
    main: i32,
    feature: i32,
    bugfix: i32,
    build: Option<i32>,
}

impl Version {}
impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.main == other.main
            && self.feature == other.feature
            && self.bugfix == other.bugfix
            && self.build == other.build
        {
            Some(Ordering::Equal)
        } else if self.main > other.main {
            Some(Ordering::Greater)
        } else if self.main < other.main {
            Some(Ordering::Less)
        } else if self.feature > other.feature {
            Some(Ordering::Greater)
        } else if self.feature < other.feature {
            Some(Ordering::Less)
        } else if self.bugfix > other.bugfix {
            Some(Ordering::Greater)
        } else if self.bugfix < other.bugfix {
            Some(Ordering::Less)
        } else if self.build > other.build {
            Some(Ordering::Greater)
        } else if self.build < other.build {
            Some(Ordering::Less)
        } else {
            None
        }
    }
}

impl Version {
    pub fn new(main: i32, feature: i32, bugfix: i32, build: Option<i32>) -> Self {
        Version {
            main,
            feature,
            bugfix,
            build,
        }
    }
    pub fn auto(&mut self, inc: &VerInc) {
        match *inc {
            VerInc::Main => {
                self.main += 1;
                self.feature = 0;
                self.bugfix = 0;
                self.build = self.build.map(|x| x + 1);
            }
            VerInc::Feature => {
                self.feature += 1;
                self.bugfix = 0;
                self.build = self.build.map(|x| x + 1);
            }
            VerInc::Bugfix => {
                self.bugfix += 1;
                self.build = self.build.map(|x| x + 1);
            }
            VerInc::Build => {
                self.build = self.build.map(|x| x + 1);
            }
            _ => {}
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.build {
            Some(build) => write!(
                f,
                "{}.{}.{}.{}",
                self.main, self.feature, self.bugfix, build
            ),
            None => write!(f, "{}.{}.{}", self.main, self.feature, self.bugfix),
        }
    }
}

#[derive(Debug, Builder, PartialEq, Clone)]
pub struct GxlVersion {
    file: String,
    export: String,
    verinc: VerInc,
}
impl GxlVersion {
    pub fn new(file: String) -> GxlVersion {
        GxlVersion {
            file,
            export: "VERSION".into(),
            verinc: VerInc::Build,
        }
    }
}
#[async_trait]
impl AsyncRunnableTrait for GxlVersion {
    async fn async_exec(&self, mut ctx: ExecContext, mut dict: VarSpace) -> TaskResult {
        ctx.append("version");
        let exp = EnvExpress::from_env_mix(dict.global().clone());
        let file_path = exp.eval(&self.file)?;
        debug!(target: ctx.path(),"version file:{file_path}");
        let data = fs::read_to_string(file_path.as_str())
            .source_err(UvsReason::business_error().into(), "source error")
            .with_context(format!("version file ({file_path}) "))?;
        match take_version(&mut data.as_str()) {
            Ok((a, b, c, d)) => {
                let mut ver = Version::new(a, b, c, d);
                ver.auto(&self.verinc);
                dict.global_mut()
                    .set(&self.export.to_uppercase(), format!("{ver}"));
                let mut file = File::create(file_path.as_str())
                    .source_err(UvsReason::resource_error().into(), "source error")?;
                file.write_all(ver.to_string().as_bytes())
                    .source_err(UvsReason::resource_error().into(), "source error")?;
                Ok(TaskValue::from((dict, ExecOut::Ignore)))
            }
            Err(_) => Err(ExecReason::from_conf()
                .to_err()
                .with_detail("version file parse failed!")),
        }
    }
}

pub fn parse_version(data: &str) -> ExecResult<Version> {
    let mut xdata = data;
    let (a, b, c, d) = take_version(&mut xdata).map_err(|err| {
        ExecReason::Args
            .to_err()
            .with_detail(format!("version parse failed: {err}"))
    })?;
    Ok(Version::new(a, b, c, d))
}

impl ComponentMeta for GxlVersion {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.ver")
    }
}

#[derive(Debug, Builder, PartialEq, Clone)]
pub struct GxlSn {
    file: String,
    export: String,
    action: SnAction,
}

impl GxlSn {
    pub fn new(file: String) -> GxlSn {
        GxlSn {
            file,
            export: "SN".into(),
            action: SnAction::Read,
        }
    }
}

#[async_trait]
impl AsyncRunnableTrait for GxlSn {
    async fn async_exec(&self, mut ctx: ExecContext, mut dict: VarSpace) -> TaskResult {
        ctx.append("sn");
        let exp = EnvExpress::from_env_mix(dict.global().clone());
        let file_path = exp.eval(&self.file)?;
        debug!(target: ctx.path(), "sn file:{file_path}");
        let sn = match self.action {
            SnAction::Read | SnAction::Add => {
                let data = fs::read_to_string(file_path.as_str())
                    .source_err(UvsReason::business_error().into(), "source error")
                    .with_context(format!("sn file ({file_path}) "))?;
                let current = parse_sn(data.as_str())?;
                match self.action {
                    SnAction::Read => current,
                    SnAction::Add => current + 1,
                    SnAction::Reset => unreachable!(),
                }
            }
            SnAction::Reset => 1,
        };
        dict.global_mut()
            .set(&self.export.to_uppercase(), sn.to_string());
        if self.action != SnAction::Read {
            let mut file = File::create(file_path.as_str())
                .source_err(UvsReason::resource_error().into(), "source error")?;
            file.write_all(sn.to_string().as_bytes())
                .source_err(UvsReason::resource_error().into(), "source error")?;
        }
        Ok(TaskValue::from((dict, ExecOut::Ignore)))
    }
}

impl ComponentMeta for GxlSn {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.sn")
    }
}

pub fn parse_sn(data: &str) -> ExecResult<i32> {
    let value = data.trim().parse::<i32>().map_err(|err| {
        ExecReason::Args
            .to_err()
            .with_detail(format!("sn parse failed: {err}"))
    })?;
    if value < 1 {
        return Err(ExecReason::Args
            .to_err()
            .with_detail(format!("sn must be >= 1: {value}")));
    }
    Ok(value)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnAction {
    Read,
    Add,
    Reset,
}

pub fn parse_sn_action(data: &str) -> ExecResult<SnAction> {
    match data.to_lowercase().as_str() {
        "read" | "get" | "null" => Ok(SnAction::Read),
        "add" | "inc" => Ok(SnAction::Add),
        "reset" => Ok(SnAction::Reset),
        other => Err(ExecReason::Args
            .to_err()
            .with_detail(format!("unsupported sn action: {other}"))),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerInc {
    Build,
    Bugfix,
    Feature,
    Main,
    Null,
}

#[cfg(test)]
mod tests {
    use fs::File;

    use crate::traits::Getter;
    use crate::types::AnyResult;
    use orion_error::dev::testing::TestAssert;

    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn version_test() {
        let mut file = File::create("./tests/tmp_version.txt").unwrap();
        file.write_all(b"0.1.0.0").unwrap();
        let ver = GxlVersion::new("./tests/tmp_version.txt".into());
        let ctx = ExecContext::default();
        let def = VarSpace::default();
        ver.async_exec(ctx.clone(), def).await.unwrap();
    }

    #[test]
    fn basic_version_test() -> AnyResult<()> {
        let versions = vec![
            (
                "3.4.5",
                Version {
                    main: 3,
                    feature: 4,
                    bugfix: 5,
                    build: None,
                },
            ),
            (
                "6.7.8.9",
                Version {
                    main: 6,
                    feature: 7,
                    bugfix: 8,
                    build: Some(9),
                },
            ),
        ];

        for (input, expected) in versions {
            let parsed = parse_version(input).assert();
            assert_eq!(parsed, expected);
        }
        Ok(())
    }

    #[test]
    fn version_increment_test() {
        // 主版本递增
        let mut ver = Version {
            main: 1,
            feature: 1,
            bugfix: 1,
            build: Some(0),
        };
        ver.auto(&VerInc::Main);
        assert_eq!(ver.main, 2);
        assert_eq!(ver.feature, 0);
        assert_eq!(ver.bugfix, 0);

        // 特性版本递增
        let mut ver = Version {
            main: 1,
            feature: 2,
            bugfix: 1,
            build: None,
        };
        ver.auto(&VerInc::Feature);
        assert_eq!(ver.feature, 3);
        assert_eq!(ver.bugfix, 0);

        // 修复版本递增
        let mut ver = Version {
            main: 1,
            feature: 2,
            bugfix: 3,
            build: Some(4),
        };
        ver.auto(&VerInc::Bugfix);
        assert_eq!(ver.bugfix, 4);
        assert_eq!(ver.feature, 2);

        // 构建版本递增
        let mut ver = Version {
            main: 1,
            feature: 2,
            bugfix: 3,
            build: Some(5),
        };
        ver.auto(&VerInc::Build);
        assert_eq!(ver.build.unwrap(), 6);
    }

    #[test]
    fn version_comparison_test() {
        // Equal versions
        let v1 = Version::new(1, 2, 3, Some(4));
        let v2 = Version::new(1, 2, 3, Some(4));
        assert_eq!(v1.partial_cmp(&v2), Some(Ordering::Equal));

        // Main version comparison
        let v1 = Version::new(2, 0, 0, None);
        let v2 = Version::new(1, 0, 0, None);
        assert_eq!(v1.partial_cmp(&v2), Some(Ordering::Greater));
        assert_eq!(v2.partial_cmp(&v1), Some(Ordering::Less));

        // Feature version comparison
        let v1 = Version::new(1, 2, 0, None);
        let v2 = Version::new(1, 1, 0, None);
        assert_eq!(v1.partial_cmp(&v2), Some(Ordering::Greater));
        assert_eq!(v2.partial_cmp(&v1), Some(Ordering::Less));

        // Bugfix version comparison
        let v1 = Version::new(1, 1, 2, None);
        let v2 = Version::new(1, 1, 1, None);
        assert_eq!(v1.partial_cmp(&v2), Some(Ordering::Greater));
        assert_eq!(v2.partial_cmp(&v1), Some(Ordering::Less));

        // Build version comparison
        let v1 = Version::new(1, 1, 1, Some(2));
        let v2 = Version::new(1, 1, 1, Some(1));
        assert_eq!(v1.partial_cmp(&v2), Some(Ordering::Greater));
        assert_eq!(v2.partial_cmp(&v1), Some(Ordering::Less));
    }

    #[test]
    fn sn_parse_test() {
        assert_eq!(parse_sn("1").assert(), 1);
        assert_eq!(parse_sn(" 42\n").assert(), 42);
        assert!(parse_sn("0").is_err());
        assert!(parse_sn("abc").is_err());
    }

    #[tokio::test]
    async fn sn_read_add_and_reset_test() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("sn.txt");
        let file = file_path.to_string_lossy().to_string();
        fs::write(&file_path, "1").unwrap();

        let sn = GxlSn::new(file.clone());
        let ctx = ExecContext::default();
        let def = VarSpace::default();
        let TaskValue { vars, .. } = sn.async_exec(ctx.clone(), def).await.unwrap();
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "1");
        assert_eq!(
            vars.global().get_copy("SN").unwrap().to_string().as_str(),
            "1"
        );

        let add = GxlSnBuilder::default()
            .file(file.clone())
            .export("SN".into())
            .action(SnAction::Add)
            .build()
            .unwrap();
        let def = VarSpace::default();
        let TaskValue { vars, .. } = add.async_exec(ctx.clone(), def).await.unwrap();
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "2");
        assert_eq!(
            vars.global().get_copy("SN").unwrap().to_string().as_str(),
            "2"
        );

        let reset = GxlSnBuilder::default()
            .file(file)
            .export("SN".into())
            .action(SnAction::Reset)
            .build()
            .unwrap();
        let def = VarSpace::default();
        let TaskValue { vars, .. } = reset.async_exec(ctx, def).await.unwrap();
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "1");
        assert_eq!(
            vars.global().get_copy("SN").unwrap().to_string().as_str(),
            "1"
        );

        fs::remove_file(&file_path).unwrap();
        let reset = GxlSnBuilder::default()
            .file(file_path.to_string_lossy().to_string())
            .export("SN".into())
            .action(SnAction::Reset)
            .build()
            .unwrap();
        let def = VarSpace::default();
        let TaskValue { vars, .. } = reset.async_exec(ExecContext::default(), def).await.unwrap();
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "1");
        assert_eq!(
            vars.global().get_copy("SN").unwrap().to_string().as_str(),
            "1"
        );
    }
}
