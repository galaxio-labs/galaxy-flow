# GXL 帮助（对齐当前实现）

## 1. 最小可运行示例

```gxl
mod envs {
  env default {
    ROOT = "./";
  }
}

mod main {
  flow conf {
    gx.echo(value: "hello galaxy flow");
  }
}
```

## 2. 常用结构

### 2.1 模块与继承

```gxl
mod base {
  flow setup {
    gx.echo(value: "setup");
  }
}

mod main : base {
  flow conf {
    gx.echo(value: "conf");
  }
}
```

### 2.2 环境与 `gx.vars`

```gxl
mod envs {
  env base {
    gx.vars {
      APP = "galaxy";
      STAGE = "dev";
    };
  }

  env default : base;
}
```

### 2.3 flow 编排

```gxl
mod main {
  flow prepare {
    gx.echo(value: "prepare");
  }

  flow deploy {
    gx.echo(value: "deploy");
  }

  flow @release | prepare | deploy {
    gx.echo(value: "release");
  }
}
```

### 2.4 函数与活动

```gxl
mod sys {
  fn echo_tag(tag = "INFO", *msg) {
    gx.echo(value: "[${tag}] ${msg}");
  }

  activity copy {
    src = "";
    dst = "";
    executer = "copy_act.sh";
  }
}

mod main {
  flow conf {
    sys.echo_tag(msg: "start");
    sys.copy(src: "a.txt", dst: "b.txt");
  }
}
```

## 3. 控制流

### 3.1 `if/else if/else`

```gxl
flow conf {
  if defined(${DEPLOY}) && ${DEPLOY} == "true" {
    gx.echo(value: "deploy");
  } else if ${STAGE} == "test" {
    gx.echo(value: "test");
  } else {
    gx.echo(value: "skip");
  }
}
```

### 3.2 `for`

```gxl
flow conf {
  for ${CUR} in ${DATA} {
    gx.echo(value: "item=${CUR}");
  }
}
```

## 4. 内置能力写法

当前实现统一是函数调用形式：

```gxl
gx.cmd(cmd: "echo hi");
gx.shell(shell: "./demo.sh", out_var: "OUT");
gx.read_file(file: "./var.yml", name: "DATA");
gx.tpl(tpl: "./tpls", dst: "./out", file: "./vars.json");
```

补充：
- 对长时间命令可使用 `stream: "true"`，例如 `gx.cmd(cmd: "ansible-playbook site.yml -vv", stream: "true")`

不要使用旧文档中的 `gx.xxx { ... }` 形式。

## 5. `gx.patch_file` 快速示例

```gxl
gx.patch_file(
  file: "./Cargo.toml",
  action: "set",
  marker: "version",
  value: "0.12.3"
);
```

对应 marker 行：

```toml
version = "0.12.2"   # @gxl:set(version)
```

更多参数见：`docs/gxl/inner/patch_file.md`
