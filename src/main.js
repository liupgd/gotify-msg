const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;

let isConnected = false;

// 加载配置
async function loadConfig() {
    try {
        const config = await invoke('load_config');
        document.getElementById('serverUrl').value = config.server_url || '';
        document.getElementById('token').value = config.token || '';
        document.getElementById('timeout').value = config.timeout_seconds || 5;
    } catch (error) {
        console.error('加载配置失败:', error);
    }
}

// 保存配置
async function saveConfig() {
    const serverUrl = document.getElementById('serverUrl').value.trim();
    const token = document.getElementById('token').value.trim();
    const timeout = parseInt(document.getElementById('timeout').value);

    if (!serverUrl || !token) {
        showStatus('请填写所有必填项', 'error');
        return;
    }

    try {
        await invoke('save_config', {
            serverUrl,
            token,
            timeoutSeconds: timeout
        });
        showStatus('配置保存成功', 'success');
    } catch (error) {
        showStatus('保存配置失败: ' + error, 'error');
    }
}

// 启动连接
async function startConnection() {
    const serverUrl = document.getElementById('serverUrl').value.trim();
    const token = document.getElementById('token').value.trim();
    const timeout = parseInt(document.getElementById('timeout').value);

    if (!serverUrl || !token) {
        showStatus('请先配置服务器地址和 Token', 'error');
        return;
    }

    try {
        await invoke('start_gotify_connection', {
            serverUrl,
            token,
            timeoutSeconds: timeout
        });
        isConnected = true;
        document.getElementById('startBtn').disabled = true;
        document.getElementById('stopBtn').disabled = false;
        showStatus('连接已启动', 'success');
    } catch (error) {
        showStatus('启动连接失败: ' + error, 'error');
    }
}

// 停止连接
async function stopConnection() {
    try {
        await invoke('stop_gotify_connection');
        isConnected = false;
        document.getElementById('startBtn').disabled = false;
        document.getElementById('stopBtn').disabled = true;
        showStatus('连接已停止', 'info');
    } catch (error) {
        showStatus('停止连接失败: ' + error, 'error');
    }
}

// 显示状态消息
function showStatus(message, type) {
    const statusEl = document.getElementById('status');
    statusEl.textContent = message;
    statusEl.className = 'status ' + type;
    setTimeout(() => {
        statusEl.className = 'status';
        statusEl.style.display = 'none';
    }, 3000);
}

// 监听 Gotify 消息
listen('gotify-message', (event) => {
    const message = event.payload;
    console.log('收到 Gotify 消息:', message);
    
    // 获取超时时间
    const timeout = parseInt(document.getElementById('timeout').value) || 5;
    
    // 创建通知窗口
    invoke('create_notification_window', {
        title: message.title || '通知',
        message: message.message || '',
        priority: message.priority || 0,
        timeoutSeconds: timeout
    }).catch(error => {
        console.error('创建通知窗口失败:', error);
    });
});

// 初始化
document.addEventListener('DOMContentLoaded', () => {
    loadConfig();
    
    document.getElementById('configForm').addEventListener('submit', (e) => {
        e.preventDefault();
        saveConfig();
    });
    
    document.getElementById('startBtn').addEventListener('click', startConnection);
    document.getElementById('stopBtn').addEventListener('click', stopConnection);
});

