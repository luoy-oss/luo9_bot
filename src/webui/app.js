// ═══════════════════════════════════════════
//   Luo9 Bot 控制台 — 前端逻辑
// ═══════════════════════════════════════════

(function () {
  'use strict';

  // ── 状态 ───────────────────────────────
  let logOffset = 0;
  let autoScroll = true;
  let storeData = [];
  let activeTag = 'all';
  let confirmResolve = null;
  let authToken = null;

  // ── Token 管理 ─────────────────────────
  function initToken() {
    // 1. 从 URL 参数获取 token
    const params = new URLSearchParams(window.location.search);
    const urlToken = params.get('token');

    if (urlToken) {
      // URL 中有 token，保存到 localStorage
      authToken = urlToken;
      localStorage.setItem('luo9_token', urlToken);
      // 清除 URL 中的 token 参数（避免泄露）
      const url = new URL(window.location);
      url.searchParams.delete('token');
      window.history.replaceState({}, '', url);
      return true;
    }

    // 2. 从 localStorage 获取 token
    const savedToken = localStorage.getItem('luo9_token');
    if (savedToken) {
      authToken = savedToken;
      return true;
    }

    // 3. 没有 token
    return false;
  }

  // ── 初始化 ─────────────────────────────
  document.addEventListener('DOMContentLoaded', () => {
    if (!initToken()) {
      // 没有 token，显示错误提示
      document.body.innerHTML = `
        <div style="display:flex;align-items:center;justify-content:center;height:100vh;
                    background:#fef7ff;font-family:system-ui;">
          <div style="text-align:center;padding:2rem;background:white;border-radius:16px;
                      box-shadow:0 4px 20px rgba(0,0,0,0.08);max-width:400px;">
            <div style="font-size:3rem;margin-bottom:1rem;">🔒</div>
            <h2 style="margin:0 0 0.5rem;color:#6b21a8;">需要访问令牌</h2>
            <p style="color:#666;margin:0 0 1.5rem;">
              请在 URL 中添加 <code style="background:#f3e8ff;padding:2px 6px;border-radius:4px;">?token=xxxx</code> 参数访问
            </p>
            <p style="color:#999;font-size:0.85rem;">
              首次启动时请查看控制台输出的 token 地址
            </p>
          </div>
        </div>
      `;
      return;
    }

    initTabs();
    initUpload();
    initLogs();
    initConfirm();
    refreshAll();
    // 定时刷新状态
    setInterval(refreshStatus, 10000);
  });

  // ── API 请求辅助 ─────────────────────
  function apiUrl(path) {
    const separator = path.includes('?') ? '&' : '?';
    return `${path}${separator}token=${encodeURIComponent(authToken)}`;
  }

  // ── Tab 切换 ───────────────────────────
  function initTabs() {
    document.querySelectorAll('.tab').forEach(btn => {
      btn.addEventListener('click', () => {
        document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
        document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
        btn.classList.add('active');
        const target = document.getElementById('tab-' + btn.dataset.tab);
        if (target) target.classList.add('active');

        // 切换到日志时刷新
        if (btn.dataset.tab === 'logs') refreshLogs();
        // 切换到商店时加载
        if (btn.dataset.tab === 'store') refreshStore();
      });
    });
  }

  // ── 全量刷新 ───────────────────────────
  function refreshAll() {
    refreshStatus();
    refreshInstalled();
    refreshStore();
    refreshLogs();
  }

  // ── 状态 API ───────────────────────────
  async function refreshStatus() {
    try {
      const resp = await fetch(apiUrl('/api/status'));
      const data = await resp.json();
      document.getElementById('stat-uptime').textContent = formatUptime(data.uptime_secs);
      document.getElementById('stat-plugins').textContent = data.plugin_count;
      document.getElementById('stat-dir').textContent = data.plugin_dir;
    } catch (e) {
      console.error('获取状态失败:', e);
    }
  }

  function formatUptime(secs) {
    if (secs < 60) return secs + '秒';
    if (secs < 3600) return Math.floor(secs / 60) + '分' + (secs % 60) + '秒';
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    return h + '时' + m + '分';
  }

  // ── 已安装插件 ─────────────────────────
  async function refreshInstalled() {
    try {
      const resp = await fetch(apiUrl('/api/plugins'));
      const plugins = await resp.json();
      const list = document.getElementById('installed-list');
      const empty = document.getElementById('installed-empty');

      if (plugins.length === 0) {
        list.innerHTML = '';
        empty.style.display = '';
        return;
      }
      empty.style.display = 'none';

      list.innerHTML = plugins.map((p, i) => `
        <div class="plugin-item" style="animation-delay:${i * 0.05}s">
          <div class="plugin-info">
            <div class="plugin-name">${esc(p.name)}</div>
            <div class="plugin-file">${esc(p.file)}</div>
            <div class="plugin-controls">
              <label class="control-label" title="优先级越高越先收到消息">
                优先级:
                <input type="number" class="priority-input" value="${p.priority || 0}" min="-100" max="100"
                       onchange="setPriority('${esc(p.name)}', this.value)">
              </label>
              <label class="control-label" title="启用后高优先级插件可阻断低优先级插件收到消息">
                <input type="checkbox" ${p.block_enabled ? 'checked' : ''}
                       onchange="setBlock('${esc(p.name)}', this.checked)">
                阻断
              </label>
            </div>
          </div>
          <div class="plugin-meta">
            <span class="badge ${p.active ? 'badge-on' : 'badge-off'}">${p.active ? '运行中' : '已停止'}</span>
            <span class="badge ${p.enabled ? 'badge-on' : 'badge-off'}">${p.enabled ? '启用' : '禁用'}</span>
            <button class="btn btn-sm btn-accent" onclick="reloadPlugin('${esc(p.name)}')" title="热重载">重载</button>
            <button class="btn btn-sm ${p.enabled ? 'btn-danger' : 'btn-success'}"
                    onclick="togglePlugin('${esc(p.name)}', ${p.enabled})">
              ${p.enabled ? '禁用' : '启用'}
            </button>
            <button class="btn btn-sm btn-danger" onclick="deletePlugin('${esc(p.name)}')">删除</button>
          </div>
        </div>
      `).join('');
    } catch (e) {
      console.error('获取插件列表失败:', e);
    }
  }

  // ── 插件商店 ───────────────────────────
  async function refreshStore() {
    const loading = document.getElementById('store-loading');
    const empty = document.getElementById('store-empty');
    const list = document.getElementById('store-list');

    loading.style.display = '';
    empty.style.display = 'none';

    try {
      const resp = await fetch(apiUrl('/api/registry'));
      if (!resp.ok) {
        throw new Error('HTTP ' + resp.status);
      }
      storeData = await resp.json();
      loading.style.display = 'none';

      if (storeData.length === 0) {
        list.innerHTML = '';
        empty.style.display = '';
        return;
      }

      // 收集所有 tags
      const allTags = new Set();
      storeData.forEach(p => (p.tags || []).forEach(t => allTags.add(t)));
      renderTagFilter(allTags);
      renderStoreList();
    } catch (e) {
      loading.style.display = 'none';
      list.innerHTML = `<div class="empty"><div class="icon">⚠️</div><div>加载注册表失败: ${esc(e.message)}</div></div>`;
    }
  }

  function renderTagFilter(tags) {
    const container = document.getElementById('tag-filter');
    container.innerHTML = '<button class="tag-chip active" data-tag="all">全部</button>';
    [...tags].sort().forEach(tag => {
      const btn = document.createElement('button');
      btn.className = 'tag-chip';
      btn.dataset.tag = tag;
      btn.textContent = tag;
      container.appendChild(btn);
    });

    container.querySelectorAll('.tag-chip').forEach(chip => {
      chip.addEventListener('click', () => {
        container.querySelectorAll('.tag-chip').forEach(c => c.classList.remove('active'));
        chip.classList.add('active');
        activeTag = chip.dataset.tag;
        renderStoreList();
      });
    });
  }

  function renderStoreList() {
    const list = document.getElementById('store-list');
    const filtered = activeTag === 'all'
      ? storeData
      : storeData.filter(p => (p.tags || []).includes(activeTag));

    if (filtered.length === 0) {
      list.innerHTML = '<div class="empty"><div class="icon">🔍</div><div>该分类下暂无插件</div></div>';
      return;
    }

    list.innerHTML = filtered.map((p, i) => {
      const tagsHtml = (p.tags || []).map(t => `<span class="badge badge-tag">${esc(t)}</span>`).join(' ');
      const installed = p.installed;
      const sdkVer = p.sdk_version || '';
      const ghUrl = `https://github.com/${p.repo}`;

      return `
        <div class="plugin-item" style="animation-delay:${i * 0.05}s">
          <div class="plugin-info">
            <div class="plugin-name">
              ${esc(p.name)}
              ${tagsHtml}
            </div>
            <div class="plugin-desc">${esc(p.description)}</div>
            ${sdkVer ? `<div class="plugin-file sdk-toggle" onclick="this.nextElementSibling.classList.toggle('show')">SDK: ${esc(sdkVer)} ▾</div><div class="sdk-details">SDK 版本: ${esc(sdkVer)}</div>` : ''}
          </div>
          <div class="plugin-meta">
            <span class="badge badge-ver">v${esc(p.latest_version)}</span>
            ${installed
              ? '<span class="badge badge-on">已安装</span>'
              : `<button class="btn btn-sm btn-accent" onclick="installPlugin('${esc(p.name)}')">安装</button>`
            }
            <a class="gh-link" href="${ghUrl}" target="_blank" title="GitHub">🔗</a>
          </div>
        </div>
      `;
    }).join('');
  }

  // ── 插件操作 ───────────────────────────
  window.togglePlugin = async function (name, enabled) {
    const action = enabled ? '禁用' : '启用';
    const ok = await showConfirm(`${action}插件`, `确定要${action}插件 ${name} 吗？`);
    if (!ok) return;

    const endpoint = enabled ? 'disable' : 'enable';
    try {
      const resp = await fetch(apiUrl(`/api/plugins/${encodeURIComponent(name)}/${endpoint}`), { method: 'POST' });
      const data = await resp.json();
      showToast(data.message, data.ok);
      if (data.ok) refreshInstalled();
    } catch (e) {
      showToast('请求失败: ' + e.message, false);
    }
  };

  window.reloadPlugin = async function (name) {
    const ok = await showConfirm('热重载插件', `确定要热重载插件 ${name} 吗？`);
    if (!ok) return;

    try {
      const resp = await fetch(apiUrl(`/api/plugins/${encodeURIComponent(name)}/reload`), { method: 'POST' });
      const data = await resp.json();
      showToast(data.message, data.ok);
      if (data.ok) refreshInstalled();
    } catch (e) {
      showToast('请求失败: ' + e.message, false);
    }
  };

  window.setPriority = async function (name, value) {
    const priority = parseInt(value, 10);
    if (isNaN(priority)) return;

    try {
      const resp = await fetch(apiUrl(`/api/plugins/${encodeURIComponent(name)}/priority`), {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ priority }),
      });
      const data = await resp.json();
      showToast(data.message, data.ok);
    } catch (e) {
      showToast('请求失败: ' + e.message, false);
    }
  };

  window.setBlock = async function (name, blockEnabled) {
    try {
      const resp = await fetch(apiUrl(`/api/plugins/${encodeURIComponent(name)}/block`), {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ block_enabled: blockEnabled }),
      });
      const data = await resp.json();
      showToast(data.message, data.ok);
    } catch (e) {
      showToast('请求失败: ' + e.message, false);
    }
  };

  window.deletePlugin = async function (name) {
    const ok = await showConfirm('删除插件', `确定要删除插件 ${name} 吗？此操作不可撤销。`);
    if (!ok) return;

    try {
      const resp = await fetch(apiUrl(`/api/plugins/${encodeURIComponent(name)}`), { method: 'DELETE' });
      const data = await resp.json();
      showToast(data.message, data.ok);
      if (data.ok) {
        refreshInstalled();
        refreshStatus();
      }
    } catch (e) {
      showToast('请求失败: ' + e.message, false);
    }
  };

  window.installPlugin = async function (name) {
    const ok = await showConfirm('安装插件', `确定要从注册表安装插件 ${name} 吗？`);
    if (!ok) return;

    try {
      const resp = await fetch(apiUrl(`/api/plugins/install/${encodeURIComponent(name)}`), { method: 'POST' });
      const data = await resp.json();
      showToast(data.message, data.ok);
      if (data.ok) {
        refreshInstalled();
        refreshStore();
        refreshStatus();
      }
    } catch (e) {
      showToast('请求失败: ' + e.message, false);
    }
  };

  // ── 上传 ───────────────────────────────
  function initUpload() {
    const zone = document.getElementById('upload-zone');
    const input = document.getElementById('upload-input');

    zone.addEventListener('click', () => input.click());

    zone.addEventListener('dragover', e => {
      e.preventDefault();
      zone.classList.add('dragover');
    });

    zone.addEventListener('dragleave', () => {
      zone.classList.remove('dragover');
    });

    zone.addEventListener('drop', e => {
      e.preventDefault();
      zone.classList.remove('dragover');
      uploadFiles(e.dataTransfer.files);
    });

    input.addEventListener('change', () => {
      if (input.files.length > 0) uploadFiles(input.files);
      input.value = '';
    });
  }

  async function uploadFiles(files) {
    const formData = new FormData();
    let count = 0;
    for (const f of files) {
      const lower = f.name.toLowerCase();
      if (lower.endsWith('.dll') || lower.endsWith('.so')) {
        formData.append('file', f);
        count++;
      }
    }
    if (count === 0) {
      showToast('请选择 .dll 或 .so 文件', false);
      return;
    }

    try {
      const resp = await fetch(apiUrl('/api/plugins/upload'), { method: 'POST', body: formData });
      const data = await resp.json();
      showToast(data.message, data.ok);
      if (data.ok) {
        refreshInstalled();
        refreshStatus();
      }
    } catch (e) {
      showToast('上传失败: ' + e.message, false);
    }
  }

  // ── 日志 ───────────────────────────────
  function initLogs() {
    const checkbox = document.getElementById('log-auto-scroll');
    checkbox.addEventListener('change', () => { autoScroll = checkbox.checked; });
    document.getElementById('log-refresh-btn').addEventListener('click', () => {
      logOffset = 0;
      document.getElementById('log-box').innerHTML = '';
      refreshLogs();
    });
  }

  async function refreshLogs() {
    try {
      const resp = await fetch(apiUrl(`/api/logs?after=${logOffset}`));
      const data = await resp.json();

      if (data.lines.length === 0) return;

      const box = document.getElementById('log-box');
      data.lines.forEach(line => {
        const div = document.createElement('div');
        div.className = 'log-line';

        // 根据日志级别着色
        if (line.includes('ERROR')) div.classList.add('log-ERROR');
        else if (line.includes('WARN')) div.classList.add('log-WARN');
        else if (line.includes('INFO')) div.classList.add('log-INFO');
        else if (line.includes('DEBUG')) div.classList.add('log-DEBUG');
        else if (line.includes('TRACE')) div.classList.add('log-TRACE');

        div.textContent = line;
        box.appendChild(div);
      });

      logOffset = data.total;
      document.getElementById('log-count').textContent = logOffset + ' 行';

      if (autoScroll) {
        box.scrollTop = box.scrollHeight;
      }
    } catch (e) {
      console.error('获取日志失败:', e);
    }
  }

  // 定时拉取日志
  setInterval(() => {
    const logsTab = document.getElementById('tab-logs');
    if (logsTab && logsTab.classList.contains('active')) {
      refreshLogs();
    }
  }, 3000);

  // ── Toast ──────────────────────────────
  function showToast(message, ok) {
    const toast = document.getElementById('toast');
    toast.textContent = message;
    toast.className = 'toast ' + (ok ? 'toast-ok' : 'toast-err');
    // 触发 reflow 以重新播放动画
    void toast.offsetWidth;
    toast.classList.add('show');
    setTimeout(() => toast.classList.remove('show'), 3000);
  }
  window.showToast = showToast;

  // ── Confirm Dialog ─────────────────────
  function initConfirm() {
    document.getElementById('confirm-cancel').addEventListener('click', () => {
      closeConfirm(false);
    });
    document.getElementById('confirm-ok').addEventListener('click', () => {
      closeConfirm(true);
    });
    document.getElementById('confirm-overlay').addEventListener('click', e => {
      if (e.target === e.currentTarget) closeConfirm(false);
    });
  }

  function showConfirm(title, message) {
    return new Promise(resolve => {
      document.getElementById('confirm-title').textContent = title;
      document.getElementById('confirm-message').textContent = message;
      document.getElementById('confirm-overlay').classList.add('show');
      confirmResolve = resolve;
    });
  }

  function closeConfirm(result) {
    document.getElementById('confirm-overlay').classList.remove('show');
    if (confirmResolve) {
      confirmResolve(result);
      confirmResolve = null;
    }
  }

  // ── 工具 ───────────────────────────────
  function esc(str) {
    if (!str) return '';
    return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
  }

})();
