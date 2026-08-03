export default {
  app: {
    title: 'HTTP Mock', targets: '目标地址', rules: '匹配规则', templates: '响应模板', logs: '请求日志', dashboard: '概览',
    enabled: '启用', disabled: '停用', save: '保存', cancel: '取消', clear: '清除', edit: '编辑', create: '新建', refresh: '刷新', noData: '暂无数据', error: '请求失败', delete: '删除', deleted: '已删除', deleteConfirm: '确认删除', deleteTargetConfirm: '删除目标地址「{name}」？被其引用的规则需先删除。', deleteRuleConfirm: '删除规则「{matcher}」？', deleteTemplateConfirm: '删除响应模板「{name}」？被其引用的规则需先删除。', deleteLogConfirm: '删除这条日志？', purge: '清理', purgeConfirm: '清理日志', purgeConfirmDesc: '将永久删除符合当前筛选条件的所有日志，是否继续？', purged: '已清理 {count} 条日志',
    fields: {
      name: '名称', url: '地址', timeout: '超时（毫秒）', matcher: '匹配器 JSON', action: '动作', method: '方法', path: '路径', priority: '优先级', delay: '延迟（毫秒）', target: '目标地址', responseTemplate: '响应模板', note: '备注', contentType: '内容类型', responseBody: '响应内容', responseHeaders: '响应头', format: '格式', statusCode: '状态码', status: '状态', id: '编号', time: '时间', request: '请求', bodyFormat: '正文格式', latency: '耗时', errorMessage: '错误信息',
    },
    actions: { all: '全部动作', proxy: '代理', static: '静态响应' },
    testRule: '规则测试', runTest: '执行测试', testNoMatch: '没有规则命中该请求', testWillProxy: '将代理到「{name}」',
    import: '导入', export: '导出', imported: '已导入 {targets} 个目标、{templates} 个模板、{rules} 条规则', warning: '提示',
    sequenceMode: '序列模式', sequenceModeHint: '按步骤依次返回，循环往复', sequenceSteps: '响应步骤', addStep: '添加步骤',
    proxyTransformHint: '将用该模板改写上游响应（可使用 \\{\\{ resp.* \\}\\} 变量）',
  },
}
