extern crate galaxy_flow;

use galaxy_flow::cmd::GxlCmd;
use galaxy_flow::execution::VarSpace;
use galaxy_flow::infra::once_init_log;
use galaxy_flow::{GxLoader, err::*};
use orion_error::dev::testing::TestAssert;

#[tokio::test]
async fn conf_simple_test() -> RunResult<()> {
    once_init_log();
    let vars = VarSpace::sys_init().assert();
    let loader = GxLoader::new();

    let spc = loader
        .parse_file("./tests/material/case_simple.gxl", false, &vars)
        .await?
        .assemble()
        .assert();
    spc.exec(
        GxlCmd::default()
            .with_env("dev".into())
            .with_flows("api".into()),
        VarSpace::default(),
        None,
    )
    .await?;
    Ok(())
}

#[ignore]
#[tokio::test]
async fn conf_cond_test() -> RunResult<()> {
    once_init_log();
    let loader = GxLoader::new();
    let vars = VarSpace::sys_init().assert();

    let spc = loader
        .parse_file("./tests/material/case_cond.gxl", false, &vars)
        .await?
        .assemble()
        .assert();
    spc.exec(
        GxlCmd::default()
            .with_env("dev".into())
            .with_flows("api,start".into()),
        VarSpace::default(),
        None,
    )
    .await?;
    Ok(())
}
