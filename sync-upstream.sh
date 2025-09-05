#!/bin/bash

# Loco 上游同步脚本
# 用于将上游更新合并到您的功能分支

set -e  # 遇到错误时退出

echo "🔄 开始同步上游更新..."

# 1. 保存当前分支
CURRENT_BRANCH=$(git branch --show-current)
echo "📍 当前分支: $CURRENT_BRANCH"

# 2. 获取上游最新更新
echo "📥 获取上游更新..."
git fetch upstream

# 3. 切换到 master 分支更新基础代码
echo "🔄 更新 master 分支..."
git checkout master
git pull upstream master

# 4. 切换回功能分支
echo "🔀 切换回功能分支..."
git checkout "$CURRENT_BRANCH"

# 5. 将上游更新合并到功能分支
echo "🔀 合并上游更新到功能分支..."
git merge upstream/master -m "sync: 合并上游更新

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"

# 6. 推送到您的 Fork
echo "📤 推送到您的 Fork..."
git push origin "$CURRENT_BRANCH"

echo "✅ 同步完成！"
echo ""
echo "📋 摘要:"
echo "  - 上游更新已合并到 $CURRENT_BRANCH"
echo "  - 已推送到您的 Fork: https://github.com/GoooIce/loco"
echo ""
echo "🔗 如果有冲突，请手动解决后重新提交"