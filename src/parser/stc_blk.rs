use super::inner::call::gal_call;
use super::inner::cmd::gal_cmd_block;
use super::inner::gxl::gal_run;
use super::inner::shell::gal_shell;
use super::prelude::*;
use orion_parse::define::take_var_ref_name;
use winnow::combinator::repeat;

use crate::ability::prelude::GxlVar;
use crate::components::gxl_block::{BlockAction, BlockNode};
use crate::components::gxl_loop::GxlLoop;
use crate::parser::cond::gal_cond;
use crate::parser::inner::archive::{gal_tar, gal_untar};

use super::atom::spaced;
use super::domain::{gal_block_beg, gal_block_end, gal_keyword};
use super::inner::{
    gal_assert, gal_cmd, gal_download, gal_echo, gal_patch_file, gal_prop, gal_read_cmd,
    gal_read_file, gal_read_stdin, gal_sn, gal_tpl, gal_upload, gal_version,
};

pub fn gal_block(input: &mut &str) -> Result<BlockNode> {
    let mut block = BlockNode::new();
    gal_block_beg
        .context(wn_desc("<block-beg>"))
        .parse_next(input)?;
    let props: Vec<GxlVar> = repeat(0.., gal_prop).parse_next(input)?;
    let sentens: Vec<BlockAction> = repeat(0.., gal_sentens_item)
        .context(wn_desc("<sentens>"))
        .parse_next(input)?;
    gal_block_end
        .context(wn_desc("<block-end>"))
        .parse_next(input)?;
    for i in props {
        block.append(i);
    }
    block.append(sentens);
    Ok(block)
}

pub fn gal_block_code(input: &mut &str) -> Result<BlockNode> {
    let mut block = BlockNode::new();
    loop {
        skip_spaces_block.parse_next(input)?;
        if starts_with((multispace0, "}"), input) {
            return Ok(block);
        }

        let senten = gal_sentens_item
            .context(wn_desc("<senten>"))
            .parse_next(input)?;
        block.append(senten);
    }
    //Ok(block)
}

pub fn gal_sentens_item(input: &mut &str) -> Result<BlockAction> {
    multispace0(input)?;
    if starts_with("if", input) {
        return gal_cond.map(BlockAction::Cond).parse_next(input);
    }
    if starts_with("for", input) {
        return gal_loop.map(BlockAction::Loop).parse_next(input);
    }
    if starts_with("gx.cmd", input) {
        return gal_cmd.map(BlockAction::Command).parse_next(input);
    }
    if starts_with("gx.shell", input) {
        return gal_shell.map(BlockAction::Shell).parse_next(input);
    }

    if starts_with("gx.run", input) {
        return gal_run.map(BlockAction::GxlRun).parse_next(input);
    }
    if starts_with("```cmd", input) {
        return gal_cmd_block.map(BlockAction::Command).parse_next(input);
    }

    if starts_with("gx.echo", input) {
        return gal_echo.map(BlockAction::Echo).parse_next(input);
    }
    if starts_with("gx.assert", input) {
        return gal_assert.map(BlockAction::Assert).parse_next(input);
    }
    if starts_with("gx.ver", input) {
        return gal_version.map(BlockAction::Version).parse_next(input);
    }
    if starts_with("gx.sn", input) {
        return gal_sn.map(BlockAction::Sn).parse_next(input);
    }
    if starts_with("gx.read_file", input) {
        return gal_read_file.map(BlockAction::Read).parse_next(input);
    }
    if starts_with("gx.read_cmd", input) {
        return gal_read_cmd.map(BlockAction::Read).parse_next(input);
    }
    if starts_with("gx.read_stdin", input) {
        return gal_read_stdin.map(BlockAction::Read).parse_next(input);
    }
    if starts_with("gx.tpl", input) {
        return gal_tpl.map(BlockAction::Tpl).parse_next(input);
    }

    if starts_with("gx.tar", input) {
        return gal_tar.map(BlockAction::Tar).parse_next(input);
    }
    if starts_with("gx.untar", input) {
        return gal_untar.map(BlockAction::UnTar).parse_next(input);
    }
    if starts_with("gx.download", input) {
        return gal_download.map(BlockAction::DownLoad).parse_next(input);
    }
    if starts_with("gx.upload", input) {
        return gal_upload.map(BlockAction::UpLoad).parse_next(input);
    }
    if starts_with("gx.patch_file", input) || starts_with("rg.patch_file", input) {
        return gal_patch_file.map(BlockAction::PatchFile).parse_next(input);
    }
    /*
    if starts_with("gx.vault", input) {
        return gal_vault.map(BlockAction::Vault).parse_next(input);
    }
    */

    gal_call
        .context(wn_desc("<flow-call>"))
        .map(|x| BlockAction::Call(Box::new(x)))
        .parse_next(input)
}

pub fn gal_loop(input: &mut &str) -> Result<GxlLoop> {
    //if val == 1
    skip_spaces_block(input)?;
    gal_keyword("for", input)?;

    let (cur_name, _, val_name) = (
        spaced(take_var_ref_name).context(wn_desc("<cur-var>")),
        spaced("in").context(wn_desc("in")),
        spaced(take_var_ref_name).context(wn_desc("<var-set>")),
    )
        .parse_next(input)?;
    let block = gal_block.parse_next(input)?;
    skip_spaces_block(input)?;
    multispace0(input)?;
    Ok(GxlLoop::new(cur_name, val_name, block))
}

#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;

    use crate::components::gxl_block::BlockAction;
    use crate::parser::{
        inner::run_gxl,
        stc_blk::{gal_block, gal_loop},
    };

    #[test]
    fn test_block_1() {
        let mut data = r#"
        {
             gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
             gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );

             gx.assert ( value : "hello" , expect : "hello" , err : "errinfo") ;
        }"#;
        let blk = run_gxl(gal_block, &mut data).assert();
        assert_eq!(blk.items().len(), 3);
        assert_eq!(data, "");
    }
    #[test]
    fn test_block_props_1() {
        let mut data = r#" {
            one= "one";
            sys_a = { mod1 : "A", mod2 : "B" , mod3 : 3};
            sys_b =  [ "C", "D" , 1,2 ];
            sys_c = ${SYS_B[1]} ;
            sys_d = ${SYS_A.MOD1} ;
            }
            "#;
        let blk = run_gxl(gal_block, &mut data).assert();
        assert_eq!(blk.props().len(), 5);
        assert_eq!(data, "");
    }

    #[test]
    fn test_for() {
        let mut data = r#"
            for  ${CUR} in ${DATA} {
                gx.echo ( value  : "${cur}/test/main.py"  );
             }
        "#;
        let _ = run_gxl(gal_loop, &mut data).assert();
        assert_eq!(data, "");
    }
    #[test]
    fn test_if_for() {
        let mut data = r#"
            {
            if ${val} == 1 {
                for  ${CUR} in ${DATA} {
                    gx.echo ( value  : "${cur}/test/main.py"  );
                }
             }
             }
        "#;
        let _ = run_gxl(gal_block, &mut data).assert();
        assert_eq!(data, "");
    }

    #[test]
    fn test_block_rg_patch_file_alias() {
        let mut data = r#"
        {
             rg.patch_file (
             file : "./Cargo.toml",
             action : "set",
             marker : "version",
             value : "v2.0",
             ) ;
        }"#;
        let blk = run_gxl(gal_block, &mut data).assert();
        assert_eq!(blk.items().len(), 1);
        assert!(matches!(blk.items()[0], BlockAction::PatchFile(_)));
        assert_eq!(data, "");
    }
}
