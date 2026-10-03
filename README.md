# SyFB's rodioplay
一个特别简单的音频播放器, 用于给作者研究 rodio 库

如果你需要使用 `rodio` 来播放游戏中的音乐或音效，那么这个仓库或许会有点用处

使用方法见 `rodioplay --help`，这里不再赘述

## 构建
使用以下命令为当前平台构建: 
```bash
cargo build -r
```

我写了一些构建脚本，那些脚本会自动将构建结果将会被复制到 `/build` 文件夹: 
```bash
./compile.sh
```

### 交叉编译
目前可以使用构建脚本为以下平台构建，并把构建结果复制到 `/build` 文件夹: 
```text
aarch64-pc-windows-gnullvm
i686-pc-windows-gnu
i686-unknown-linux-gnu
x86_64-pc-windows-gnu
x86_64-unknown-linux-gnu
```

通过以下命令来为指定平台构建，其中，`<TARGET>` 是目标名: 
```bash
./compile-cross.sh <TARGET>
```

通过以下命令来为所有平台构建: 
```bash
./compile-all.sh
```

如果你需要为其它平台构建，使用 Cargo，其中，`<TARGET>` 是目标名: 
```bash
cargo build -r --target <TARGET>
```

## 更新日志

### 1.100
- 将名字 rodio-test 改为 rodioplay
- 添加 `mix` 和 `advmix` 子命令，用于研究音效的播放

### 1.000
- 初版
