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
    listRules: (params?: RuleQuery) => unwrap(client.GET('/api/rules', { params: { query: params } })),
    createRule: (body: operations['createRule']['requestBody']['content']['application/json']) =>
      unwrap(client.POST('/api/rules', { body })),
    updateRule: (id: number, body: operations['updateRule']['requestBody']['content']['application/json']) =>
      unwrap(client.PUT('/api/rules/{id}', { params: { path: { id } }, body })),
    listTemplates: (params?: TemplateQuery) => unwrap(client.GET('/api/templates', { params: { query: params } })),
    createTemplate: (body: operations['createTemplate']['requestBody']['content']['application/json']) =>
      unwrap(client.POST('/api/templates', { body })),
    updateTemplate: (id: number, body: operations['updateTemplate']['requestBody']['content']['application/json']) =>
      unwrap(client.PUT('/api/templates/{id}', { params: { path: { id } }, body })),
    listLogs: (params?: LogQuery) => unwrap(client.GET('/api/logs', { params: { query: params } })),
    getLog: (id: number) => unwrap(client.GET('/api/logs/{id}', { params: { path: { id } } })),
  }
}
