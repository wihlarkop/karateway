// API Response wrapper
export interface JsonResponse<T> {
  data?: T
  message?: string
  success: boolean
  meta?: MetaResponse
  status_code: number
  timestamp: string
  error_code?: string
}

export interface MetaResponse {
  page: number
  limit: number
  total_data: number
  total_pages: number
}

// Backend Service
export interface BackendService {
  id: string
  name: string
  description?: string
  base_url: string
  health_check_url?: string
  health_check_interval_seconds?: number
  timeout_ms?: number
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface CreateBackendServiceRequest {
  name: string
  description?: string
  base_url: string
  health_check_url?: string
  health_check_interval_seconds?: number
  timeout_ms?: number
}

export interface UpdateBackendServiceRequest {
  name?: string
  description?: string
  base_url?: string
  health_check_url?: string
  health_check_interval_seconds?: number
  timeout_ms?: number
  is_active?: boolean
}

// API Route
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'HEAD' | 'OPTIONS'

export interface ApiRoute {
  id: string
  path_pattern: string
  method: HttpMethod
  backend_service_id: string
  strip_path_prefix: boolean
  preserve_host_header: boolean
  timeout_ms?: number
  is_active: boolean
  priority: number
  metadata: Record<string, any>
  created_at: string
  updated_at: string
}

export interface CreateApiRouteRequest {
  path_pattern: string
  method: HttpMethod
  backend_service_id: string
  strip_path_prefix?: boolean
  preserve_host_header?: boolean
  timeout_ms?: number
  priority?: number
  metadata?: Record<string, any>
}

export interface UpdateApiRouteRequest {
  path_pattern?: string
  method?: HttpMethod
  backend_service_id?: string
  strip_path_prefix?: boolean
  preserve_host_header?: boolean
  timeout_ms?: number
  is_active?: boolean
  priority?: number
  metadata?: Record<string, any>
}

// Rate Limit
export type IdentifierType = 'Ip' | 'ApiKey' | 'UserId' | 'Global'

export interface RateLimit {
  id: string
  name: string
  api_route_id?: string
  identifier_type: IdentifierType
  max_requests: number
  window_seconds: number
  burst_size?: number
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface CreateRateLimitRequest {
  name: string
  api_route_id?: string
  identifier_type: IdentifierType
  max_requests: number
  window_seconds: number
  burst_size?: number
}

export interface UpdateRateLimitRequest {
  name?: string
  api_route_id?: string
  identifier_type?: IdentifierType
  max_requests?: number
  window_seconds?: number
  burst_size?: number
  is_active?: boolean
}

// Whitelist Rule
export type RuleType = 'Ip' | 'ApiKey' | 'Jwt' | 'Custom'

export interface WhitelistRule {
  id: string
  rule_name: string
  rule_type: RuleType
  api_route_id?: string
  config: Record<string, any>
  is_active: boolean
  priority: number
  created_at: string
  updated_at: string
}

export interface CreateWhitelistRuleRequest {
  rule_name: string
  rule_type: RuleType
  api_route_id?: string
  config: Record<string, any>
  priority?: number
}

export interface UpdateWhitelistRuleRequest {
  rule_name?: string
  rule_type?: RuleType
  api_route_id?: string
  config?: Record<string, any>
  is_active?: boolean
  priority?: number
}

// Backend Service with Routes
export interface BackendServiceWithRoutes extends BackendService {
  routes: ApiRoute[]
}

// Service Health
export interface ServiceHealth {
  id: string
  name: string
  base_url: string
  health_check_url?: string
  is_healthy: boolean
  status_message: string
}

export interface ServicesHealthResponse {
  services: ServiceHealth[]
  last_checked: string
}

// Audit Logs
export type AuditSeverity = 'info' | 'warning' | 'critical'
export type AuditEventCategory = 'authentication' | 'rate_limit' | 'whitelist' | 'admin'

export interface AuditLog {
  id: string
  event_type: string
  event_category: AuditEventCategory
  severity: AuditSeverity
  request_method?: string
  request_path?: string
  client_ip?: string
  user_agent?: string
  api_route_id?: string
  backend_service_id?: string
  message: string
  metadata: Record<string, any>
  status_code?: number
  created_at: string
}

export interface AuditLogResponse {
  logs: AuditLog[]
  total: number
  limit: number
  offset: number
}
