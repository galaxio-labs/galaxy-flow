extern crate galaxy_flow;
extern crate shells;

use std::path::PathBuf;

use galaxy_flow::GxLoader;
use galaxy_flow::infra::once_init_log;
use galaxy_flow::util::path::WorkDir;
use orion_accessor::addr::GitRepository;
use orion_error::dev::testing::TestAssertWithMsg;
use orion_infra::path::ensure_path;

// use shells;
#[ignore]
#[tokio::test]
async fn init_test() {
    once_init_log();
    let loader = GxLoader::new();

    let path = PathBuf::from("./tests/temp/init");
    ensure_path(&path).assert("path");
    let _work_path = WorkDir::change(&path);
    let addr =
        GitRepository::from("https://github.com/galaxio-labs/gal-init.git").with_branch("main");
    loader.init_from_git(addr).await.assert("init");
}
