import type {
    ApiRoute,
    AuditLogResponse,
    BackendService,
    BackendServiceWithRoutes,
    CreateApiRouteRequest,
    CreateBackendServiceRequest,
    CreateRateLimitRequest,
    CreateWhitelistRuleRequest,
    JsonResponse,
    RateLimit,
    ServicesHealthResponse,
    UpdateApiRouteRequest,
    UpdateBackendServiceRequest,
    UpdateRateLimitRequest,
    UpdateWhitelistRuleRequest,
    WhitelistRule,
} from './types'

const API_BASE_URL = 'http://localhost:8081'

class ApiClient {
    private async request<T>(
        endpoint: string,
        options?: RequestInit
    ): Promise<JsonResponse<T>> {
        const url = `${API_BASE_URL}${endpoint}`

        try {
            const response = await fetch(url, {
                ...options,
                headers: {
                    'Content-Type': 'application/json',
                    ...options?.headers,
                },
            })

            return await response.json()
        } catch (error) {
            console.error('API request failed:', error)
            throw error
        }
    }

    // Backend Services
    async getServices(page = 1, limit = 10, search?: string): Promise<JsonResponse<BackendService[]>> {
        const params = new URLSearchParams({
            page: page.toString(),
            limit: limit.toString()
        })
        if (search) {
            params.append('search', search)
        }
        return this.request(`/api/services?${params.toString()}`)
    }

    async getService(id: string): Promise<JsonResponse<BackendService>> {
        return this.request(`/api/services/${id}`)
    }

    async getServiceWithRoutes(id: string): Promise<JsonResponse<BackendServiceWithRoutes>> {
        return this.request(`/api/services/${id}/routes`)
    }

    async createService(data: CreateBackendServiceRequest): Promise<JsonResponse<BackendService>> {
        return this.request('/api/services', {
            method: 'POST',
            body: JSON.stringify(data),
        })
    }

    async updateService(
        id: string,
        data: UpdateBackendServiceRequest
    ): Promise<JsonResponse<BackendService>> {
        return this.request(`/api/services/${id}`, {
            method: 'PUT',
            body: JSON.stringify(data),
        })
    }

    async deleteService(id: string): Promise<JsonResponse<void>> {
        return this.request(`/api/services/${id}`, {
            method: 'DELETE',
        })
    }

    async getServicesHealth(forceRefresh = false): Promise<JsonResponse<ServicesHealthResponse>> {
        const url = forceRefresh
            ? '/api/services/health?force_refresh=true'
            : '/api/services/health'
        return this.request(url)
    }

    // API Routes
    async getRoutes(page = 1, limit = 10, search?: string): Promise<JsonResponse<ApiRoute[]>> {
        const params = new URLSearchParams({
            page: page.toString(),
            limit: limit.toString()
        })
        if (search) {
            params.append('search', search)
        }
        return this.request(`/api/routes?${params.toString()}`)
    }

    async getRoute(id: string): Promise<JsonResponse<ApiRoute>> {
        return this.request(`/api/routes/${id}`)
    }

    async createRoute(data: CreateApiRouteRequest): Promise<JsonResponse<ApiRoute>> {
        return this.request('/api/routes', {
            method: 'POST',
            body: JSON.stringify(data),
        })
    }

    async updateRoute(id: string, data: UpdateApiRouteRequest): Promise<JsonResponse<ApiRoute>> {
        return this.request(`/api/routes/${id}`, {
            method: 'PUT',
            body: JSON.stringify(data),
        })
    }

    async deleteRoute(id: string): Promise<JsonResponse<void>> {
        return this.request(`/api/routes/${id}`, {
            method: 'DELETE',
        })
    }

    // Rate Limits
    async getRateLimits(page = 1, limit = 10, search?: string): Promise<JsonResponse<RateLimit[]>> {
        const params = new URLSearchParams({
            page: page.toString(),
            limit: limit.toString()
        })
        if (search) {
            params.append('search', search)
        }
        return this.request(`/api/rate-limits?${params.toString()}`)
    }

    async getRateLimit(id: string): Promise<JsonResponse<RateLimit>> {
        return this.request(`/api/rate-limits/${id}`)
    }

    async createRateLimit(data: CreateRateLimitRequest): Promise<JsonResponse<RateLimit>> {
        return this.request('/api/rate-limits', {
            method: 'POST',
            body: JSON.stringify(data),
        })
    }

    async updateRateLimit(
        id: string,
        data: UpdateRateLimitRequest
    ): Promise<JsonResponse<RateLimit>> {
        return this.request(`/api/rate-limits/${id}`, {
            method: 'PUT',
            body: JSON.stringify(data),
        })
    }

    async deleteRateLimit(id: string): Promise<JsonResponse<void>> {
        return this.request(`/api/rate-limits/${id}`, {
            method: 'DELETE',
        })
    }

    // Whitelist Rules
    async getWhitelistRules(page = 1, limit = 10, search?: string): Promise<JsonResponse<WhitelistRule[]>> {
        const params = new URLSearchParams({
            page: page.toString(),
            limit: limit.toString()
        })
        if (search) {
            params.append('search', search)
        }
        return this.request(`/api/whitelist?${params.toString()}`)
    }

    async getWhitelistRule(id: string): Promise<JsonResponse<WhitelistRule>> {
        return this.request(`/api/whitelist/${id}`)
    }

    async createWhitelistRule(
        data: CreateWhitelistRuleRequest
    ): Promise<JsonResponse<WhitelistRule>> {
        return this.request('/api/whitelist', {
            method: 'POST',
            body: JSON.stringify(data),
        })
    }

    async updateWhitelistRule(
        id: string,
        data: UpdateWhitelistRuleRequest
    ): Promise<JsonResponse<WhitelistRule>> {
        return this.request(`/api/whitelist/${id}`, {
            method: 'PUT',
            body: JSON.stringify(data),
        })
    }

    async deleteWhitelistRule(id: string): Promise<JsonResponse<void>> {
        return this.request(`/api/whitelist/${id}`, {
            method: 'DELETE',
        })
    }

    // Audit Logs
    async getAuditLogs(limit = 50, offset = 0): Promise<AuditLogResponse> {
        const params = new URLSearchParams({
            limit: limit.toString(),
            offset: offset.toString()
        })
        const response = await this.request<AuditLogResponse>(`/api/audit-logs?${params.toString()}`)
        return response.data!
    }
}

export const api = new ApiClient()
