export default {
  app: {
    title: 'HTTP Mock', targets: '目标地址', rules: '匹配规则', templates: '响应模板', logs: '请求日志', dashboard: '概览',
    enabled: '启用', disabled: '停用', save: '保存', cancel: '取消', clear: '清除', edit: '编辑', create: '新建', refresh: '刷新', noData: '暂无数据', error: '请求失败',
    fields: {
      name: '名称', url: '地址', timeout: '超时（毫秒）', matcher: '匹配器 JSON', action: '动作', method: '方法', path: '路径', priority: '优先级', target: '目标地址', responseTemplate: '响应模板', note: '备注', contentType: '内容类型', responseBody: '响应内容', responseHeaders: '响应头', format: '格式', statusCode: '状态码', status: '状态', id: '编号', time: '时间', request: '请求', bodyFormat: '正文格式', latency: '耗时', errorMessage: '错误信息',
    },
    actions: { all: '全部动作', proxy: '代理', static: '静态响应' },
  },
}
