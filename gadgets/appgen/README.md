# AppGen - macOS Application Bundle Generator

>[!warning]
> 本工具初步试验发现可能只支持 shell 脚本可执行文件进行打包然后去除黑窗口.

AppGen 是一个用于将可执行文件打包成 macOS 应用程序包（.app 文件夹）的命令行工具。使用此工具，您可以轻松地将任何可执行文件转换为正式的 macOS 应用程序，并可控制是否在运行时显示终端窗口。

## 功能特点

- 将任意可执行文件打包成标准的 macOS .app 应用程序包
- 支持自定义应用程序图标（.icns 格式）
- 自动生成必要的 Info.plist 文件
- 支持设置应用程序版本号、Bundle ID 等元数据
- 支持添加额外的文件和文件夹到应用程序包中
- 可选择是否在应用程序运行时显示终端窗口
- 支持单例模式，确保每个用户只能运行一个应用程序实例

## 安装

从源代码编译：

```bash
# 克隆仓库
git clone https://github.com/azazo1/appgen.git
cd appgen

# 编译
cargo build --release

# 安装到系统中（可选）
cargo install --path .
```

## 使用方法

基本用法：

```bash
appgen --executable <可执行文件路径> --name <应用程序名称>
```

### 命令行选项

| 选项 | 短选项 | 说明 | 默认值 |
|------|--------|------|--------|
| `--executable` | `-e` | 要打包的可执行文件路径 | (必填) |
| `--name` | `-n` | 应用程序名称（不含 .app 后缀） | (必填) |
| `--icon` | `-i` | 应用程序图标路径（.icns 格式） | (可选) |
| `--app-version` | `-v` | 应用程序版本号 | 1.0.0 |
| `--bundle-id` | `-b` | 应用程序包标识符 | com.example.app |
| `--output` | `-o` | 输出目录 | . |
| `--additional-file` | `-a` | 要添加到应用程序包中的额外文件或文件夹 | (可选) |
| `--default-location` | `-d` | 额外文件的默认位置 | resources |
| `--show-terminal` | `-t` | 运行应用程序时显示终端窗口 | false |
| `--single-instance` | `-s` | 确保应用程序在用户范围内仅运行一个实例 | false |

## 使用示例

### 基本示例

```bash
# 最基本的用法
appgen --executable ./my_program --name "My Application"
```

### 添加图标

```bash
appgen --executable ./my_program --name "My Application" --icon ./path/to/icon.icns
```

### 设置元数据

```bash
appgen --executable ./my_program --name "My Application" \
  --app-version "2.1.0" \
  --bundle-id "com.yourcompany.myapp" \
  --output ~/Desktop
```

### 添加额外文件

```bash
# 添加单个文件到特定位置
appgen --executable ./my_program --name "My Application" \
  --additional-file config.json:Resources/config.json

# 添加多个文件或文件夹
appgen --executable ./my_program --name "My Application" \
  --additional-file config.json:Resources/config.json \
  --additional-file images:Resources/images \
  --additional-file LICENSE:Resources

# 更改默认位置
appgen --executable ./my_program --name "My Application" \
  --default-location contents \
  --additional-file README.md
```

### 控制终端窗口显示

```bash
# 创建显示终端窗口的应用
appgen --executable ./my_program --name "My Application" --show-terminal

# 创建不显示终端窗口的应用（默认行为）
appgen --executable ./my_program --name "My Application"
```

### 启用单例模式

```bash
# 创建一个在用户范围内仅允许单实例运行的应用
appgen --executable ./my_program --name "My Application" --single-instance

# 组合使用多个选项
appgen --executable ./my_program --name "My Application" \
  --icon ./path/to/icon.icns \
  --single-instance \
  --show-terminal \
  --bundle-id "com.yourcompany.myapp"
```

```bash
# 打包应用程序，运行时显示终端窗口
appgen --executable ./my_program --name "My Application" --show-terminal

# 完整示例：创建带图标、显示终端窗口并添加额外文件的应用程序
appgen --executable ./my_program --name "My Application" \
  --icon ./path/to/icon.icns \
  --show-terminal \
  --additional-file config.json:Resources/config.json \
  --bundle-id "com.yourcompany.myapp"
```

## 添加文件格式

`--additional-file` 选项的格式是 `源路径:目标路径`，其中：

- `源路径` 是要添加的文件或文件夹的路径
- `目标路径` 是该文件或文件夹在应用程序包中的相对路径（相对于 Contents 目录）

如果不指定目标路径（只提供源路径），文件将被复制到由 `--default-location` 指定的默认位置。

## 综合示例

下面是一个结合多种选项的综合示例：

```bash
appgen --executable ./my_program --name "My Application" \
  --icon ./path/to/icon.icns \
  --app-version "2.1.0" \
  --bundle-id "com.yourcompany.myapp" \
  --output ~/Desktop \
  --additional-file config.json:Resources/config.json \
  --additional-file images:Resources/images \
  --show-terminal
```

上面的命令会创建一个完整的 macOS 应用程序，包含自定义图标、版本号、额外文件和文件夹，并且运行时会显示终端窗口。

## Shell命令补全脚本

AppGen提供了Bash、Zsh和Fish shell的命令行补全脚本，可以帮助您更轻松地使用命令行选项。

### 安装补全脚本

#### Bash

将以下内容添加到您的`~/.bashrc`或`~/.bash_profile`：

```bash
source /path/to/appgen/completions/appgen.bash
```

或将文件复制到您的bash补全目录：

```bash
cp /path/to/appgen/completions/appgen.bash /etc/bash_completion.d/
```

#### Zsh

将补全文件复制到您的`$fpath`目录之一：

```zsh
# 找到您的fpath目录
echo $fpath

# 然后复制文件，例如：
cp /path/to/appgen/completions/_appgen ~/.zsh/completions/
```

或者将补全目录添加到您的`.zshrc`：

```zsh
fpath=(/path/to/appgen/completions $fpath)
autoload -Uz compinit
compinit
```

#### Fish

将补全文件复制到您的fish补全目录：

```fish
cp /path/to/appgen/completions/appgen.fish ~/.config/fish/completions/
```

或创建符号链接：

```fish
ln -s /path/to/appgen/completions/appgen.fish ~/.config/fish/completions/
```

更多详细信息请参阅 [`completions/README.md`](completions/README.md) 文件。

## 辅助脚本

AppGen 包含一些辅助脚本，用于简化应用程序开发过程。

可自行放入 PATH 环境变量下.

### icnsgen - 图标生成工具

`icnsgen` 是一个用于从 PNG 图像生成 macOS 应用程序图标 (.icns) 的辅助脚本。该脚本可以将一个单一的高分辨率 PNG 图像转换为包含各种尺寸的 .icns 图标文件，适用于 macOS 应用程序。

#### icnsgen 使用方法

```bash
./scripts/icnsgen input.png
```

这将生成与输入 PNG 文件同名但扩展名为 .icns 的图标文件（例如：input.icns）。

#### 功能特性

- 自动创建各种所需的图标尺寸（16×16 到 1024×1024）
- 生成 @2x 高分辨率变体
- 输出标准的 .icns 文件，可直接用于 AppGen 的 `--icon` 选项

#### 系统要求

- 需要 macOS 系统
- 使用系统内置的 `sips` 和 `iconutil` 工具

## 许可证

MIT
