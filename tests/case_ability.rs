extern crate galaxy_flow;

use galaxy_flow::GxLoader;
use galaxy_flow::cmd::GxlCmd;
use galaxy_flow::err::RunResult;
use galaxy_flow::execution::VarSpace;
use galaxy_flow::infra::once_init_log;
use orion_error::dev::testing::TestAssert;

#[tokio::test]
async fn conf_base_test() -> RunResult<()> {
    once_init_log();
    let loader = GxLoader::new();
    let vars = VarSpace::sys_init().assert();

    let spc = loader
        .parse_file("./tests/material/ability.gxl", false, &vars)
        .await?
        .assemble()
        .assert();
    spc.exec(
        GxlCmd::default().with_flows("test".into()),
        VarSpace::default(),
        None,
    )
    .await?;
    Ok(())
}

#[tokio::test]
async fn conf_web_test() {
    once_init_log();
    let vars = VarSpace::sys_init().assert();
    let loader = GxLoader::new();

    let spc = loader
        .parse_file("./tests/material/run_web.gxl", false, &vars)
        .await
        .unwrap()
        .assemble()
        .assert();
    spc.exec(
        GxlCmd::default()
            .with_env("dev".into())
            .with_flows("api".into()),
        VarSpace::default(),
        None,
    )
    .await
    .unwrap();
}
