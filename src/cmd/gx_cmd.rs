use clap::{ArgAction, Args, Parser, Subcommand};
use derive_getters::Getters;

use crate::cmd::gxl_cmd::GFlowCmd;

#[derive(Debug, Args, Clone)]
#[command(about = "run workflow flows from the default work config")]
#[command(
    after_help = "Examples:\n  gx run -e dev -c ./_gal/work.gxl conf\n  gx run -e prod --cmd-arg \"-x -y\" conf\n  gx run -e test --dryrun conf\n\n示例：\n  gx run -e dev -c ./_gal/work.gxl conf\n  gx run -e prod --cmd-arg \"-x -y\" conf\n  gx run -e test --dryrun conf"
)]
pub struct RunCmd {
    #[command(flatten)]
    pub cmd: GFlowCmd,
}

#[derive(Debug, Args, Clone)]
#[command(about = "run admin flows from the default admin config")]
#[command(
    after_help = "Examples:\n  gx adm -e dev -c ./_gal/adm.gxl conf\n  gx adm -e prod --cmd-arg \"-x -y\" conf\n  gx adm -e test --dryrun conf\n\n示例：\n  gx adm -e dev -c ./_gal/adm.gxl conf\n  gx adm -e prod --cmd-arg \"-x -y\" conf\n  gx adm -e test --dryrun conf"
)]
pub struct AdmCmd {
    #[command(flatten)]
    pub cmd: GFlowCmd,
}

#[derive(Debug, Parser, Clone)]
#[command(name = "gx")]
#[command(version, about = "Galaxy Flow unified CLI")]
#[command(disable_help_subcommand = true)]
pub enum GxCmd {
    /// run workflow flows from the default work config
    Run(RunCmd),
    /// run admin flows from the default admin config
    Adm(AdmCmd),
    /// initialize local environment or project scaffolding
    #[command(subcommand)]
    Init(InitCmd),
    /// manage project modules
    #[command(subcommand)]
    Mod(ModCmd),
    /// show built-in documentation topics
    #[command(name = "doc", alias = "docs")]
    Doc(DocArgs),
    /// print current runtime environment information
    Check,
    /// manage gx self-update operations
    #[command(name = "self", subcommand)]
    SelfUpdate(SelfCmd),
}

#[derive(Debug, Subcommand, Clone)]
pub enum InitCmd {
    /// init local Galaxy environment (~/.galaxy, conf.toml, net access control)
    Env,
    /// init project (local by default, use --path or --repo for remote)
    Project(InitArgs),
}

#[derive(Debug, Subcommand, Clone)]
pub enum ModCmd {
    /// update project modules defined by local _gal configs
    Update(PrjArgs),
}

#[derive(Debug, Args, Clone, Getters)]
pub struct DocArgs {
    #[arg(long, action = ArgAction::SetTrue, default_value = "false")]
    pub markdown: bool,
    pub topic: Option<String>,
}

#[derive(Debug, Subcommand, Clone)]
pub enum SelfCmd {
    Status,
    Check(SelfCheckArgs),
    Update(SelfUpdateArgs),
    Rollback(SelfRollbackArgs),
}

#[derive(Debug, Args, Clone, Getters)]
pub struct SelfCheckArgs {
    #[arg(long, default_value = "stable")]
    pub channel: String,
    #[arg(long, action = ArgAction::SetTrue, default_value = "false")]
    pub json: bool,
}

#[derive(Debug, Args, Clone, Getters)]
pub struct SelfUpdateArgs {
    #[arg(long, default_value = "stable")]
    pub channel: String,
    #[arg(long = "to")]
    pub to_version: Option<String>,
    #[arg(long, action = ArgAction::SetTrue, default_value = "false")]
    pub yes: bool,
    #[arg(long = "dry-run", action = ArgAction::SetTrue, default_value = "false")]
    pub dry_run: bool,
    #[arg(long, action = ArgAction::SetTrue, default_value = "false")]
    pub force: bool,
}

#[derive(Debug, Args, Clone, Getters)]
pub struct SelfRollbackArgs {
    #[arg(long = "id")]
    pub backup_id: Option<String>,
}

#[derive(Debug, Args, Getters, Clone)]
pub struct InitArgs {
    /// git repository URL. Used with --path for remote init.
    #[arg(long)]
    pub(crate) repo: Option<String>,
    /// subdirectory path within the repository.
    /// When used without --repo, uses default repo: https://github.com/galaxio-labs/prj-tpl.git
    #[arg(long)]
    pub(crate) path: Option<String>,
    /// branch for git repository (requires --repo or --path)
    #[arg(short, long, conflicts_with = "tag")]
    pub(crate) branch: Option<String>,
    /// tag for git repository (requires --repo or --path)
    #[arg(long, conflicts_with = "branch")]
    pub(crate) tag: Option<String>,
    /// debug level ; eg: -d 1
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub(crate) debug: usize,
    #[arg(long = "log")]
    pub log: Option<String>,
}

#[derive(Debug, Args, Clone)]
pub struct PrjArgs {
    /// debug level ; eg: -d 1
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub(crate) debug: usize,
    #[arg(long = "log")]
    pub log: Option<String>,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{AdmCmd, GxCmd, InitCmd, ModCmd, RunCmd, SelfCmd};

    #[test]
    fn parse_doc_topic() {
        let cmd = GxCmd::try_parse_from(["gx", "doc", "gx.cmd"]).expect("doc command should parse");

        match cmd {
            GxCmd::Doc(args) => {
                assert_eq!(args.topic.as_deref(), Some("gx.cmd"));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parse_doc_markdown() {
        let cmd = GxCmd::try_parse_from(["gx", "doc", "--markdown", "gx.cmd"])
            .expect("doc markdown command should parse");

        match cmd {
            GxCmd::Doc(args) => {
                assert!(args.markdown);
                assert_eq!(args.topic.as_deref(), Some("gx.cmd"));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parse_mod_update() {
        let cmd = GxCmd::try_parse_from(["gx", "mod", "update"]).expect("mod update should parse");

        match cmd {
            GxCmd::Mod(ModCmd::Update(_args)) => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parse_run_wrapper() {
        let cmd = GxCmd::try_parse_from(["gx", "run", "conf"]).expect("run should parse");

        match cmd {
            GxCmd::Run(RunCmd { cmd }) => {
                assert_eq!(cmd.flows, vec!["conf".to_string()]);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parse_adm_wrapper() {
        let cmd = GxCmd::try_parse_from(["gx", "adm", "conf"]).expect("adm should parse");

        match cmd {
            GxCmd::Adm(AdmCmd { cmd }) => {
                assert_eq!(cmd.flows, vec!["conf".to_string()]);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn reject_removed_init_cmd_print_flag() {
        let result = GxCmd::try_parse_from(["gx", "init", "project", "--cmd_print"]);
        assert!(result.is_err(), "removed --cmd_print flag should not parse");
    }

    #[test]
    fn reject_removed_mod_update_quiet_flag() {
        let result = GxCmd::try_parse_from(["gx", "mod", "update", "--quiet"]);
        assert!(result.is_err(), "removed --quiet flag should not parse");
    }

    #[test]
    fn reject_removed_mod_update_conf_flags() {
        let result = GxCmd::try_parse_from(["gx", "mod", "update", "--conf-work", "x"]);
        assert!(result.is_err(), "removed --conf-work flag should not parse");

        let result = GxCmd::try_parse_from(["gx", "mod", "update", "--conf-adm", "x"]);
        assert!(result.is_err(), "removed --conf-adm flag should not parse");
    }

    #[test]
    fn parse_init_project() {
        // no args = local init (repo/path are None)
        let cmd =
            GxCmd::try_parse_from(["gx", "init", "project"]).expect("init project should parse");
        match cmd {
            GxCmd::Init(InitCmd::Project(args)) => {
                assert_eq!(args.repo(), &None);
                assert_eq!(args.path(), &None);
            }
            other => panic!("unexpected command: {other:?}"),
        }

        // --repo only
        let cmd = GxCmd::try_parse_from([
            "gx",
            "init",
            "project",
            "--repo",
            "https://github.com/user/repo.git",
        ])
        .expect("init project with repo should parse");
        match cmd {
            GxCmd::Init(InitCmd::Project(args)) => {
                assert_eq!(
                    args.repo(),
                    &Some("https://github.com/user/repo.git".to_string())
                );
                assert_eq!(args.path(), &None);
            }
            other => panic!("unexpected command: {other:?}"),
        }

        // --repo with --path
        let cmd = GxCmd::try_parse_from([
            "gx",
            "init",
            "project",
            "--repo",
            "https://github.com/user/repo.git",
            "--path",
            "rust",
        ])
        .expect("init project with repo and path should parse");
        match cmd {
            GxCmd::Init(InitCmd::Project(args)) => {
                assert_eq!(
                    args.repo(),
                    &Some("https://github.com/user/repo.git".to_string())
                );
                assert_eq!(args.path(), &Some("rust".to_string()));
            }
            other => panic!("unexpected command: {other:?}"),
        }

        // --path only (default repo is applied at runtime)
        let cmd = GxCmd::try_parse_from(["gx", "init", "project", "--path", "rust"])
            .expect("init project with path should parse");
        match cmd {
            GxCmd::Init(InitCmd::Project(args)) => {
                assert_eq!(args.repo(), &None);
                assert_eq!(args.path(), &Some("rust".to_string()));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parse_self_check_defaults_channel_to_stable() {
        let cmd = GxCmd::try_parse_from(["gx", "self", "check"]).expect("self check should parse");

        match cmd {
            GxCmd::SelfUpdate(SelfCmd::Check(args)) => {
                assert_eq!(args.channel, "stable");
                assert!(!args.json);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parse_self_update_defaults_channel_to_stable() {
        let cmd =
            GxCmd::try_parse_from(["gx", "self", "update"]).expect("self update should parse");

        match cmd {
            GxCmd::SelfUpdate(SelfCmd::Update(args)) => {
                assert_eq!(args.channel, "stable");
                assert_eq!(args.to_version, None);
                assert!(!args.yes);
                assert!(!args.dry_run);
                assert!(!args.force);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn reject_init_project_branch_and_tag_together() {
        let result = GxCmd::try_parse_from([
            "gx", "init", "project", "--branch", "main", "--tag", "v1.0.0",
        ]);
        assert!(
            result.is_err(),
            "branch and tag should be mutually exclusive"
        );
    }

    #[test]
    fn parse_init_project_with_branch_and_path() {
        // --branch with --path uses default repo
        let cmd = GxCmd::try_parse_from([
            "gx", "init", "project", "--path", "rust", "--branch", "main",
        ])
        .expect("init project with path and branch should parse");
        match cmd {
            GxCmd::Init(InitCmd::Project(args)) => {
                assert_eq!(args.repo(), &None);
                assert_eq!(args.path(), &Some("rust".to_string()));
                assert_eq!(args.branch(), &Some("main".to_string()));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parse_init_project_with_tag_and_repo() {
        // --tag with --repo
        let cmd = GxCmd::try_parse_from([
            "gx",
            "init",
            "project",
            "--repo",
            "https://github.com/user/repo.git",
            "--tag",
            "v1.0.0",
        ])
        .expect("init project with repo and tag should parse");
        match cmd {
            GxCmd::Init(InitCmd::Project(args)) => {
                assert_eq!(
                    args.repo(),
                    &Some("https://github.com/user/repo.git".to_string())
                );
                assert_eq!(args.tag(), &Some("v1.0.0".to_string()));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
