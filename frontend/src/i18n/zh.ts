/** 中文（默认）翻译 */
const zh: Record<string, string> = {
  /* App shell */
  'app.name': 'OpsPilot',
  'app.tagline': 'AI 驱动的基础设施运维平台',
  'nav.light': '浅色',
  'nav.dark': '深色',
  'nav.logout': '退出',
  'nav.back': '返回',

  /* Sidebar categories */
  'cat.dashboard': '数据大屏',
  'cat.system': '系统管理',
  'cat.infrastructure': '基础设施',
  'cat.security': '安全合规',
  'cat.automation': '自动化',
  'cat.monitor': '监控告警',
  'cat.intelligence': '智能分析',
  'cat.integration': '集成管理',

  /* Tab labels (short) */
  'tab.dashboard': '大屏',
  'tab.chat': '对话',
  'tab.modules': '模块',
  'tab.hosts': '主机',
  'tab.vault': '保险库',
  'tab.security': '安全',
  'tab.health': '健康',
  'tab.topo': '拓扑',
  'tab.monitor': '监控',
  'tab.escalation': '告警',
  'tab.fim': 'FIM',
  'tab.baseline': '基线',
  'tab.runbook': '手册',
  'tab.knowledge': '知识库',
  'tab.config': '配置',
  'tab.webhook': '钩子',
  'tab.scheduler': '调度',
  'tab.filesync': '同步',
  'tab.advisor': '建议',
  'tab.terminal': '终端',

  /* Tab titles (page heading) */
  'title.dashboard': '数据大屏',
  'title.chat': 'AI 对话',
  'title.modules': '模块管理',
  'title.hosts': '主机管理',
  'title.vault': '凭据保险库',
  'title.security': '安全扫描',
  'title.health': '健康状态',
  'title.topo': '网络拓扑',
  'title.monitor': '性能监控',
  'title.escalation': '告警升级',
  'title.fim': '文件完整性',
  'title.baseline': '安全基线',
  'title.runbook': '运维手册',
  'title.knowledge': '知识库',
  'title.config': '系统配置',
  'title.webhook': 'Webhook',
  'title.scheduler': '任务调度',
  'title.filesync': '文件同步',
  'title.advisor': '智能建议',
  'title.terminal': 'Web 终端',

  /* Terminal page */
  'terminal.title': 'Web 终端',
  'terminal.status.connecting': '连接中...',
  'terminal.status.connected': '已连接',
  'terminal.status.disconnected': '已断开',
  'terminal.status.error': '连接错误',
  'terminal.connecting': '正在建立 SSH 连接...',
  'terminal.connection_error': 'WebSocket 连接失败',
  'terminal.reconnect': '重新连接',
  'terminal.disconnect': '断开连接',
  'terminal.ssh': 'WebSSH',
  'terminal.select_host_hint': '请从主机管理页面选择一台主机，然后点击「WebSSH」按钮连接终端。',
  'terminal.go_to_hosts': '前往主机管理',

  /* Desk */
  'desk': '桌面',
  'mobile': '移动',

  /* Modules page */
  'modules.title': '模块管理',
  'modules.reload': '刷新',
  'modules.name': '名称',
  'modules.version': '版本',
  'modules.description': '描述',
  'modules.health': '健康',
  'modules.enabled': '启用',
  'modules.healthy': '健康',
  'modules.degraded': '降级',
  'modules.unhealthy': '不健康',

  /* Hosts page */
  'hosts.title': '主机管理',
  'hosts.add': '添加主机',
  'hosts.reload': '刷新',
  'hosts.name': '名称',
  'hosts.address': '地址',
  'hosts.port': '端口',
  'hosts.status': '状态',
  'hosts.auth': '认证',
  'hosts.actions': '操作',
  'hosts.empty': '暂无已配置的主机',
  'hosts.vault_locked': '保险库已锁定。主机凭据加密存储。请前往「凭据保险库」解锁并管理凭据。',

  /* Vault page */
  'vault.title': '凭据保险库',
  'vault.desc': '保险库使用基于用户口令派生的密钥加密主机凭据。你的口令不会被存储——仅保留验证哈希。',
  'vault.set_passphrase': '设置保险库口令',
  'vault.login_password': '登录密码',
  'vault.new_passphrase': '新口令',
  'vault.confirm_passphrase': '确认口令',
  'vault.set_passphrase_btn': '设置口令',

  /* Security page */
  'security.title': '安全扫描',
  'security.desc': '对托管主机执行合规检查、漏洞扫描和补丁审计',
  'security.scan_config': '扫描配置',
  'security.check_type': '检查类型',
  'security.target_host': '目标主机',
  'security.all_hosts': '所有主机',
  'security.run_scan': '执行扫描',
  'security.profiles': '可用扫描规则',
  'security.medium': '中',
  'security.high': '高',
  'security.critical': '严重',
  'security.low': '低',

  /* Health page */
  'health.title': '健康状态',
  'health.refresh': '刷新',
  'health.module': '模块',
  'health.status': '状态',
  'health.enabled': '启用状态',

  /* Topo page */
  'topo.title': '网络拓扑',
  'topo.refresh': '刷新',
  'topo.discover': '发现拓扑',
  'topo.empty': '暂无拓扑数据——点击「发现拓扑」进行扫描',

  /* Monitor page */
  'monitor.title': '性能监控',
  'monitor.select_host': '选择主机',
  'monitor.collect': '采集指标',
  'monitor.hint': '选择主机并点击「采集指标」开始监控',

  /* Escalation page */
  'escalation.title': '告警升级',
  'escalation.define': '定义升级策略',
  'escalation.name': '策略名称',
  'escalation.severity': '严重级别',
  'escalation.delay': '延迟（分钟）',
  'escalation.channels': '通知渠道',
  'escalation.save': '保存策略',
  'escalation.trigger': '触发告警',
  'escalation.alert_id': '告警 ID',
  'escalation.message': '消息',

  /* FIM page */
  'fim.title': '文件完整性',
  'fim.select_host': '选择主机',
  'fim.create_baseline': '创建基线',
  'fim.run_scan': '执行扫描',
  'fim.hint': '选择主机，创建基线，然后执行扫描以检测变更',

  /* Baseline page */
  'baseline.title': '安全基线',
  'baseline.select_host': '选择主机',
  'baseline.run_check': '执行检查',
  'baseline.hint': '选择主机并点击「执行检查」进行安全基线审计',

  /* Runbook page */
  'runbook.title': '运维手册',
  'runbook.create': '创建手册',
  'runbook.name': '名称',
  'runbook.steps': '步骤（每行一步）',
  'runbook.create_btn': '创建手册',
  'runbook.execute': '执行手册',
  'runbook.runbook_name': '手册名称',
  'runbook.target_host': '目标主机（可选）',
  'runbook.execute_btn': '执行',

  /* Knowledge page */
  'knowledge.title': '知识库',
  'knowledge.search': '搜索知识',
  'knowledge.search_placeholder': '搜索关键词...',
  'knowledge.search_btn': '搜索',
  'knowledge.extract': '从事件提取知识',
  'knowledge.extract_btn': '提取',

  /* Config page */
  'config.title': '系统配置',
  'config.refresh': '刷新',
  'config.key': '键',
  'config.value': '值',
  'config.add': '添加/更新配置',
  'config.key_placeholder': '配置键（如 ssh.host1）',
  'config.value_placeholder': '配置值（如 "10.0.0.1" 或 {"port": 22}）',
  'config.save': '保存',
  'config.empty': '暂无配置项',

  /* Webhook page */
  'webhook.title': 'Webhook',
  'webhook.register': '注册 Webhook',
  'webhook.name': '名称（如 slack-alerts）',
  'webhook.url': 'URL',
  'webhook.secret': '密钥（可选）',
  'webhook.register_btn': '注册',
  'webhook.name_col': '名称',
  'webhook.url_col': 'URL',
  'webhook.secret_col': '密钥',
  'webhook.retries': '重试次数',
  'webhook.empty': '暂无已配置的 Webhook',

  /* Scheduler page */
  'scheduler.title': '任务调度',
  'scheduler.create': '创建定时任务',
  'scheduler.name': '任务名称',
  'scheduler.cron': 'Cron 表达式（如 */5 * * * *）',
  'scheduler.action': '操作',
  'scheduler.create_btn': '创建',
  'scheduler.name_col': '名称',
  'scheduler.cron_col': 'Cron',
  'scheduler.action_col': '操作',
  'scheduler.status': '状态',
  'scheduler.last_run': '上次执行',
  'scheduler.next_run': '下次执行',
  'scheduler.empty': '暂无定时任务',

  /* FileSync page */
  'filesync.title': '文件同步',
  'filesync.push': '推送文件到主机',
  'filesync.target_host': '目标主机',
  'filesync.select_host': '选择主机',
  'filesync.remote_path': '远程文件路径',
  'filesync.content': '文件内容',
  'filesync.push_btn': '推送文件',

  /* Advisor page */
  'advisor.title': '智能建议',
  'advisor.refresh': '刷新',
  'advisor.empty': '暂无私建议',

  /* Chat page */
  'chat.title': 'AI 对话',
  'chat.placeholder': '输入消息...',
  'chat.send': '发送',
  'chat.start': '与 OpsPilot 智能体开始对话。',

  /* Login page */
  'login.username': '用户名',
  'login.password': '密码',
  'login.signin': '登录',
  'login.no_account': '没有账号？',
  'login.create': '注册',
  'login.create_title': '创建账号',
  'login.email': '邮箱',
  'login.create_account': '创建账号',
  'login.has_account': '已有账号？',
  'login.sign_in': '登录',

  /* Theme picker */
  'theme.title': '主题',
  'theme.magenta': '洋红',
  'theme.blue': '蓝色',
  'theme.green': '绿色',
  'theme.orange': '橙色',
  'theme.purple': '紫色',
  'theme.teal': '青绿',
  'theme.rose': '玫瑰',
  'theme.neutral': '中性',

  /* Language */
  'lang.zh': '中文',
  'lang.en': 'English',
};

export default zh;
