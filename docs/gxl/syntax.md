# GXL 语法（对齐当前代码）

本文档按当前解析器实现整理（`src/parser/*`）。

## 1. 顶层结构

GXL 文件由多个 `mod` 组成：

```gxl
mod main {
  env default {
    ROOT = "./";
  }

  flow conf {
    gx.echo(value: "hello");
  }
}
```

也支持 `extern mod`（外部模块引用）：

```gxl
extern mod os,ssh { path = "./_gal/mods"; }
extern mod net { git = "https://example.com/repo.git", branch = "main"; }
```

## 2. 语法骨架

```ebnf
GxlFile      = { ExternMod | Module } ;

ExternMod    = "extern" "mod" ModNameList ModAddr ;
ModNameList  = Name { "," Name } ;
ModAddr      = "{" ( "path" "=" String
                 | "git" "=" String [ "," ("branch"|"channel") "=" String ] [ "," "tag" "=" String ] ) "}" ;

Module       = [Annotation] "mod" Name [":" MixList] "{" { Prop | Env | Flow | Fun | Activity } "}" [";"] ;
Env          = [Annotation] "env" Name [":" MixList] (";" | "{" { Prop | EnvStmt } "}" [";"]) ;
Flow         = [Annotation] "flow" FlowHead (";" | Block [";"]) ;
Fun          = "fn" Name "(" [FunParams] ")" (";" | Block [";"]) ;
Activity     = "activity" Name "{" { FormalParam [","|";" ] } "}" ;

FlowHead     = Name [":" FlowList [":" FlowList]]
             | [FlowPipe "|"] "@" Name ["|" FlowPipe]
             | Name ["|" FlowPipe] ;
FlowPipe     = FlowRef { "|" FlowRef } ;
FlowList     = FlowRef { "," FlowRef } ;
FlowRef      = Name | Name "." Name | VarRef ;
MixList      = MixItem { "," MixItem } ;
MixItem      = Name | VarRef ;

Block        = "{" { Prop | IfStmt | ForStmt | Builtin | Call | CmdBlock } "}" ;
IfStmt       = "if" Expr Block { "else" "if" Expr Block } [ "else" Block ] ;
ForStmt      = "for" VarRef "in" VarRef Block ;
CmdBlock     = "```cmd" <raw text> "```" ;

Prop         = Name "=" GxlObject (","|";") ;
GxlObject    = VarRef | Scalar | List | Object ;
Object       = "{" [ ObjItem {"," ObjItem} ] "}" ;
ObjItem      = Name ":" ScalarOrNested ;
List         = "[" [ ScalarOrNested {"," ScalarOrNested} ] "]" ;

Builtin      = gx.xxx call syntax ;
Call         = Path "(" [ActualParams] ")" [";"] ;
Annotation   = "#[" AnnFun {"," AnnFun} "]" ;
```

## 3. Flow 头部支持形式

### 3.1 旧式（冒号）

```gxl
flow test : pre1,pre2 : post1,post2 {
  gx.echo(value: "run");
}
```

### 3.2 管道 + `@` 指定主 flow

```gxl
flow pre1 | pre2 | @test | post1 | post2 {
  gx.echo(value: "run");
}
```

### 3.3 简写管道

```gxl
flow test | post1 | post2 {
  gx.echo(value: "run");
}
```

## 4. 参数与调用约定

所有内置能力统一使用调用形式：

```gxl
gx.cmd(cmd: "echo hello");
gx.echo("hello");              // 等价，匿名参数映射到 default
```

说明：
- 调用参数分隔符是 `,`。
- 命名参数使用 `:`，如 `name: "v"`。
- 很多能力支持 `default`（匿名首参数）写法。
- `gx.cmd`、`gx.shell`、`gx.read_cmd` 支持 `stream: "true"`，用于长时间命令的实时输出。

## 5. 条件表达式

支持：
- 比较：`== != > >= < <= =*`（`=*` 为通配）
- 逻辑：`&& || !`
- 函数：`defined(${VAR})`

示例：

```gxl
if defined(${CUR.ENABLE}) && ${CUR.ENABLE} == true {
  gx.echo(value: "enabled");
} else {
  gx.echo(value: "disabled");
}
```

## 6. 目前块内可直接识别的内置能力

- `gx.cmd`
- `gx.shell`
- `gx.run`
- `gx.echo`
- `gx.assert`
- `gx.ver`
- `gx.read_file`
- `gx.read_cmd`
- `gx.read_stdin`
- `gx.tpl`
- `gx.tar`
- `gx.untar`
- `gx.download`
- `gx.upload`
- `gx.patch_file`
- `gx.sn`

补充：
- `gx.vars` 只在 `env` 块中使用。
- `defined(...)` 是表达式函数，不是 `gx.defined` 命令。

## 7. 注解

注解用 `#[...]` 写在 `mod` / `env` / `flow` 之前，多个注解可以用 `,` 写在同一个括号内：

```gxl
#[usage(desp="default auto")]
env default : local;

#[usage(desp="developer local env",color="red"),auto_load(entry)]
flow __into {
  prj_bins = "${ENV_ROOT}/bin";
}
```

当前解析器识别的注解（见 `src/parser/stc_ann.rs`、`src/parser/stc_mod.rs`、`src/parser/stc_flow/body.rs`）：

- `#[usage(desp="...", color="...")]`：给弹窗/菜单展示的描述与颜色
- `#[auto_load(entry)]` / `#[auto_load(exit)]`：标记自动加载的入口/出口 flow
- `#[task(name="...")]`：给 flow 打任务名

参数形式统一是 `名称="值"`（字符串）；`#[fun]`（无参数）也合法。
