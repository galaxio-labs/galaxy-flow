#[cfg(test)]
mod tests {
    use orion_conf::TomlIO;
    use orion_error::dev::testing::TestAssertWithMsg;
    use orion_infra::path::make_clean_path;
    use std::path::PathBuf;

    use crate::conf::gxlconf::GxlConf;
    use crate::task_report::task_rc_config::ReportCenterConf;

    #[test]
    fn test_gxl_conf_serialization() {
        let conf = GxlConf::new(ReportCenterConf::new("example.com", 8080), true);
        let path = PathBuf::from("./temp/conf");
        make_clean_path(&path).assert("clean path");
        let file = path.join("conf.toml");
        conf.save_toml(&file).assert("save toml");
        let loaded = GxlConf::load_toml(&file).assert("load toml");
        assert_eq!(
            loaded.task_report().report_svr(),
            conf.task_report().report_svr()
        );
    }
}
