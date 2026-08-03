import createClient from 'openapi-fetch'
import type { operations, paths } from '~/types/api.generated'
import { normalizeApiBase } from '~/utils/api-base'

type TargetQuery = NonNullable<operations['listTargets']['parameters']>['query']
type RuleQuery = NonNullable<operations['listRules']['parameters']>['query']
type TemplateQuery = NonNullable<operations['listTemplates']['parameters']>['query']
type LogQuery = NonNullable<operations['listLogs']['parameters']>['query']

export function useApi() {
  const config = useRuntimeConfig()
  const client = createClient<paths>({
    baseUrl: normalizeApiBase(String(config.public.apiBase || '/')),
  })

  async function unwrap<T>(result: Promise<{ data?: T; error?: unknown }>) {
    const response = await result
    if (response.error) {
      throw new Error('API request failed')
    }
    if (response.data === undefined) {
      throw new Error('API response was empty')
    }
    return response.data
  }

  return {
    listTargets: (params?: TargetQuery) => unwrap(client.GET('/api/targets', { params: { query: params } })),
    createTarget: (body: operations['createTarget']['requestBody']['content']['application/json']) =>
      unwrap(client.POST('/api/targets', { body })),
    updateTarget: (id: number, body: operations['updateTarget']['requestBody']['content']['application/json']) =>
      unwrap(client.PUT('/api/targets/{id}', { params: { path: { id } }, body })),
    deleteTarget: (id: number) => unwrap(client.DELETE('/api/targets/{id}', { params: { path: { id } } })),
    listRules: (params?: RuleQuery) => unwrap(client.GET('/api/rules', { params: { query: params } })),
    createRule: (body: operations['createRule']['requestBody']['content']['application/json']) =>
      unwrap(client.POST('/api/rules', { body })),
    testRule: (body: operations['testRule']['requestBody']['content']['application/json']) =>
      unwrap(client.POST('/api/rules/test', { body })),
    updateRule: (id: number, body: operations['updateRule']['requestBody']['content']['application/json']) =>
      unwrap(client.PUT('/api/rules/{id}', { params: { path: { id } }, body })),
    deleteRule: (id: number) => unwrap(client.DELETE('/api/rules/{id}', { params: { path: { id } } })),
    listTemplates: (params?: TemplateQuery) => unwrap(client.GET('/api/templates', { params: { query: params } })),
    createTemplate: (body: operations['createTemplate']['requestBody']['content']['application/json']) =>
      unwrap(client.POST('/api/templates', { body })),
    updateTemplate: (id: number, body: operations['updateTemplate']['requestBody']['content']['application/json']) =>
      unwrap(client.PUT('/api/templates/{id}', { params: { path: { id } }, body })),
    deleteTemplate: (id: number) => unwrap(client.DELETE('/api/templates/{id}', { params: { path: { id } } })),
    listLogs: (params?: LogQuery) => unwrap(client.GET('/api/logs', { params: { query: params } })),
    getLog: (id: number) => unwrap(client.GET('/api/logs/{id}', { params: { path: { id } } })),
    deleteLog: (id: number) => unwrap(client.DELETE('/api/logs/{id}', { params: { path: { id } } })),
    purgeLogs: (params?: LogQuery) => unwrap(client.DELETE('/api/logs', { params: { query: params } })),
    exportConfig: () => unwrap(client.GET('/api/export')),
    importConfig: (body: operations['importConfig']['requestBody']['content']['application/json']) =>
      unwrap(client.POST('/api/import', { body })),
  }
}
