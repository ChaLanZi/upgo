#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────
# upgo-start.sh — 开机自动启动 upgo 开发环境
# ──────────────────────────────────────────────────────────
# 使用方式：
#   手动: ./scripts/upgo-start.sh
#   开机: launchctl load ~/Library/LaunchAgents/com.upgo.plist
# ──────────────────────────────────────────────────────────
set -euo pipefail

UPGO_DIR="$(cd "$(dirname "$0")/.." && pwd)"
LOG_FILE="/tmp/upgo-startup.log"
PID_FILE="/tmp/upgo-pf.pid"

log()  { echo "[$(date '+%H:%M:%S')] $*" | tee -a "$LOG_FILE"; }
die()  { log "FAIL: $*"; exit 1; }

# ── 1. 等待 Docker Desktop 就绪 ────────────────────────
wait_for_docker() {
    log "Waiting for Docker..."
    for i in $(seq 1 30); do
        docker info >/dev/null 2>&1 && { log "Docker ready (${i}s)"; return 0; }
        sleep 2
    done
    die "Docker did not start within 60s"
}

# ── 2. 启动 Minikube ────────────────────────────────────
start_minikube() {
    log "Checking Minikube..."
    if minikube status >/dev/null 2>&1; then
        log "Minikube already running"
    else
        log "Starting Minikube..."
        minikube start --driver=docker --cpus=4 --memory=4096 2>&1 | tee -a "$LOG_FILE"
        log "Minikube started"
    fi
}

# ── 3. 创建 namespace + 部署基础设施 ────────────────────
deploy_infra() {
    cd "$UPGO_DIR"
    log "Creating namespace..."
    kubectl create namespace upgo --dry-run=client -o yaml | kubectl apply -f -
    log "Deploying infrastructure..."
    kubectl apply -k k8s/overlays/dev 2>&1 | tee -a "$LOG_FILE"
    log "Waiting for deployments..."
    kubectl wait --for=condition=Available deployments --all -n upgo --timeout=180s 2>&1 || \
        log "Warning: Some deployments not ready yet (will auto-recover)"
    log "Creating databases..."
    kubectl exec deploy/postgres -n upgo -- psql -U postgres -c "CREATE DATABASE upgo_auth;" 2>/dev/null || true
    kubectl exec deploy/postgres -n upgo -- psql -U postgres -c "CREATE DATABASE upgo_account;" 2>/dev/null || true
}

# ── 4. 端口转发（后台运行）─────────────────────────────
start_port_forward() {
    # Kill any existing port-forwards
    pkill -f "kubectl port-forward.*upgo" 2>/dev/null || true
    sleep 1

    log "Starting port-forwards (background)..."
    kubectl port-forward -n upgo svc/auth 9090:9090 &
    kubectl port-forward -n upgo svc/minio 9001:9001 &
    kubectl port-forward -n upgo svc/rnacos 8848:8848 &
    kubectl port-forward -n upgo svc/mailpit 8025:8025 &

    # Save PIDs for cleanup
    jobs -p > "$PID_FILE" 2>/dev/null || true
    log "Port-forwards started (PIDs saved to $PID_FILE)"
}

# ── 5. 输出服务地址 ────────────────────────────────────
print_urls() {
    echo ""
    echo "╔═══════════════════════════════════════════╗"
    echo "║        upgo 开发环境已就绪 🚀              ║"
    echo "╠═══════════════════════════════════════════╣"
    echo "║                                          ║"
    echo "║  前端应用:  $(minikube service frontend -n upgo --url 2>/dev/null || echo 'N/A')"
    echo "║  Auth健康:  http://localhost:9090/health  ║"
    echo "║  MinIO:     http://localhost:9001         ║"
    echo "║  RNacos:    http://localhost:8848         ║"
    echo "║  Mailpit:   http://localhost:8025         ║"
    echo "║                                          ║"
    echo "║  Pod状态:   kubectl get pods -n upgo      ║"
    echo "║  停止:      just k8s-down                 ║"
    echo "╚═══════════════════════════════════════════╝"
    echo ""
}

# ── Main ────────────────────────────────────────────────
log "=== upgo startup ==="
wait_for_docker
start_minikube
deploy_infra
start_port_forward
print_urls
