# LaunchIt - macOS Launch Agent Manager

在 macOS 中以交互式的方式管理 LaunchAgents.

## 功能

- 列出 launchctl 中 `gui/$(id -u)` 下的服务
- 显示服务状态 (运行中、已停止、错误状态)
- 创建新的启动项在 `gui/$(id -u)` 下
  - 设置启动项域名信息
  - 设置启动项程序和参数
  - 配置 KeepAlive 选项
  - 配置标准输出和标准错误流路径
  - 配置 StartInterval (可选)
- 管理现有启动项
  - 启用服务
  - 禁用服务
  - 卸载服务

## 使用方法

从项目根目录运行:

```sh
cargo run --bin launchit
```

### 键盘快捷键

主界面:
- `j`/`k` 或 `Up`/`Down`: 选择服务
- `Enter`: 查看服务详情
- `n`: 创建新的服务
- `r`: 刷新服务列表
- `q`: 退出程序

服务详情界面:
- `j`/`k` 或 `Up`/`Down`: 选择操作
- `Enter`: 执行选定操作
- `Esc`/`q`: 返回主界面

创建新服务界面:
- `Tab`/`j`/`k` 或 `Up`/`Down`: 在表单字段间导航
- `Enter`: 选择/编辑字段
- `Esc`: 取消创建
