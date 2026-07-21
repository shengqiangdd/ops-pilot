# OpsPilot 部署指南

## 前置条件

- Docker 24+ / Docker Compose v2
- 域名 + DNS A 记录指向公网 IP（如使用 Caddy/Nginx 反代）
- 防火墙开放 80/443（反代）或 3001（直连）

## 快速部署

```bash
# 1. 生成密钥
JWT_SECRET=$(openssl rand -hex 32)
OPSPILOT_MASTER_KEY=$(openssl rand -base64 32)

# 2. 启动（在 backend/ 目录下执行）
cd backend && JWT_SECRET=$JWT_SECRET OPSPILOT_MASTER_KEY=$OPSPILOT_MASTER_KEY docker compose up -d

# 3. 访问
# 浏览器打开 http://localhost:3001
# 注册管理员账号 → 登录 → 设置 Vault Passphrase
```

## TLS/SSL 配置（推荐 Caddy 反代）

在 `docker-compose.yml` 中添加 caddy 服务：

```yaml
  caddy:
    image: caddy:2
    container_name: ops-pilot-caddy
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      - caddy_data:/data
      - caddy_config:/config
    restart: unless-stopped
    networks:
      - ops-pilot
```

创建 `Caddyfile`：

```
ops-pilot.example.com {
    reverse_proxy ops-pilot:3001
}
```

然后在 volumes 中添加 `caddy_data` 和 `caddy_config`，重启即可自动申请 HTTPS 证书。

## 环境变量

| 变量 | 必填 | 默认值 | 说明 |
|------|------|--------|------|
| `JWT_SECRET` | **是** | — | JWT 签名密钥，`openssl rand -hex 32` |
| `OPSPILOT_MASTER_KEY` | **是** | — | 主机凭据加密主密钥，`openssl rand -base64 32` |
| `DATABASE_URL` | 否 | `sqlite:///app/data/ops-pilot.db` | SQLite 连接串 |
| `LLM_PROVIDER` | 否 | `ollama` | LLM 提供商：`openai` / `ollama` / `deepseek` |
| `LLM_BASE_URL` | 否 | `http://ollama:11434/v1` | LLM API 地址 |
| `LLM_API_KEY` | 否 | — | LLM API 密钥（OpenAI/DeepSeek 等需要） |
| `LLM_MODEL` | 否 | `qwen2.5:32b` | 聊天模型名称 |
| `RUST_LOG` | 否 | `ops_pilot=info,tower_http=info` | 日志级别 |
| `LISTEN_ADDR` | 否 | `0.0.0.0:3001` | 监听地址 |
| `TZ` | 否 | `UTC` | 时区 |

## 安全检查清单

- [ ] `JWT_SECRET` 已设置（32+ 字节随机 hex）
- [ ] `OPSPILOT_MASTER_KEY` 已设置（32 字节 base64）
- [ ] HTTPS 已配置（Caddy / Nginx + Let's Encrypt）
- [ ] 防火墙仅开放 443（或 80+443）
- [ ] 管理员账号已注册并测试登录
- [ ] Vault Passphrase 已设置（主机凭据独立加密）
- [ ] Rate limit 生效（5 次/分钟 登录尝试）
- [ ] 审计日志可查

## 生产环境建议

### 数据持久化

数据存储在 Docker volume `ops_pilot_data` 中，挂载到 `/app/data/`。

```bash
# 备份数据库
docker exec ops-pilot cat /app/data/ops-pilot.db > backup/ops-pilot-$(date +%Y%m%d).db
```

### 日志管理

docker-compose 已配置日志轮转（`max-size: 10m`, `max-file: 3`）。

### 监控

- 内置健康检查：`GET /api/v1/health`
- 推荐：Uptime Kuma / Prometheus + Grafana

### 资源限制

在 docker-compose.yml 中添加：

```yaml
  ops-pilot:
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: "1.0"
```

## Kubernetes 部署（Helm）

### 前置条件

- Kubernetes 1.24+
- Helm 3.8+
- （可选）cert-manager 用于自动 HTTPS 证书
- （可选）Prometheus Operator 用于 ServiceMonitor

### 安装

```bash
# 1. 克隆仓库
git clone https://github.com/shengqiangdd/ops-pilot.git

# 2. 生成密钥
JWT_SECRET=$(openssl rand -hex 32)
OPSPILOT_MASTER_KEY=$(openssl rand -base64 32)

# 3. 安装 Chart
helm upgrade --install ops-pilot deploy/helm/ops-pilot \
  --set env.JWT_SECRET="$JWT_SECRET" \
  --set env.OPSPILOT_MASTER_KEY="$OPSPILOT_MASTER_KEY" \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=ops-pilot.example.com \
  --namespace ops-pilot --create-namespace

# 4. 验证
kubectl get pods -n ops-pilot
kubectl get svc -n ops-pilot
```

### 使用 Secret 传递密钥（更安全）

创建 `values-prod.yaml`：

```yaml
ingress:
  enabled: true
  hosts:
    - host: ops-pilot.example.com
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  tls:
    - secretName: ops-pilot-tls
      hosts:
        - ops-pilot.example.com

secrets:
  JWT_SECRET: "<base64-encoded-value>"
  OPSPILOT_MASTER_KEY: "<base64-encoded-value>"

persistence:
  storageClass: "ssd"
  size: 5Gi

resources:
  limits:
    cpu: 2
    memory: 1Gi
  requests:
    cpu: 500m
    memory: 512Mi

serviceMonitor:
  enabled: true
```

```bash
helm upgrade --install ops-pilot deploy/helm/ops-pilot \
  -f values-prod.yaml \
  --namespace ops-pilot --create-namespace
```

### 卸载

```bash
helm uninstall ops-pilot -n ops-pilot
kubectl delete pvc -l app.kubernetes.io/instance=ops-pilot -n ops-pilot
```

### Chart 结构

```
deploy/helm/ops-pilot/
├── Chart.yaml              # Chart 元数据
├── values.yaml             # 默认配置
├── ci/
│   └── ci-values.yaml      # CI 测试配置
└── templates/
    ├── _helpers.tpl        # 模板辅助函数
    ├── deployment.yaml     # Deployment + Pod
    ├── service.yaml        # Service
    ├── ingress.yaml        # Ingress（可选）
    ├── pvc.yaml            # PersistentVolumeClaim
    ├── secrets.yaml        # Secrets（可选）
    ├── serviceaccount.yaml # ServiceAccount
    ├── hpa.yaml            # HPA（可选）
    ├── pdb.yaml            # PDB（可选）
    ├── networkpolicy.yaml  # NetworkPolicy（可选）
    └── servicemonitor.yaml # ServiceMonitor（可选）
```
