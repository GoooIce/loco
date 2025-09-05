# Loco 开发工作流程

## 🎯 概述

这是一个基于 Fork + 上游同步的开发工作流程，让您可以在不影响原始仓库的情况下进行开发。

## 📁 远程仓库配置

```
origin    -> https://github.com/GoooIce/loco.git (您的 Fork)
upstream  -> https://github.com/loco-rs/loco.git (原始仓库)
```

## 🔄 日常开发流程

### 1. 获取上游更新

```bash
# 方法 1: 使用同步脚本（推荐）
./sync-upstream.sh

# 方法 2: 手动同步
git fetch upstream
git merge upstream/master
git push origin feature/mcp-i18n-support
```

### 2. 创建新功能分支

```bash
git checkout master
git pull upstream master
git checkout -b feature/new-feature-name
```

### 3. 提交修改

```bash
git add .
git commit -m "feat: 添加新功能

详细描述...

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"
git push origin feature/new-feature-name
```

### 4. 创建 Pull Request

```bash
gh pr create --title "新功能标题" --body "功能描述..."
```

## 🛠️ 有用的 Git 命令

### 查看仓库状态
```bash
git status
git remote -v
git branch -a
```

### 处理冲突
```bash
# 如果合并时出现冲突
git merge --abort  # 中止合并
# 或手动解决冲突后
git add .
git commit
```

### 清理
```bash
git gc  # 垃圾回收
git prune  # 清理无效引用
```

## 📋 当前功能分支

- **分支名**: `feature/mcp-i18n-support`
- **功能**: MCP 服务器支持和国际化
- **状态**: 已推送到您的 Fork
- **PR 链接**: https://github.com/GoooIce/loco/pull/new/feature/mcp-i18n-support

## 🎉 完成的好处

✅ **安全隔离**: 您的修改不会影响原始仓库  
✅ **轻松同步**: 随时获取上游最新更新  
✅ **便于贡献**: 可以轻松创建 Pull Request  
✅ **版本控制**: 完整的历史记录和分支管理  
✅ **自动化**: 提供同步脚本简化操作