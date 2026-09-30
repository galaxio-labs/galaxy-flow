//extern crate galaxy_flow;
//#[cfg(feature = "res_depend_test")]
mod tests {
    use galaxy_flow::cmd::GxlCmd;
    use galaxy_flow::err::report_gxl_error;
    use galaxy_flow::execution::VarSpace;
    use galaxy_flow::util::path::WorkDirWithLock;
    use galaxy_flow::{GxLoader, err::RunResult, infra::once_init_log};
    use orion_error::dev::testing::TestAssert;

    #[tokio::test(flavor = "current_thread")]
    async fn example_read() -> RunResult<()> {
        //once_init_log();
        let _dir = WorkDirWithLock::change("./examples/read");
        let vars = VarSpace::sys_init().assert();
        let loader = GxLoader::new();
        let spc = loader
            .parse_file("./_gal/work.gxl", false, &vars)
            .await?
            .assemble()
            .assert();
        spc.exec(
            GxlCmd::default().with_flows("conf".into()),
            VarSpace::default(),
            None,
        )
        .await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn example_shell() -> RunResult<()> {
        //once_init_log();
        let _dir = WorkDirWithLock::change("./examples/shell");
        let vars = VarSpace::sys_init().assert();
        let loader = GxLoader::new();
        let spc = loader
            .parse_file("./_gal/work.gxl", false, &vars)
            .await?
            .assemble()
            .assert();
        spc.exec(
            GxlCmd::default().with_flows("conf".into()),
            VarSpace::default(),
            None,
        )
        .await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn example_function() -> RunResult<()> {
        //once_init_log();
        let _dir = WorkDirWithLock::change("./examples/fun");
        let vars = VarSpace::sys_init().assert();
        let loader = GxLoader::new();
        let spc = loader
            .parse_file("./_gal/work.gxl", false, &vars)
            .await?
            .assemble()
            .assert();
        spc.exec(
            GxlCmd::default().with_flows("conf".into()),
            VarSpace::default(),
            None,
        )
        .await?;
        Ok(())
    }
    #[tokio::test(flavor = "current_thread")]
    async fn example_assert() -> RunResult<()> {
        //once_init_log();
        let _dir = WorkDirWithLock::change("./examples/assert");
        let vars = VarSpace::sys_init().assert();
        let loader = GxLoader::new();
        let spc = loader
            .parse_file("./_gal/work.gxl", false, &vars)
            .await?
            .assemble()
            .assert();
        spc.exec(
            GxlCmd::default().with_flows("assert_main".into()),
            VarSpace::default(),
            None,
        )
        .await?;
        Ok(())
    }
    #[tokio::test(flavor = "current_thread")]
    async fn example_template() -> RunResult<()> {
        //jonce_init_log();
        let _dir = WorkDirWithLock::change("./examples/template");
        let loader = GxLoader::new();
        let vars = VarSpace::sys_init().assert();
        let spc = loader
            .parse_file("./_gal/work.gxl", false, &vars)
            .await?
            .assemble()
            .assert();
        spc.exec(
            GxlCmd::default().with_flows("conf".into()),
            VarSpace::default(),
            None,
        )
        .await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn example_translate() -> RunResult<()> {
        once_init_log();
        let _dir = WorkDirWithLock::change("./examples/transaction");
        let loader = GxLoader::new();
        let vars = VarSpace::sys_init().assert();
        let spc = loader
            .parse_file("./_gal/work.gxl", false, &vars)
            .await?
            .assemble()
            .assert();
        let result = spc
            .exec(
                GxlCmd::default()
                    .with_env("default".into())
                    .with_flows("trans1".into()),
                VarSpace::default(),
                None,
            )
            .await;
        match result {
            Ok(_) => {
                panic!("need fail!");
            }
            Err(e) => {
                report_gxl_error(e);
            }
        }
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn example_dryrun() -> RunResult<()> {
        once_init_log();
        let _dir = WorkDirWithLock::change("./examples/dryrun");
        let loader = GxLoader::new();
        let vars = VarSpace::sys_init().assert();
        let spc = loader
            .parse_file("./_gal/work.gxl", false, &vars)
            .await?
            .assemble()
            .assert();
        spc.exec(
            GxlCmd::default()
                .with_dryrun(true)
                .with_env("default".into())
                .with_flows("start".into()),
            VarSpace::default(),
            None,
        )
        .await?;

        let fail = spc
            .exec(
                GxlCmd::default().with_flows("start".into()),
                VarSpace::default(),
                None,
            )
            .await;
        assert!(fail.is_err());
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    #[ignore]
    async fn example_ai() -> RunResult<()> {
        once_init_log();
        let vars = VarSpace::sys_init().assert();

        let _dir = WorkDirWithLock::change("./examples/ai");
        let loader = GxLoader::new();
        let spc = loader
            .parse_file("./_gal/work.gxl", false, &vars)
            .await?
            .assemble()
            .assert();
        spc.exec(
            GxlCmd::default().with_flows("dev_ai".into()),
            VarSpace::default(),
            None,
        )
        .await?;

        /*
        let fail = spc
            .exec(
                GxlCmd::default().with_flows("start".into()),
                VarSpace::default(),
                None,
            )
            .await;
        assert!(fail.is_err());
        */
        Ok(())
    }
}
