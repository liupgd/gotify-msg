// 等待 Tauri API 加载完成
let invoke, listen;
let isConnected = false;

// 初始化 Tauri API
async function initTauri() {
    console.log('开始初始化 Tauri API...');
    console.log('当前 window.__TAURI__:', typeof window.__TAURI__);
    
    // 等待 Tauri API 加载（最多等待 10 秒，因为 HTTP 服务器模式可能需要更长时间）
    for (let i = 0; i < 100; i++) {
        if (typeof window.__TAURI__ !== 'undefined') {
            console.log(`第 ${i + 1} 次检查: window.__TAURI__ 已存在`);
            console.log('window.__TAURI__ 的键:', Object.keys(window.__TAURI__));
            
            // Tauri 2.0 使用 core 而不是 tauri
            if (window.__TAURI__.core && window.__TAURI__.core.invoke) {
                invoke = window.__TAURI__.core.invoke;
                console.log('✓ 使用 Tauri 2.0 core API');
            } else if (window.__TAURI__.tauri && window.__TAURI__.tauri.invoke) {
                // 兼容旧版本
                invoke = window.__TAURI__.tauri.invoke;
                console.log('✓ 使用 Tauri 1.x API');
            } else {
                console.log('⚠ window.__TAURI__ 存在，但找不到 invoke 方法');
                console.log('window.__TAURI__.core:', window.__TAURI__.core);
                console.log('window.__TAURI__.tauri:', window.__TAURI__.tauri);
            }
            
            // Tauri 2.0 事件 API 路径
            // 根据 notification.html 中的用法，应该使用 window.__TAURI__.event.listen
            console.log('检查事件 API 路径...');
            console.log('window.__TAURI__:', window.__TAURI__);
            console.log('window.__TAURI__.event:', window.__TAURI__?.event);
            console.log('window.__TAURI__.core:', window.__TAURI__?.core);
            
            // 优先使用 core.event.listen（Tauri 2.0 标准方式，需要权限）
            // 如果权限配置正确，应该使用 core.event.listen
            if (window.__TAURI__?.core?.event?.listen) {
                listen = window.__TAURI__.core.event.listen;
                console.log('✓ 使用 Tauri 2.0 core.event.listen（需要权限配置）');
            } else if (window.__TAURI__?.event?.listen) {
                listen = window.__TAURI__.event.listen;
                console.log('✓ 使用 window.__TAURI__.event.listen（兼容模式）');
            } else {
                console.error('⚠ event.listen 未找到');
                if (window.__TAURI__) {
                    console.log('window.__TAURI__ 的键:', Object.keys(window.__TAURI__));
                    if (window.__TAURI__.event) {
                        console.log('event 的键:', Object.keys(window.__TAURI__.event));
                    }
                    if (window.__TAURI__.core) {
                        console.log('core 的键:', Object.keys(window.__TAURI__.core));
                        if (window.__TAURI__.core.event) {
                            console.log('core.event 的键:', Object.keys(window.__TAURI__.core.event));
                        }
                    }
                }
            }
            
            if (invoke && listen) {
                console.log('✓✓ Tauri API 已加载成功！');
                console.log('完整的 API 结构:', Object.keys(window.__TAURI__));
                return true;
            }
        } else {
            if (i % 10 === 0) {
                console.log(`等待 Tauri API 加载... (${i * 100}ms)`);
            }
        }
        // 等待 100ms 后重试
        await new Promise(resolve => setTimeout(resolve, 100));
    }
    
    console.error('❌ Tauri API 未找到，请确保在 Tauri 应用中运行');
    console.log('window.__TAURI__:', typeof window.__TAURI__);
    if (typeof window.__TAURI__ !== 'undefined') {
        console.log('window.__TAURI__ 内容:', window.__TAURI__);
        console.log('可用的键:', Object.keys(window.__TAURI__ || {}));
    } else {
        console.log('window.__TAURI__ 完全未定义');
        console.log('这可能意味着：');
        console.log('1. 不是在 Tauri 应用中运行（直接在浏览器中打开）');
        console.log('2. Tauri API 注入失败');
        console.log('3. 使用了错误的 URL（应该通过 Tauri 应用访问）');
    }
    return false;
}

// 加载配置
async function loadConfig() {
    if (!invoke) {
        console.error('Tauri API 未初始化，无法加载配置');
        return;
    }
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
    if (!invoke) {
        console.error('Tauri API 未初始化，无法保存配置');
        showStatus('Tauri API 未初始化', 'error');
        return;
    }
    
    const serverUrl = document.getElementById('serverUrl').value.trim();
    const token = document.getElementById('token').value.trim();
    const timeout = parseInt(document.getElementById('timeout').value);

    console.log('saveConfig 被调用，参数:', { serverUrl, token: token ? '***' : '', timeout });

    if (!serverUrl || !token) {
        showStatus('请填写所有必填项', 'error');
        return;
    }

    try {
        console.log('准备调用 invoke save_config，参数:', { serverUrl, token: '***', timeoutSeconds: timeout });
        const result = await invoke('save_config', {
            serverUrl,
            token,
            timeoutSeconds: timeout
        });
        console.log('save_config 调用成功，返回值:', result);
        showStatus('配置保存成功', 'success');
    } catch (error) {
        console.error('save_config 调用失败:', error);
        showStatus('保存配置失败: ' + error, 'error');
    }
}

// 启动连接
async function startConnection() {
    if (!invoke) {
        console.error('Tauri API 未初始化');
        return;
    }
    
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
    if (!invoke) {
        console.error('Tauri API 未初始化');
        return;
    }
    
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
async function setupMessageListener() {
    if (!listen || !invoke) {
        console.error('Tauri API 未初始化，无法设置消息监听');
        console.error('listen:', typeof listen, 'invoke:', typeof invoke);
        return false;
    }
    
    console.log('准备设置消息监听器，listen 函数:', typeof listen);
    console.log('listen 函数对象:', listen);
    
    try {
        // 检查 listen 函数是否存在
        if (typeof listen !== 'function') {
            throw new Error('listen 不是一个函数，类型: ' + typeof listen);
        }
        
        console.log('调用 listen 函数，事件名: gotify-message');
        
        // 根据 notification.html 的用法，直接调用 listen
        // 如果返回 Promise，则处理它
        const eventHandler = async (event) => {
            console.log('=== 收到 Gotify 消息事件 ===');
            console.log('完整事件对象:', event);
            const message = event.payload;
            console.log('消息 payload:', message);
            console.log('消息类型:', typeof message);
            if (message && typeof message === 'object') {
                console.log('消息键:', Object.keys(message));
            }
            
            // 获取超时时间
            const timeout = parseInt(document.getElementById('timeout').value) || 5;
            
            // 提取消息字段（增强容错性）
            const title = extractMessageField(message, ['title', 'Title', 'subject', 'Subject'], '通知');
            const msg = extractMessageField(message, ['message', 'Message', 'msg', 'content', 'Content', 'body', 'Body'], '');
            const priority = extractMessageField(message, ['priority', 'Priority', 'level', 'Level'], 0);
            
            function extractMessageField(obj, possibleKeys, defaultValue) {
                if (!obj || typeof obj !== 'object') {
                    return defaultValue;
                }
                
                for (const key of possibleKeys) {
                    if (obj.hasOwnProperty(key) && obj[key] !== null && obj[key] !== undefined) {
                        const value = obj[key];
                        // 如果是字符串且不为空，返回原值
                        if (typeof value === 'string') {
                            return value.trim() === '' ? defaultValue : value;
                        }
                        // 如果是数字，返回原值
                        if (typeof value === 'number') {
                            return value;
                        }
                        // 其他类型转换为字符串
                        return String(value);
                    }
                }
                return defaultValue;
            }
            
            console.log('提取的字段 - title:', title, 'message:', msg, 'priority:', priority);
            
            // 创建通知窗口（需要 await）
            try {
                await invoke('create_notification_window', {
                    title: title,
                    message: msg,
                    priority: priority,
                    timeoutSeconds: timeout
                });
                console.log('✓ 通知窗口创建成功');
            } catch (error) {
                console.error('✗ 创建通知窗口失败:', error);
                console.error('错误详情:', error.toString());
            }
        };
        
        console.log('调用 listen 函数，参数:', {
            eventName: 'gotify-message',
            handlerType: typeof eventHandler,
            listenType: typeof listen
        });
        
        // 尝试调用 listen，捕获权限错误
        let listenResult;
        try {
            listenResult = listen('gotify-message', eventHandler);
            console.log('listen 调用成功，返回值类型:', typeof listenResult);
        } catch (syncError) {
            console.error('✗ listen 同步调用失败:', syncError);
            throw syncError;
        }
        
        // 如果返回 Promise，等待它完成
        if (listenResult && typeof listenResult.then === 'function') {
            console.log('等待 Promise 完成...');
            try {
                await listenResult;
                console.log('✓ Gotify 消息监听器已设置（Promise 完成）');
                return true;
            } catch (promiseError) {
                console.error('✗ Promise 被拒绝:', promiseError);
                console.error('错误详情:', promiseError.toString());
                // 检查是否是权限错误
                if (promiseError.toString().includes('not allowed') || 
                    promiseError.toString().includes('Permissions')) {
                    console.error('⚠ 这是权限错误！请确保：');
                    console.error('1. capabilities.json 中包含 core:event:allow-listen');
                    console.error('2. 窗口 label 在 capabilities.json 的 windows 数组中');
                    console.error('3. 应用已完全重启（不只是重新构建）');
                    console.error('4. 检查 tauri.conf.json 中的窗口 label 是否匹配');
                }
                throw promiseError;
            }
        } else {
            console.log('✓ Gotify 消息监听器已设置（同步调用）');
            return true;
        }
    } catch (error) {
        console.error('✗ 设置消息监听失败:', error);
        console.error('错误详情:', error.toString());
        console.error('错误堆栈:', error.stack);
        // 显示用户友好的错误信息
        showStatus('设置消息监听失败: ' + error.message, 'error');
        return false;
    }
}

// 测试事件监听是否工作
async function testMessageListener() {
    if (!invoke) {
        console.error('Tauri API 未初始化');
        return;
    }
    
    console.log('=== 测试消息监听 ===');
    try {
        // 模拟发送一个测试消息
        await invoke('create_notification_window', {
            title: '测试消息',
            message: '如果您看到这个通知，说明通知窗口功能正常',
            priority: 5,
            timeoutSeconds: 3
        });
        console.log('✓ 测试通知窗口已创建');
    } catch (error) {
        console.error('✗ 测试失败:', error);
    }
}

// 初始化
document.addEventListener('DOMContentLoaded', async () => {
    // 等待 Tauri API 加载
    const tauriReady = await initTauri();
    
    if (tauriReady) {
        // 设置消息监听
        await setupMessageListener();
        
        // 加载配置
        await loadConfig();
        
        // 绑定事件
        document.getElementById('configForm').addEventListener('submit', (e) => {
            e.preventDefault();
            saveConfig();
        });
        
        document.getElementById('startBtn').addEventListener('click', startConnection);
        document.getElementById('stopBtn').addEventListener('click', stopConnection);
        document.getElementById('testBtn').addEventListener('click', testMessageListener);
        
        // 将测试函数暴露到全局，方便在控制台调用
        window.testMessageListener = testMessageListener;
        window.setupMessageListener = setupMessageListener;
        console.log('✓ 测试函数已暴露到全局: testMessageListener(), setupMessageListener()');
    } else {
        // 如果不在 Tauri 环境中，显示错误信息
        const statusEl = document.getElementById('status');
        statusEl.textContent = '错误：请在 Tauri 应用中运行';
        statusEl.className = 'status error';
        statusEl.style.display = 'block';
    }
});

