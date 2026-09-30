# Read 示例

## 示例代码

```rust
extern mod os { path= "../../_gal/mods"; }
mod envs {
    env _dev_local {
        gx.read_file ( file : "./var.ini" );
    }
    env default : _dev_local ;
}
mod main   {
  flow conf  {
    gx.echo (  "${RUST}" );
    gx.echo (  "${JAVA}" );
    gx.assert ( value : "${JAVA}" , expect : "90"  );

    gx.read_cmd (
        //fail!
        //cmd  : r#"git branch --show-current |  sed -E "s/(feature|develop|ver-dev|release|master|issue)(\/\.*)?/_branch_\1/g" "# ,
        //suc!
        cmd  : "git branch --show-current | sed -E 's/release/rls/g'" ,
        name : "GIT_BRANCH" );

    gx.read_cmd (
        cmd : "PYTHONUNBUFFERED=1 ansible-playbook site.yml -vv",
        name : "LAST_STDOUT",
        stream : "true" );

    gx.echo ( "what:${GIT_BRANCH}" );

    gx.read_file ( file : "./var2.ini" , name : "DATA");

    for ${CUR} in ${DATA} {
        gx.echo ( value : "${CUR}" );
    }
  }



}
```

```mermaid
graph TD
    A[Start] --> B[Load envs module]
    B --> C[Execute _dev_local env]
    C --> D[Execute gx.read_file command]
    D --> E[Load main module]
    E --> F[Execute conf flow]
    F --> G[Execute gx.echo commands]
    G --> H[Execute gx.read_cmd command]
    H --> I[Execute gx.echo command]
    I --> J[Execute gx.read_file command]
    J --> K[Loop through DATA]
    K --> L[Execute gx.echo command for each item]
    L --> M[End]
```

## 说明

这个示例展示了如何使用 `gx.read` 命令从不同来源读取数据。在 `_dev_local` 环境中，使用 `gx.read_file` 从 `var.ini` 文件读取数据。在 `conf` 流程中，使用 `gx.read_cmd` 执行 Git 命令并捕获输出，也展示了 `stream: "true"` 的实时输出模式。即使开启流式输出，`gx.read_cmd` 最终仍只会把 stdout 写入变量。还展示了如何遍历读取的数据。
