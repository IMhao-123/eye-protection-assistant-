# 参与贡献

感谢你愿意改进护眼助手。提交代码前，请先在 Issue 中说明问题或方案，避免重复工作。

## 本地开发

需要 Apple Silicon Mac、Node.js 20+、Rust stable 和 Xcode Command Line Tools。

```bash
npm ci
npm run tauri:dev
```

提交 Pull Request 前必须运行：

```bash
npm run verify
```

## 开发原则

- 核心计时规则由 Rust 权威状态机维护，React 只渲染快照并发送动作。
- 功能修改遵循 Red → Green → Refactor，先提交能复现问题的测试。
- 不引入远程字体、远程图片、遥测或核心功能的网络依赖。
- GUI 不使用 Emoji；新增图标应沿用现有 Lucide 或本地矢量体系。
- 不弱化 `Cmd+Q`、菜单栏退出或其他明确的用户退出路径。

## Pull Request

请保持改动聚焦，并在描述中包括：

- 问题与预期行为；
- 实现方式及主要权衡；
- 新增或修改的测试；
- GUI 改动的浅色/深色截图；
- 尚未验证的原生 macOS 场景。

