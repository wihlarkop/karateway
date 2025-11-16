<script lang="ts">
  import { onMount } from 'svelte'
  import { VERSION } from '$lib/version'

  // Components
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card'
  import { Badge } from '$lib/components/ui/badge'
  import { Skeleton } from '$lib/components/ui/skeleton'
  import * as Alert from '$lib/components/ui/alert'

  let health = $state<any>(null)
  let loading = $state(true)
  let error = $state<string | null>(null)

  onMount(async () => {
    try {
      const response = await fetch('http://localhost:8081/health')
      if (!response.ok) throw new Error('Failed to fetch health status')
      health = await response.json()
    } catch (err) {
      error = 'Failed to load health status. Make sure the Admin API is running.'
      console.error('Failed to fetch health:', err)
    } finally {
      loading = false
    }
  })

  const getStatusColor = (status: string) => {
    if (status === 'healthy') return 'text-green-500'
    if (status === 'degraded') return 'text-yellow-500'
    return 'text-red-500'
  }

  const getStatusGradient = (status: string) => {
    if (status === 'healthy') return 'from-green-500/20 to-green-500/5'
    if (status === 'degraded') return 'from-yellow-500/20 to-yellow-500/5'
    return 'from-red-500/20 to-red-500/5'
  }
</script>

<div class="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-700">
  <!-- Hero Section -->
  <div class="relative overflow-hidden rounded-2xl bg-gradient-to-br from-primary/10 via-primary/5 to-background border p-8 shadow-lg">
    <div class="absolute top-0 right-0 w-64 h-64 bg-gradient-to-br from-primary/20 to-transparent rounded-full blur-3xl"></div>
    <div class="relative z-10">
      <div class="flex items-center gap-3 mb-3">
        <div class="p-3 rounded-xl bg-gradient-to-br from-primary to-primary/80 shadow-lg shadow-primary/20">
          <svg class="w-6 h-6 text-primary-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>
          </svg>
        </div>
        <h1 class="text-4xl font-bold tracking-tight bg-gradient-to-r from-foreground to-foreground/70 bg-clip-text">
          Dashboard
        </h1>
      </div>
      <p class="text-muted-foreground text-lg">Welcome to Karateway API Gateway Admin Dashboard</p>
      <div class="flex items-center gap-2 mt-4">
        <Badge variant="secondary" class="px-3 py-1">
          <svg class="w-3 h-3 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
          </svg>
          Dashboard v{VERSION}
        </Badge>
        {#if health?.data}
          <Badge variant="outline" class="px-3 py-1">
            <svg class="w-3 h-3 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01"/>
            </svg>
            API v{health.data.version}
          </Badge>
        {/if}
      </div>
    </div>
  </div>

  <!-- Error Alert -->
  {#if error}
    <Alert.Root variant="destructive" class="border-destructive/50 bg-destructive/10">
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
      </svg>
      <Alert.Title>Connection Error</Alert.Title>
      <Alert.Description>{error}</Alert.Description>
    </Alert.Root>
  {/if}

  <!-- System Health Status -->
  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-2xl font-bold tracking-tight">System Health</h2>
        <p class="text-sm text-muted-foreground">Real-time status of core services</p>
      </div>
      {#if health?.data}
        <div class="flex items-center gap-2 px-4 py-2 rounded-xl border {getStatusGradient(health.data.status)} bg-gradient-to-r">
          <div class="w-2 h-2 rounded-full {health.data.status === 'healthy' ? 'bg-green-500 animate-pulse' : health.data.status === 'degraded' ? 'bg-yellow-500' : 'bg-red-500'}"></div>
          <span class="font-semibold {getStatusColor(health.data.status)}">
            {health.data.status.charAt(0).toUpperCase() + health.data.status.slice(1)}
          </span>
        </div>
      {/if}
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
      {#if loading}
        {#each Array(4) as _}
          <Card>
            <CardContent class="pt-6 space-y-3">
              <div class="flex items-center justify-between">
                <Skeleton class="h-4 w-20" />
                <Skeleton class="h-5 w-24" />
              </div>
              <Skeleton class="h-4 w-full" />
              <Skeleton class="h-4 w-3/4" />
            </CardContent>
          </Card>
        {/each}
      {:else if health?.data}
        <!-- Database Health -->
        <Card class="group relative overflow-hidden transition-all duration-300 hover:shadow-lg hover:border-primary/20">
          <div class="absolute top-0 right-0 w-32 h-32 bg-gradient-to-br {health.data.database.connected ? 'from-green-500/10' : 'from-red-500/10'} to-transparent rounded-bl-full transition-all duration-300 group-hover:scale-110"></div>
          <CardContent class="relative z-10 pt-6 space-y-3">
            <div class="flex items-center justify-between">
              <div class="text-sm font-semibold text-muted-foreground">Database</div>
              <div class="flex items-center gap-2">
                <div class="w-2 h-2 rounded-full {health.data.database.connected ? 'bg-green-500 animate-pulse' : 'bg-red-500'}"></div>
                <Badge variant={health.data.database.connected ? 'default' : 'destructive'} class="text-xs">
                  {health.data.database.connected ? 'Connected' : 'Disconnected'}
                </Badge>
              </div>
            </div>
            <div class="text-sm font-medium text-foreground/80 leading-relaxed">
              {health.data.database.message}
            </div>
          </CardContent>
        </Card>

        <!-- Redis Health -->
        <Card class="group relative overflow-hidden transition-all duration-300 hover:shadow-lg hover:border-primary/20">
          <div class="absolute top-0 right-0 w-32 h-32 bg-gradient-to-br {health.data.redis.connected ? 'from-green-500/10' : 'from-red-500/10'} to-transparent rounded-bl-full transition-all duration-300 group-hover:scale-110"></div>
          <CardContent class="relative z-10 pt-6 space-y-3">
            <div class="flex items-center justify-between">
              <div class="text-sm font-semibold text-muted-foreground">Redis Cache</div>
              <div class="flex items-center gap-2">
                <div class="w-2 h-2 rounded-full {health.data.redis.connected ? 'bg-green-500 animate-pulse' : 'bg-red-500'}"></div>
                <Badge variant={health.data.redis.connected ? 'default' : 'destructive'} class="text-xs">
                  {health.data.redis.connected ? 'Connected' : 'Disconnected'}
                </Badge>
              </div>
            </div>
            <div class="text-sm font-medium text-foreground/80 leading-relaxed">
              {health.data.redis.message}
            </div>
          </CardContent>
        </Card>
        <!-- API Version -->
        <Card>
          <CardContent class="pt-6">
            <div class="flex items-center justify-between mb-3">
              <div class="text-sm font-semibold text-muted-foreground">API Version</div>
              <svg class="w-4 h-4 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 20l4-16m2 16l4-16M6 9h14M4 15h14"/>
              </svg>
            </div>
            <div class="text-2xl font-bold">v{health.data.version}</div>
            <div class="text-xs text-muted-foreground mt-2">Admin API</div>
          </CardContent>
        </Card>

        <!-- Dashboard Version -->
        <Card>
          <CardContent class="pt-6">
            <div class="flex items-center justify-between mb-3">
              <div class="text-sm font-semibold text-muted-foreground">Dashboard</div>
              <svg class="w-4 h-4 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"/>
              </svg>
            </div>
            <div class="text-2xl font-bold">v{VERSION}</div>
            <div class="text-xs text-muted-foreground mt-2">Frontend</div>
          </CardContent>
        </Card>
      {/if}
    </div>
  </div>

  <!-- Quick Stats -->
  <div class="space-y-4">
    <div>
      <h2 class="text-2xl font-bold tracking-tight">Quick Stats</h2>
      <p class="text-sm text-muted-foreground">Overview of your gateway configuration</p>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      <!-- Backend Services -->
      <Card class="group relative overflow-hidden transition-all duration-300 hover:shadow-xl hover:shadow-primary/5 hover:-translate-y-1">
        <div class="absolute inset-0 bg-gradient-to-br from-blue-500/10 to-blue-500/5 opacity-50 transition-opacity duration-300 group-hover:opacity-100"></div>
        <CardContent class="relative z-10 pt-6 space-y-3">
          <div class="flex items-center justify-between">
            <div class="text-sm font-medium text-muted-foreground">Backend Services</div>
            <div class="p-2 rounded-lg bg-background/50 backdrop-blur-sm">
              <svg class="w-4 h-4 text-primary transition-transform duration-300 group-hover:scale-110" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01"/>
              </svg>
            </div>
          </div>
          <div class="text-3xl font-bold tracking-tight">-</div>
          <p class="text-xs text-muted-foreground">Total configured services</p>
        </CardContent>
        <div class="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500">
          <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/10 to-transparent -skew-x-12 translate-x-[-200%] group-hover:translate-x-[200%] transition-transform duration-1000"></div>
        </div>
      </Card>

      <!-- API Routes -->
      <Card class="group relative overflow-hidden transition-all duration-300 hover:shadow-xl hover:shadow-primary/5 hover:-translate-y-1">
        <div class="absolute inset-0 bg-gradient-to-br from-purple-500/10 to-purple-500/5 opacity-50 transition-opacity duration-300 group-hover:opacity-100"></div>
        <CardContent class="relative z-10 pt-6 space-y-3">
          <div class="flex items-center justify-between">
            <div class="text-sm font-medium text-muted-foreground">API Routes</div>
            <div class="p-2 rounded-lg bg-background/50 backdrop-blur-sm">
              <svg class="w-4 h-4 text-primary transition-transform duration-300 group-hover:scale-110" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
              </svg>
            </div>
          </div>
          <div class="text-3xl font-bold tracking-tight">-</div>
          <p class="text-xs text-muted-foreground">Active route configurations</p>
        </CardContent>
        <div class="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500">
          <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/10 to-transparent -skew-x-12 translate-x-[-200%] group-hover:translate-x-[200%] transition-transform duration-1000"></div>
        </div>
      </Card>

      <!-- Rate Limits -->
      <Card class="group relative overflow-hidden transition-all duration-300 hover:shadow-xl hover:shadow-primary/5 hover:-translate-y-1">
        <div class="absolute inset-0 bg-gradient-to-br from-orange-500/10 to-orange-500/5 opacity-50 transition-opacity duration-300 group-hover:opacity-100"></div>
        <CardContent class="relative z-10 pt-6 space-y-3">
          <div class="flex items-center justify-between">
            <div class="text-sm font-medium text-muted-foreground">Rate Limits</div>
            <div class="p-2 rounded-lg bg-background/50 backdrop-blur-sm">
              <svg class="w-4 h-4 text-primary transition-transform duration-300 group-hover:scale-110" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
              </svg>
            </div>
          </div>
          <div class="text-3xl font-bold tracking-tight">-</div>
          <p class="text-xs text-muted-foreground">Rate limiting rules</p>
        </CardContent>
        <div class="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500">
          <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/10 to-transparent -skew-x-12 translate-x-[-200%] group-hover:translate-x-[200%] transition-transform duration-1000"></div>
        </div>
      </Card>

      <!-- Whitelist Rules -->
      <Card class="group relative overflow-hidden transition-all duration-300 hover:shadow-xl hover:shadow-primary/5 hover:-translate-y-1">
        <div class="absolute inset-0 bg-gradient-to-br from-green-500/10 to-green-500/5 opacity-50 transition-opacity duration-300 group-hover:opacity-100"></div>
        <CardContent class="relative z-10 pt-6 space-y-3">
          <div class="flex items-center justify-between">
            <div class="text-sm font-medium text-muted-foreground">Whitelist Rules</div>
            <div class="p-2 rounded-lg bg-background/50 backdrop-blur-sm">
              <svg class="w-4 h-4 text-primary transition-transform duration-300 group-hover:scale-110" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/>
              </svg>
            </div>
          </div>
          <div class="text-3xl font-bold tracking-tight">-</div>
          <p class="text-xs text-muted-foreground">Access control rules</p>
        </CardContent>
        <div class="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500">
          <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/10 to-transparent -skew-x-12 translate-x-[-200%] group-hover:translate-x-[200%] transition-transform duration-1000"></div>
        </div>
      </Card>
    </div>
  </div>
</div>
