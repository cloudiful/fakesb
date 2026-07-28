/* Generated from the Rust utoipa document. Refresh with `bun run api:generate`. */
export interface components {
  schemas: {
    IdResponse: { id: number }
    LogDetail: components['schemas']['RequestLog'] & {
      snapshots: components['schemas']['MessageSnapshot'][]
    }
    LogPage: {
      items: components['schemas']['RequestLog'][]
      total: number
    }
    MessageSnapshot: {
      id: number
      kind: 'request' | 'response'
      raw_body: string
      normalized_json: unknown
    }
    RequestLog: {
      id: number
      occurred_at: string
      rule_id?: number | null
      target_id?: number | null
      mode?: components['schemas']['RuleMode'] | null
      service_code: string
      message_type: string
      message_code: string
      http_status_code?: string | null
      ret_code?: string | null
      ret_msg?: string | null
      latency_ms?: number | null
      error_message?: string | null
    }
    ResponseTemplate: {
      id: number
      name: string
      content_type: string
      raw_template: string
      format: string
      enabled: boolean
      note?: string | null
      created_at: string
      updated_at: string
    }
    Rule: {
      id: number
      service_code: string
      message_type: string
      message_code: string
      target_id?: number | null
      mode: components['schemas']['RuleMode']
      response_template_id?: number | null
      priority: number
      enabled: boolean
      note?: string | null
      created_at: string
      updated_at: string
    }
    RuleMode: 'passthrough' | 'mock'
    RulePage: {
      items: components['schemas']['Rule'][]
      total: number
    }
    RulePayload: {
      service_code: string
      message_type: string
      message_code: string
      target_id?: number | null
      mode: components['schemas']['RuleMode']
      response_template_id?: number | null
      priority?: number
      enabled?: boolean
      note?: string | null
    }
    Target: {
      id: number
      name: string
      base_url: string
      enabled: boolean
      timeout_ms: number
      note?: string | null
      created_at: string
      updated_at: string
    }
    TargetPage: {
      items: components['schemas']['Target'][]
      total: number
    }
    TargetPayload: {
      name: string
      base_url: string
      enabled?: boolean
      timeout_ms?: number
      note?: string | null
    }
    TemplatePage: {
      items: components['schemas']['ResponseTemplate'][]
      total: number
    }
    TemplatePayload: {
      name: string
      content_type?: string
      raw_template: string
      format?: string
      enabled?: boolean
      note?: string | null
    }
  }
}

export interface operations {
  listTargets: {
    parameters?: { query?: { offset?: number; limit?: number } }
    responses: { 200: { content: { 'application/json': components['schemas']['TargetPage'] } } }
  }
  createTarget: {
    requestBody: { content: { 'application/json': components['schemas']['TargetPayload'] } }
    responses: { 201: { content: { 'application/json': components['schemas']['IdResponse'] } } }
  }
  updateTarget: {
    parameters: { path: { id: number } }
    requestBody: { content: { 'application/json': components['schemas']['TargetPayload'] } }
    responses: { 200: { content: { 'application/json': components['schemas']['IdResponse'] } } }
  }
  listRules: {
    parameters?: { query?: { offset?: number; limit?: number } }
    responses: { 200: { content: { 'application/json': components['schemas']['RulePage'] } } }
  }
  createRule: {
    requestBody: { content: { 'application/json': components['schemas']['RulePayload'] } }
    responses: { 201: { content: { 'application/json': components['schemas']['IdResponse'] } } }
  }
  updateRule: {
    parameters: { path: { id: number } }
    requestBody: { content: { 'application/json': components['schemas']['RulePayload'] } }
    responses: { 200: { content: { 'application/json': components['schemas']['IdResponse'] } } }
  }
  listTemplates: {
    parameters?: { query?: { offset?: number; limit?: number } }
    responses: { 200: { content: { 'application/json': components['schemas']['TemplatePage'] } } }
  }
  createTemplate: {
    requestBody: { content: { 'application/json': components['schemas']['TemplatePayload'] } }
    responses: { 201: { content: { 'application/json': components['schemas']['IdResponse'] } } }
  }
  updateTemplate: {
    parameters: { path: { id: number } }
    requestBody: { content: { 'application/json': components['schemas']['TemplatePayload'] } }
    responses: { 200: { content: { 'application/json': components['schemas']['IdResponse'] } } }
  }
  listLogs: {
    parameters?: {
      query?: {
        offset?: number
        limit?: number
        service_code?: string
        message_type?: string
        message_code?: string
        mode?: components['schemas']['RuleMode']
        ret_code?: string
        start_time?: string
        end_time?: string
      }
    }
    responses: { 200: { content: { 'application/json': components['schemas']['LogPage'] } } }
  }
  getLog: {
    parameters: { path: { id: number } }
    responses: { 200: { content: { 'application/json': components['schemas']['LogDetail'] } } }
  }
}

export interface paths {
  '/api/targets': { get: operations['listTargets']; post: operations['createTarget'] }
  '/api/targets/{id}': { put: operations['updateTarget'] }
  '/api/rules': { get: operations['listRules']; post: operations['createRule'] }
  '/api/rules/{id}': { put: operations['updateRule'] }
  '/api/templates': { get: operations['listTemplates']; post: operations['createTemplate'] }
  '/api/templates/{id}': { put: operations['updateTemplate'] }
  '/api/logs': { get: operations['listLogs'] }
  '/api/logs/{id}': { get: operations['getLog'] }
}
