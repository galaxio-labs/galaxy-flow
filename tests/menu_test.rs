extern crate galaxy_flow;
use galaxy_flow::GxLoader;
use galaxy_flow::execution::VarSpace;
use galaxy_flow::infra::once_init_log;
use galaxy_flow::menu::*;
use galaxy_flow::traits::ExecLoadTrait;
use orion_error::dev::testing::TestAssert;

#[tokio::test]
async fn menu_normal() {
    once_init_log();
    let loader = GxLoader::new();

    let vars = VarSpace::sys_init().assert();
    let spc = loader
        .parse_file("./tests/material/menu.gxl", false, &vars)
        .await
        .unwrap()
        .assemble()
        .assert();
    let menu = spc.menu().unwrap();
    let mut expect = GxMenu::default();
    expect.envs.push(MenuItem::new("e1", None, None));
    expect.envs.push(MenuItem::new("b_e1", None, None));
    expect.envs.push(MenuItem::new("g_e1", None, None));
    expect.flows.push(MenuItem::new("f1", None, None));
    expect.flows.push(MenuItem::new("f2", None, None));
    expect.flows.push(MenuItem::new("b_f1", None, None));
    expect.flows.push(MenuItem::new("b_f2", None, None));
    expect.flows.push(MenuItem::new("g_f1", None, None));
    expect.flows.push(MenuItem::new("g_f2", None, None));
    assert_eq!(menu, expect);
}

#[tokio::test]
async fn menu_simple() {
    once_init_log();
    let loader = GxLoader::new();

    let vars = VarSpace::sys_init().assert();
    let spc = loader
        .parse_file("./tests/material/simple_menu.gxl", false, &vars)
        .await
        .unwrap()
        .assemble()
        .assert();
    let menu = spc.menu().unwrap();
    let mut expect = GxMenu::default();
    expect.envs.push(MenuItem::new("e1", None, None));
    expect.flows.push(MenuItem::new("f1", None, None));
    expect.flows.push(MenuItem::new("f2", None, None));
    assert_eq!(menu, expect);
}
