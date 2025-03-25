new Vue({
    el: '#app',
    data() {
        return {
            // 连接状态
            socket: null,
            connected: false,
            
            // 调试状态
            isDebugging: false,
            selectedPlugin: '',
            plugins: [],
            
            // 消息设置
            userId: '10000',
            groupId: '10000',
            messageType: 'group',
            
            // 消息内容
            inputMessage: '',
            messages: [],
            
            // 历史记录
            history: {},
            activeHistoryPlugin: [],
            
            // 文件上传
            fileToUpload: null,
            fileType: null,
            
            // UI状态
            imagePreviewVisible: false,
            imagePreviewUrl: '',
            notificationVisible: false,
            notificationMessage: ''
        };
    },
    created() {
        this.initSocket();
        this.loadPlugins();
        this.loadHistory();
    },
    mounted() {
        // 检查URL参数是否有插件名称
        const urlParams = new URLSearchParams(window.location.search);
        const pluginParam = urlParams.get('plugin');
        if (pluginParam) {
            this.selectedPlugin = pluginParam;
            this.$nextTick(() => {
                this.startDebug();
            });
        }
    },
    // 在Vue实例的methods中添加
    methods: {
        // 初始化WebSocket连接
        initSocket() {
            this.socket = io();
            
            this.socket.on('connect', () => {
                this.connected = true;
                console.log('WebSocket连接成功');
            });
            
            this.socket.on('disconnect', () => {
                this.connected = false;
                console.log('WebSocket连接断开');
            });
            
            this.socket.on('status', (data) => {
                this.isDebugging = data.active;
                if (data.plugin) {
                    this.selectedPlugin = data.plugin;
                }
            });
            
            this.socket.on('message_update', (message) => {
                this.messages.push(message);
                this.$nextTick(() => {
                    this.scrollToBottom();
                });
            });
            
            this.socket.on('error', (data) => {
                this.showNotification(data.message);
            });
            
            this.socket.on('upload_success', (data) => {
                this.sendFileMessage(data.path);
            });
        },
        
        // 加载可用插件列表
        loadPlugins() {
            fetch('/api/plugins')
                .then(response => {
                    if (!response.ok) {
                        throw new Error(`HTTP错误 ${response.status}`);
                    }
                    return response.json();
                })
                .then(data => {
                    if (data.error) {
                        throw new Error(data.error);
                    }
                    console.log("加载的插件列表:", data);
                    this.plugins = data;
                })
                .catch(error => {
                    console.error('加载插件列表失败:', error);
                    this.showNotification('加载插件列表失败: ' + error.message);
                });
        },
        
        // 加载历史记录
        loadHistory() {
            fetch('/api/history')
                .then(response => response.json())
                .then(data => {
                    this.history = data;
                })
                .catch(error => {
                    console.error('加载历史记录失败:', error);
                });
        },
        
        // 加载特定历史会话
        // 在 Vue 实例的 data 对象中添加
        data: {
            // ... 现有属性 ...
            currentHistorySession: null,
            currentHistoryPlugin: null
        },
        
        // 并修改 loadHistorySession 方法
        loadHistorySession(filename) {
            fetch(`/api/history/${filename}`)
                .then(response => response.json())
                .then(data => {
                    this.messages = data;
                    this.currentHistorySession = filename;
                    
                    // 找出当前会话属于哪个插件
                    for (const plugin in this.history) {
                        const found = this.history[plugin].find(session => session.filename === filename);
                        if (found) {
                            this.currentHistoryPlugin = plugin;
                            break;
                        }
                    }
                    
                    this.$nextTick(() => {
                        this.scrollToBottom();
                    });
                })
                .catch(error => {
                    console.error('加载历史会话失败:', error);
                    this.$message.error('加载历史会话失败');
                });
        },
        
        // 开始调试
        startDebug() {
            if (!this.selectedPlugin) {
                this.showNotification('请选择要调试的插件');
                return;
            }
            
            fetch('/api/start_debug', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    plugin_name: this.selectedPlugin
                })
            })
            .then(response => response.json())
            .then(data => {
                if (data.success) {
                    this.isDebugging = true;
                    this.messages = [];
                    this.showNotification(data.message);
                } else {
                    this.showNotification(data.message);
                }
            })
            .catch(error => {
                console.error('启动调试失败:', error);
                this.showNotification('启动调试失败');
            });
        },
        
        // 停止调试
        stopDebug() {
            fetch('/api/stop_debug', {
                method: 'POST'
            })
            .then(response => response.json())
            .then(data => {
                if (data.success) {
                    this.isDebugging = false;
                    this.showNotification(data.message);
                    this.loadHistory(); // 重新加载历史记录
                } else {
                    this.showNotification(data.message);
                }
            })
            .catch(error => {
                console.error('停止调试失败:', error);
                this.showNotification('停止调试失败');
            });
        },
        
        // 发送消息
        enterSendMessage(event) {
            // 如果按下Shift+Enter，不处理（允许换行）
            if (event.shiftKey) {
                return;
            }
            
            // 阻止默认行为（添加换行符）
            event.preventDefault();
            
            // 清理消息内容，去除可能的换行符
            const cleanMessage = this.inputMessage.trim();
            
            // 如果消息不为空且正在调试中，则发送消息
            if (cleanMessage && this.isDebugging) {
                const messageData = {
                    content: cleanMessage,
                    user_id: parseInt(this.userId),
                    group_id: parseInt(this.groupId),
                    message_type: this.messageType,
                    type: 'text'
                };
                
                this.socket.emit('send_message', messageData);
                this.inputMessage = '';
            }
        },

        // 发送消息
        sendMessage() {
            if (!this.inputMessage.trim() || !this.isDebugging) return;
            
            const messageData = {
                content: this.inputMessage,
                user_id: parseInt(this.userId),
                group_id: parseInt(this.groupId),
                message_type: this.messageType,
                type: 'text'
            };
            
            this.socket.emit('send_message', messageData);
            this.inputMessage = '';
        },
        
        // 发送文件消息
        sendFileMessage(filePath) {
            if (!this.isDebugging) return;
            
            const messageData = {
                content: filePath,
                user_id: parseInt(this.userId),
                group_id: parseInt(this.groupId),
                message_type: this.messageType,
                type: this.fileType
            };
            
            this.socket.emit('send_message', messageData);
            this.fileToUpload = null;
            this.fileType = null;
        },
        
        // 处理图片文件选择
        handleFileChange(file) {
            if (!file) return;
            this.fileToUpload = file.raw;
            this.fileType = 'image';
            this.uploadFile();
        },
        
        // 处理音频文件选择
        handleAudioChange(file) {
            if (!file) return;
            this.fileToUpload = file.raw;
            this.fileType = 'audio';
            this.uploadFile();
        },
        
        // 处理视频文件选择
        handleVideoChange(file) {
            if (!file) return;
            this.fileToUpload = file.raw;
            this.fileType = 'video';
            this.uploadFile();
        },
        
        // 上传文件
        uploadFile() {
            if (!this.fileToUpload || !this.fileType) return;
            
            const reader = new FileReader();
            reader.readAsArrayBuffer(this.fileToUpload);
            reader.onload = () => {
                const arrayBuffer = reader.result;
                this.socket.emit('upload_file', {
                    file: arrayBuffer,
                    type: this.fileType,
                    filename: this.fileToUpload.name
                });
            };
        },
        
        // 预览图片
        previewImage(imagePath) {
            // 使用完整路径或文件名
            const filename = imagePath.includes('/') ? imagePath.split('/').pop() : imagePath;
            this.imagePreviewUrl = '/uploads/' + filename;
            
            // 预加载图片
            const img = new Image();
            img.onload = () => {
                this.imagePreviewVisible = true;
            };
            img.onerror = () => {
                this.$message.error('图片加载失败');
            };
            img.src = this.imagePreviewUrl;
        },
        
        // 格式化消息内容
        formatMessage(content, type) {
            // 如果是图片类型，优化图片HTML渲染
            if (type === 'image') {
                return `<div class="message-image">
                    <img src="/uploads/${content}" alt="图片" 
                         loading="lazy" 
                         onclick="app.previewImage('${content}')"
                         onerror="this.onerror=null;this.src='/static/img/image-error.png';">
                </div>`;
            }
            
            // 使用marked库将markdown格式转换为HTML
            try {
                // 配置marked以正确处理换行符
                marked.setOptions({
                    breaks: true,  // 将换行符转换为<br>
                    gfm: true      // 使用GitHub风格的Markdown
                });
                return marked.parse(content);
            } catch (e) {
                return content;
            }
        },
        
        // 格式化时间戳
        formatTimestamp(timestamp) {
            const date = new Date(timestamp.replace(/_/g, ':'));
            return date.toLocaleString();
        },
        
        // 滚动到底部
        scrollToBottom() {
            const container = this.$refs.messagesContainer;
            if (container) {
                // 使用 nextTick 确保 DOM 已更新
                this.$nextTick(() => {
                    container.scrollTop = container.scrollHeight;
                });
            }
        },
        
        // 显示通知
        showNotification(message, type = 'info') {
            // 创建通知元素
            const notification = document.createElement('div');
            notification.className = `notification notification-${type}`;
            
            // 根据类型设置图标
            let icon = '';
            let title = '';
            switch(type) {
                case 'success':
                    icon = '✓';
                    title = '成功';
                    break;
                case 'warning':
                    icon = '⚠';
                    title = '警告';
                    break;
                case 'error':
                    icon = '✗';
                    title = '错误';
                    break;
                case 'info':
                default:
                    icon = 'ℹ';
                    title = '信息';
                    break;
            }
            
            // 设置通知内容
            notification.innerHTML = `
                <div class="notification-icon">${icon}</div>
                <div class="notification-content">
                    <div class="notification-title">${title}</div>
                    <div class="notification-message">${message}</div>
                </div>
            `;
            
            // 添加到容器
            const container = document.getElementById('notification-container');
            container.appendChild(notification);
            
            // 触发动画
            setTimeout(() => {
                notification.classList.add('show');
            }, 10);
            
            // 3秒后自动移除
            setTimeout(() => {
                notification.classList.remove('show');
                
                // 等待过渡动画完成后移除元素
                setTimeout(() => {
                    container.removeChild(notification);
                }, 300);
            }, 3000);
        },

        // 删除单个历史记录
        deleteHistorySession(filename) {
            this.$confirm('确定要删除此历史记录及其相关图片吗？', '提示', {
                confirmButtonText: '确定',
                cancelButtonText: '取消',
                type: 'warning'
            }).then(() => {
                fetch(`/api/history/${filename}`, {
                    method: 'DELETE'
                })
                .then(response => response.json())
                .then(data => {
                    if (data.success) {
                        this.$message({
                            type: data.type || 'success',
                            message: data.message
                        });
                        this.loadHistory(); // 重新加载历史记录
                        // 如果当前正在查看的就是被删除的会话，则清空消息
                        if (this.currentHistorySession === filename) {
                            this.messages = [];
                            this.currentHistorySession = null;
                        }
                    } else {
                        this.$message({
                            type: data.type || 'error',
                            message: data.message
                        });
                    }
                })
                .catch(error => {
                    console.error('删除历史记录失败:', error);
                    this.$message.error('删除历史记录失败');
                });
            }).catch(() => {
                // 用户取消删除
            });
        },
        
        // 删除插件所有历史记录
        deletePluginHistory(pluginName) {
            this.$confirm(`确定要删除 ${pluginName} 的所有历史记录及相关图片吗？`, '提示', {
                confirmButtonText: '确定',
                cancelButtonText: '取消',
                type: 'warning'
            }).then(() => {
                fetch(`/api/history/plugin/${pluginName}`, {
                    method: 'DELETE'
                })
                .then(response => response.json())
                .then(data => {
                    if (data.success) {
                        this.$message({
                            type: data.type || 'success',
                            message: data.message
                        });
                        this.loadHistory(); // 重新加载历史记录
                        // 如果当前正在查看的是被删除插件的会话，则清空消息
                        if (this.currentHistoryPlugin === pluginName) {
                            this.messages = [];
                            this.currentHistorySession = null;
                        }
                    } else {
                        this.$message({
                            type: data.type || 'error',
                            message: data.message
                        });
                    }
                })
                .catch(error => {
                    console.error('删除插件历史记录失败:', error);
                    this.$message.error('删除插件历史记录失败');
                });
            }).catch(() => {
                // 用户取消删除
            });
        },
        
        // 删除所有历史记录
        deleteAllHistory() {
            this.$confirm('确定要删除所有历史记录及相关图片吗？', '提示', {
                confirmButtonText: '确定',
                cancelButtonText: '取消',
                type: 'warning'
            }).then(() => {
                fetch('/api/history', {
                    method: 'DELETE'
                })
                .then(response => response.json())
                .then(data => {
                    if (data.success) {
                        this.$message({
                            type: data.type || 'success',
                            message: data.message
                        });
                        this.loadHistory(); // 重新加载历史记录
                        this.messages = []; // 清空当前显示的消息
                        this.currentHistorySession = null;
                    } else {
                        this.$message({
                            type: data.type || 'error',
                            message: data.message
                        });
                    }
                })
                .catch(error => {
                    console.error('删除所有历史记录失败:', error);
                    this.$message.error('删除所有历史记录失败');
                });
            }).catch(() => {
                // 用户取消删除
            });
        }
    }
});
