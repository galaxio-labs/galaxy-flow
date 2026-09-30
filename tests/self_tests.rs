extern crate galaxy_flow;

use galaxy_flow::GxLoader;
use galaxy_flow::cmd::GxlCmd;
use galaxy_flow::err::RunResult;
use galaxy_flow::execution::VarSpace;
use galaxy_flow::infra::once_init_log;
use orion_error::dev::testing::TestAssert;

#[tokio::test]
async fn prj_conf() -> RunResult<()> {
    once_init_log();
    let loader = GxLoader::new();

    let vars = VarSpace::sys_init().assert();
    let spc = loader
        .parse_file("./_gal/work.gxl", false, &vars)
        .await?
        .assemble()
        .assert();
    spc.exec(
        GxlCmd::default().with_flows("conf".into()),
        VarSpace::sys_init().assert(),
        None,
    )
    .await?;
    Ok(())
}
