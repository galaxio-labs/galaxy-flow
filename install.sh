#!/usr/bin/env bash
#
# ⚠️ 已弃用（DEPRECATED）—— 请改用官方安装脚本
#
# 本脚本曾从本仓库的 updates/{stable,alpha,beta}/manifest.json 安装 gx。该链路已废弃：
#   - 本仓库的清单长期未维护（stable 停在 0.12.4），且 sha256 是占位符，安装时会跳过校验；
#   - `gx self` 自更新读取的是 get 仓库（updates/gx/{channel}/），与这里不是同一份数据，
#     两套 manifest 长期不一致。
# 因此本仓库的 updates/ 清单与安装逻辑已一并移除。
#
# 请使用官方安装脚本（README「安装说明」中的推荐方式）：
#
#   # stable
#   curl -sSf https://get.warpparse.ai/inst-x.sh | bash -s -- gx
#
#   # alpha channel
#   curl -sSf https://get.warpparse.ai/inst-x.sh | bash -s -- gx alpha
#
#   # 自定义安装目录
#   curl -sSf https://get.warpparse.ai/inst-x.sh | INSTALL_DIR=/usr/local/bin bash -s -- gx
#
# 安装后的版本管理用：gx self status / gx self check / gx self update

set -euo pipefail

cat >&2 <<'MSG'
[install.sh] 已弃用，请改用官方安装脚本：

  # stable
  curl -sSf https://get.warpparse.ai/inst-x.sh | bash -s -- gx

  # alpha
  curl -sSf https://get.warpparse.ai/inst-x.sh | bash -s -- gx alpha

原因：本脚本依赖的本仓库 updates/ 清单已移除；它也与 `gx self` 使用的
galaxio-labs/get 清单不是同一份数据，不再维护。
MSG

exit 1
