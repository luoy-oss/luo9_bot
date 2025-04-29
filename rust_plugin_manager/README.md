Luo9 机器人的插件管理系统的 Rust 实现，旨在提高性能并保持与现有 Python 代码的兼容性。
## 安装指南
### 前提条件
- Rust 工具链 (rustc, cargo)
  - sudo apt install rustc
  - sudo apt install cargo
- Python 3.7+

### 安装步骤

1. 安装 maturin

```bash
pip install maturin
```

2. 构建 Rust 库

如果你使用conda管理python库环境，可以直接使用以下代码进行构建，否则请使用maturin build --release进行构建
```bash
cd rust_plugin_manager
maturin develop --release
```

使用maturin build --release进行构建

maturin build --release构建成功后会出现：📦 Built wheel for CPython 3.x to /home/路径/xxxx.whl

请进一步使用pip3 install xxxxx.whl进行安装
```bash
cd rust_plugin_manager
maturin build --release
pip3 install xxxxx.whl  # xxxxx.whl请替换为编译出的实际的whl文件路径
```
