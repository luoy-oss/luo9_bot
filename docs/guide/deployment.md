# 部署

## 本地部署

### Windows

```bash
cargo build --release
target/release/luo9_bot.exe
```

### Linux

```bash
cargo build --release
./target/release/luo9_bot
```

## systemd 服务（Linux）

创建 `/etc/systemd/system/luo9-bot.service`：

```ini
[Unit]
Description=luo9_bot QQ Bot
After=network.target

[Service]
Type=simple
User=luo9
WorkingDirectory=/opt/luo9_bot/rust
ExecStart=/opt/luo9_bot/rust/target/release/luo9_bot
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable luo9-bot
sudo systemctl start luo9-bot
sudo journalctl -u luo9-bot -f  # 查看日志
```

## Docker 部署

### Dockerfile

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/rust/target/release/luo9_bot .
COPY --from=builder /app/rust/config ./config
COPY --from=builder /app/rust/plugins ./plugins
CMD ["./luo9_bot"]
```

### docker-compose.yml

```yaml
version: '3.8'
services:
  luo9-bot:
    build: .
    container_name: luo9-bot
    restart: unless-stopped
    ports:
      - "27001:27001"
      - "27080:27080"
    volumes:
      - ./config:/app/config
      - ./plugins:/app/plugins
```

## 反向代理（Nginx）

```nginx
server {
    listen 443 ssl;
    server_name bot.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://127.0.0.1:27080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## 安全建议

1. **防火墙**：仅开放必要端口
2. **HTTPS**：生产环境务必启用
3. **Token**：设置强随机 token
4. **备份**：定期备份 `config/` 和 `plugins/` 目录
