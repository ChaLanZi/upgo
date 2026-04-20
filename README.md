# upgo

全栈 Rust 股票市场应用程序，基于 Kubernetes 微服务架构。

## 前置依赖

| 工具 | 安装方式 |
|------|---------|
| Rust 1.85+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` |
| Minikube | `brew install minikube`（macOS）或 minikube.sigs.k8s.io |
| kubectl | `brew install kubectl` |
| Docker | docker.com/products/docker-desktop |
| Just | `cargo install just` |
| Dagger | `curl -fsSL https://dl.dagger.io/dagger/install.sh | sh` |
| cargo-nextest | `cargo install cargo-nextest` |

## 一键启动

```bash
# 启动 Minikube + 部署基础设施（PostgreSQL、NATS、Redis）
just k8s-up

# 运行全部测试
just test

# 暂停集群（保留数据）
just k8s-down

# 完全重置
just k8s-reset
```

## Dagger CI/CD

```bash
# 本地执行 CI 流水线（cargo check + cargo nextest run）
just dagger-ci

# 完整流程：启动集群 → 流水线 → 清理
just ci
```

## 仅运行 Rust 测试

```bash
# 单元测试（无需基础设施）
cargo test --lib

# 全部测试（集成测试需 Docker）
cargo nextest run
```

## 项目结构

```
upgo/
├── contracts/            # proto 编译集中管理
├── services/account/     # account 微服务
├── k8s/base/             # k8s 基础设施清单
├── k8s/overlays/dev/     # 环境覆盖
├── ci/                   # Dagger CI/CD 模块
├── Justfile              # 一键命令入口
└── .env                  # 环境变量默认值
```

## 技术栈

全栈 Rust，Axum (HTTP) + Tonic (gRPC)，PostgreSQL，NATS JetStream，Redis，Minikube/k8s，SigNoz (OpenTelemetry)，RNacos，Pingora 网关，Dioxus 前端。
