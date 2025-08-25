# appgen的Shell补全脚本

本目录包含`appgen`工具的shell补全脚本。

## 安装说明

### Bash

将以下内容添加到您的`~/.bashrc`或`~/.bash_profile`：

```bash
source /path/to/appgen/completions/appgen.bash
```

或将文件复制到您的bash补全目录：

```bash
cp /path/to/appgen/completions/appgen.bash /etc/bash_completion.d/
```

### Zsh

将补全文件复制到您的`$fpath`目录之一：

```zsh
# 查找您的fpath目录
echo $fpath

# 然后将文件复制到其中一个目录，例如：
cp /path/to/appgen/completions/_appgen ~/.zsh/completions/
```

或者将补全目录添加到您的`.zshrc`：

```zsh
fpath=(/path/to/appgen/completions $fpath)
autoload -Uz compinit
compinit
```

### Fish

将补全文件复制到您的fish补全目录：

```fish
cp /path/to/appgen/completions/appgen.fish ~/.config/fish/completions/
```

或创建符号链接：

```fish
ln -s /path/to/appgen/completions/appgen.fish ~/.config/fish/completions/
```

## 使用方法

安装后，您可以在使用`appgen`命令时使用Tab键自动补全：

```
appgen --<tab>
```

这将显示所有可用选项。在适用的情况下，Tab补全也适用于选项值。
