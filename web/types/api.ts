import type { components } from './api.generated'

export type RuleAction = components['schemas']['RuleAction']
export type RuleMatcher = components['schemas']['RuleMatcher']
export type Page<T> = { items: T[]; total: number }
export type Target = components['schemas']['Target']
export type Rule = components['schemas']['Rule']
export type ResponseTemplate = components['schemas']['ResponseTemplate']
export type RequestLog = components['schemas']['RequestLog']
export type MessageSnapshot = components['schemas']['MessageSnapshot']
export type LogDetail = components['schemas']['LogDetail']
export type TargetPayload = components['schemas']['TargetPayload']
export type RulePayload = components['schemas']['RulePayload']
export type TemplatePayload = components['schemas']['TemplatePayload']
export type RuleTestResult = components['schemas']['RuleTestResult']
export type RenderedPreview = components['schemas']['RenderedPreview']
export type SequenceStep = components['schemas']['SequenceStep']
export type SequenceStepPayload = components['schemas']['SequenceStepPayload']
