export default {
  app: {
    title: 'HTTP Mocks', targets: 'Targets', rules: 'Rules', templates: 'Response templates', logs: 'Request logs', dashboard: 'Overview',
    enabled: 'Enabled', disabled: 'Disabled', save: 'Save', cancel: 'Cancel', clear: 'Clear', edit: 'Edit', create: 'Create', refresh: 'Refresh', noData: 'No data', error: 'Request failed', delete: 'Delete', deleted: 'Deleted', deleteConfirm: 'Confirm deletion', deleteTargetConfirm: 'Delete target "{name}"? Rules referencing it must be removed first.', deleteRuleConfirm: 'Delete rule "{matcher}"?', deleteTemplateConfirm: 'Delete template "{name}"? Rules referencing it must be removed first.', deleteLogConfirm: 'Delete this log entry?', purge: 'Purge', purgeConfirm: 'Purge logs', purgeConfirmDesc: 'All logs matching the current filters will be permanently deleted. Continue?', purged: 'Purged {count} log(s)',
    fields: {
      name: 'Name', url: 'URL', timeout: 'Timeout (ms)', matcher: 'Matcher JSON', action: 'Action', method: 'Method', path: 'Path', priority: 'Priority', delay: 'Delay (ms)', target: 'Target', responseTemplate: 'Response template', note: 'Note', contentType: 'Content type', responseBody: 'Response body', responseHeaders: 'Response headers', format: 'Format', statusCode: 'Status code', status: 'Status', id: 'ID', time: 'Time', request: 'Request', bodyFormat: 'Body format', latency: 'Latency', errorMessage: 'Error',
    },
    actions: { all: 'All actions', proxy: 'Proxy', static: 'Static response' },
    testRule: 'Test rule', runTest: 'Run test', testNoMatch: 'No rule matched this request', testWillProxy: 'Would proxy to "{name}"',
    import: 'Import', export: 'Export', imported: 'Imported {targets} targets, {templates} templates, {rules} rules', warning: 'Warning',
    sequenceMode: 'Sequence mode', sequenceModeHint: 'Return each step in turn, then repeat', sequenceSteps: 'Response steps', addStep: 'Add step',
    proxyTransformHint: 'The upstream response will be rewritten with this template ({{ resp.* }} variables available)',
  },
}
