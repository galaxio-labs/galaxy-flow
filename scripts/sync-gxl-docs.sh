#!/usr/bin/env bash
#
# 同步 / 校验 GXL 文档镜像（单一真源：galaxy-flow 本仓库的 docs/gxl/）
#
# 背景
#   operator-docs 仓库中的 gxl/ 大部分是本仓库 docs/gxl/ 的镜像，但
#   docs/gxl/example/*.md 在 operator-docs 侧是刻意改写的运维向内容
#   （补充“对应目录 / 运行方式 / 这个示例验证了什么”等），不是镜像。
#   因此这里用显式边界，而不是整目录覆盖——整目录 rsync 会毁掉那 9 个文件。
#
# 用法
#   scripts/sync-gxl-docs.sh sync  [--src DIR] [--dest DIR]
#   scripts/sync-gxl-docs.sh check [--src DIR] [--dest DIR]
#
#   sync   把镜像文件从 src 复制到 dest（并删除 dest 中已成为孤儿的镜像文件）
#   check  只校验，发现差异或孤儿时输出清单并以退出码 1 结束（供 CI 使用）
#   list   打印镜像范围内的文件相对路径（每行一个），供其他脚本/CI 使用
#
# 默认路径
#   --src   <本仓库>/docs/gxl
#   --dest  <本仓库>/../operator-docs/gxl
#
# 边界
#   镜像：docs/gxl/** 中除下面 EXCLUDE_GLOBS 之外的**全部文件**
#   同步：无（dest 侧自有内容），见 EXCLUDE_GLOBS

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

SRC="$REPO_ROOT/docs/gxl"
DEST="$REPO_ROOT/../operator-docs/gxl"

# 这些是 operator-docs 侧自有的运维向文档，不属于镜像，永不覆盖。
# 使用相对 src 的 glob 匹配。
EXCLUDE_GLOBS=(
	"example/*.md"
)

case "${1:-}" in
-h | --help)
	awk 'NR>=3 { if ($0 !~ /^#/) exit; sub(/^# ?/, ""); print }' "${BASH_SOURCE[0]}"
	exit 0
	;;
esac

MODE="${1:-}"
if [ "$MODE" != "sync" ] && [ "$MODE" != "check" ] && [ "$MODE" != "list" ]; then
	echo "usage: $0 {sync|check|list} [--src DIR] [--dest DIR]" >&2
	exit 2
fi
shift || true

while [ $# -gt 0 ]; do
	case "$1" in
	--src)
		SRC="${2:?--src needs a value}"
		shift 2
		;;
	--dest)
		DEST="${2:?--dest needs a value}"
		shift 2
		;;
	*)
		echo "unknown argument: $1" >&2
		exit 2
		;;
	esac
done

if [ ! -d "$SRC" ]; then
	echo "source dir not found: $SRC" >&2
	exit 2
fi
if [ "$MODE" != "list" ] && [ ! -d "$DEST" ]; then
	echo "dest dir not found: $DEST" >&2
	echo "hint: pass --dest pointing at the operator-docs gxl/ directory" >&2
	exit 2
fi

is_excluded() {
	local rel="$1"
	local pattern
	for pattern in "${EXCLUDE_GLOBS[@]}"; do
		# shellcheck disable=SC2053
		if [[ "$rel" == $pattern ]]; then
			return 0
		fi
	done
	return 1
}

# 镜像文件清单（相对 SRC）
mirrored=()
while IFS= read -r rel; do
	if ! is_excluded "$rel"; then
		mirrored+=("$rel")
	fi
done < <(cd "$SRC" && find . -type f | sed 's|^\./||' | sort)

if [ "$MODE" = "list" ]; then
	printf '%s\n' "${mirrored[@]}"
	exit 0
fi

drift=()
orphans=()

for rel in "${mirrored[@]}"; do
	if [ ! -f "$DEST/$rel" ]; then
		drift+=("$rel (missing in dest)")
		continue
	fi
	if ! cmp -s "$SRC/$rel" "$DEST/$rel"; then
		drift+=("$rel (content differs)")
	fi
done

# dest 中属于镜像范围、但 src 已不存在的文件 = 孤儿
while IFS= read -r rel; do
	if is_excluded "$rel"; then
		continue
	fi
	if [ ! -f "$SRC/$rel" ]; then
		orphans+=("$rel")
	fi
done < <(cd "$DEST" && find . -type f | sed 's|^\./||' | sort)

if [ "$MODE" = "sync" ]; then
	for rel in "${mirrored[@]}"; do
		mkdir -p "$DEST/$(dirname "$rel")"
		cp "$SRC/$rel" "$DEST/$rel"
	done
	for rel in "${orphans[@]}"; do
		rm -f "$DEST/$rel"
	done
	echo "synced ${#mirrored[@]} file(s) to $DEST"
	if [ ${#orphans[@]} -gt 0 ]; then
		echo "removed ${#orphans[@]} orphaned file(s):"
		printf '  - %s\n' "${orphans[@]}"
	fi
	exit 0
fi

# check 模式
if [ ${#drift[@]} -eq 0 ] && [ ${#orphans[@]} -eq 0 ]; then
	echo "GXL docs mirror is in sync (${#mirrored[@]} file(s) checked)"
	exit 0
fi

echo "GXL docs mirror is OUT OF SYNC" >&2
if [ ${#drift[@]} -gt 0 ]; then
	echo "" >&2
	echo "源已变更、镜像未同步：" >&2
	printf '  - %s\n' "${drift[@]}" >&2
fi
if [ ${#orphans[@]} -gt 0 ]; then
	echo "" >&2
	echo "镜像中已成孤儿的文件（源已删除）：" >&2
	printf '  - %s\n' "${orphans[@]}" >&2
fi
echo "" >&2
echo "修复：在 galaxy-flow 仓库执行 scripts/sync-gxl-docs.sh sync --dest <operator-docs>/gxl" >&2
exit 1
