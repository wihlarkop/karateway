<script lang="ts">
  import { onMount } from 'svelte'
  import { api } from '$lib/api'
  import type { RateLimit, ApiRoute, CreateRateLimitRequest, IdentifierType } from '$lib/types'
  import { toasts } from '$lib/stores/toast'
  import {
    createSvelteTable,
    FlexRender,
  } from '$lib/components/ui/data-table'
  import {
    getCoreRowModel,
    getPaginationRowModel,
    type ColumnDef,
    type PaginationState
  } from '@tanstack/table-core'

  // shadcn components
  import { Button } from '$lib/components/ui/button'
  import { Input } from '$lib/components/ui/input'
  import { Label } from '$lib/components/ui/label'
  import { Badge } from '$lib/components/ui/badge'
  import * as Dialog from '$lib/components/ui/dialog'
  import * as Table from '$lib/components/ui/table'
  import * as Alert from '$lib/components/ui/alert'
  import { Card, CardContent } from '$lib/components/ui/card'
  import { Skeleton } from '$lib/components/ui/skeleton'

  let rateLimits = $state<RateLimit[]>([])
  let apiRoutes = $state<ApiRoute[]>([])
  let loading = $state(true)
  let showModal = $state(false)
  let editingLimit = $state<RateLimit | null>(null)
  let error = $state<string | null>(null)
  let globalFilter = $state('')
  let currentPage = $state(1)
  let pageSize = $state(10)
  let totalRateLimits = $state(0)

  const identifierTypes: IdentifierType[] = ['Ip', 'ApiKey', 'UserId', 'Global']

  // Pagination state for TanStack Table
  let pagination = $state<PaginationState>({
    pageIndex: currentPage - 1,
    pageSize: pageSize
  })

  // Form state
  let formData = $state<CreateRateLimitRequest>({
    name: '',
    api_route_id: undefined,
    identifier_type: 'Ip',
    max_requests: 100,
    window_seconds: 60,
    burst_size: 10,
  })

  // TanStack Table column definitions
  const columns: ColumnDef<RateLimit>[] = [
    {
      accessorKey: 'name',
      header: 'Name',
      cell: (info) => info.getValue()
    },
    {
      accessorKey: 'api_route_id',
      header: 'Route',
      cell: (info) => getRouteName(info.getValue() as string | undefined)
    },
    {
      accessorKey: 'identifier_type',
      header: 'Identifier Type',
      cell: (info) => info.getValue()
    },
    {
      accessorKey: 'max_requests',
      header: 'Max Requests',
      cell: (info) => info.getValue()
    },
    {
      accessorKey: 'window_seconds',
      header: 'Window',
      cell: (info) => `${info.getValue()}s`
    },
    {
      accessorKey: 'burst_size',
      header: 'Burst Size',
      cell: (info) => info.getValue()
    },
    {
      id: 'actions',
      header: () => 'Actions',
      cell: (info) => 'Edit | Delete',
      meta: {
        headerClass: 'text-center'
      }
    }
  ]

  // Create TanStack Table
  const table = createSvelteTable({
    get data() {
      return rateLimits
    },
    columns,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    state: {
      get pagination() {
        return pagination
      }
    },
    onPaginationChange: (updater) => {
      pagination = typeof updater === 'function' ? updater(pagination) : updater
      currentPage = pagination.pageIndex + 1
    },
    manualPagination: false,
    pageCount: Math.ceil((totalRateLimits || 0) / pageSize)
  })

  // Debounce search to avoid too many API calls
  let searchTimeout: ReturnType<typeof setTimeout> | null = null
  $effect(() => {
    const query = globalFilter
    if (searchTimeout) clearTimeout(searchTimeout)
    searchTimeout = setTimeout(() => {
      currentPage = 1
      pagination.pageIndex = 0
      loadData()
    }, 300)
  })

  onMount(() => {
    loadData()
  })

  async function loadData() {
    loading = true
    error = null
    try {
      const [limitsRes, routesRes] = await Promise.all([
        api.getRateLimits(currentPage, pageSize, globalFilter || undefined),
        api.getRoutes(1, 100)
      ])

      if (limitsRes.success && limitsRes.data) {
        rateLimits = limitsRes.data
        totalRateLimits = limitsRes.meta?.total_data || rateLimits.length
      }
      if (routesRes.success && routesRes.data) {
        apiRoutes = routesRes.data
      }
    } catch (err) {
      error = 'Failed to load data'
      console.error(err)
    } finally {
      loading = false
    }
  }

  function openCreateModal() {
    editingLimit = null
    formData = {
      name: '',
      api_route_id: undefined,
      identifier_type: 'Ip',
      max_requests: 100,
      window_seconds: 60,
      burst_size: 10,
    }
    showModal = true
  }

  function openEditModal(limit: RateLimit) {
    editingLimit = limit
    formData = {
      name: limit.name,
      api_route_id: limit.api_route_id,
      identifier_type: limit.identifier_type,
      max_requests: limit.max_requests,
      window_seconds: limit.window_seconds,
      burst_size: limit.burst_size,
    }
    showModal = true
  }

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault()
    error = null
    try {
      if (editingLimit) {
        await api.updateRateLimit(editingLimit.id, formData)
        toasts.add('success', 'Rate limit updated successfully')
      } else {
        await api.createRateLimit(formData)
        toasts.add('success', 'Rate limit created successfully')
      }
      showModal = false
      await loadData()
    } catch (err) {
      const errorMsg = 'Failed to save rate limit'
      error = errorMsg
      toasts.add('error', errorMsg)
      console.error(err)
    }
  }

  async function handleDelete(limit: RateLimit) {
    if (!confirm('Are you sure you want to delete this rate limit?')) return

    error = null
    try {
      await api.deleteRateLimit(limit.id)
      toasts.add('success', 'Rate limit deleted successfully')
      await loadData()
    } catch (err) {
      const errorMsg = 'Failed to delete rate limit'
      error = errorMsg
      toasts.add('error', errorMsg)
      console.error(err)
    }
  }

  function getRouteName(routeId?: string): string {
    if (!routeId) return 'Global'
    const route = apiRoutes.find(r => r.id === routeId)
    return route ? `${route.method} ${route.path_pattern}` : 'Unknown'
  }
</script>

{#snippet ActionsCell({limit}: {limit: RateLimit})}
  <div class="flex items-center justify-center gap-2">
    <Button
      variant="ghost"
      size="sm"
      onclick={() => openEditModal(limit)}
      class="h-8 px-3 text-xs font-medium"
    >
      Edit
    </Button>
    <Button
      variant="ghost"
      size="sm"
      onclick={() => handleDelete(limit)}
      class="h-8 px-3 text-xs font-medium text-destructive hover:text-destructive hover:bg-destructive/10"
    >
      Delete
    </Button>
  </div>
{/snippet}

<div class="space-y-6">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
    <div>
      <h1 class="text-3xl font-bold tracking-tight">Rate Limits</h1>
      <p class="text-muted-foreground mt-1">Control API request rates and prevent abuse</p>
    </div>
    <Button onclick={openCreateModal}>
      <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      New Rate Limit
    </Button>
  </div>

  <!-- Search -->
  <div class="relative flex-1 max-w-md w-full">
    <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
    </svg>
    <Input
      type="search"
      placeholder="Search rate limits..."
      bind:value={globalFilter}
      class="pl-10"
    />
  </div>

  <!-- Error Alert -->
  {#if error}
    <Alert.Root variant="destructive">
      <Alert.Title>Error</Alert.Title>
      <Alert.Description>{error}</Alert.Description>
    </Alert.Root>
  {/if}

  <!-- Rate Limits Table -->
  <Card>
    <CardContent class="p-0">
      {#if loading}
        <div class="p-6 space-y-2">
          {#each Array(5) as _}
            <Skeleton class="h-12 w-full" />
          {/each}
        </div>
      {:else if rateLimits.length === 0}
        <div class="text-center py-12">
          <svg class="mx-auto h-12 w-12 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
          </svg>
          <h3 class="mt-4 text-lg font-semibold">No rate limits found</h3>
          <p class="text-muted-foreground mt-2">
            {globalFilter ? 'Try adjusting your search query' : 'Get started by creating your first rate limit'}
          </p>
        </div>
      {:else}
        <div class="overflow-x-auto">
          <Table.Root>
            <Table.Header>
              {#each table.getHeaderGroups() as headerGroup}
                <Table.Row class="hover:bg-transparent">
                  {#each headerGroup.headers as header}
                    <Table.Head class="h-12 px-6 font-semibold {header.column.columnDef.meta?.headerClass || ''}">
                      {#if !header.isPlaceholder}
                        <FlexRender content={header.column.columnDef.header} context={header.getContext()} />
                      {/if}
                    </Table.Head>
                  {/each}
                </Table.Row>
              {/each}
            </Table.Header>
            <Table.Body>
              {#each table.getRowModel().rows as row}
                {@const limit = row.original}
                <Table.Row>
                  <Table.Cell class="px-6 py-3">
                    <span class="font-semibold">{limit.name}</span>
                  </Table.Cell>
                  <Table.Cell class="px-6 py-3">
                    <code class="text-sm font-mono bg-muted px-2 py-1 rounded">{getRouteName(limit.api_route_id)}</code>
                  </Table.Cell>
                  <Table.Cell class="px-6 py-3">
                    <Badge variant="outline" class="w-fit">{limit.identifier_type}</Badge>
                  </Table.Cell>
                  <Table.Cell class="px-6 py-3">
                    <span class="text-sm font-semibold">{limit.max_requests}</span>
                  </Table.Cell>
                  <Table.Cell class="px-6 py-3">
                    <span class="text-sm text-muted-foreground">{limit.window_seconds}s</span>
                  </Table.Cell>
                  <Table.Cell class="px-6 py-3">
                    <span class="text-sm text-muted-foreground">{limit.burst_size}</span>
                  </Table.Cell>
                  <Table.Cell class="px-6 py-3">
                    {@render ActionsCell({limit})}
                  </Table.Cell>
                </Table.Row>
              {/each}
            </Table.Body>
          </Table.Root>
        </div>

        <!-- Pagination -->
        <div class="flex items-center justify-between py-4 px-6 border-t">
          <div class="text-sm text-muted-foreground">
            Showing {table.getState().pagination.pageIndex * table.getState().pagination.pageSize + 1}
            - {Math.min((table.getState().pagination.pageIndex + 1) * table.getState().pagination.pageSize, table.getFilteredRowModel().rows.length)}
            of {table.getFilteredRowModel().rows.length} rate limits
          </div>
          <div class="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onclick={() => table.previousPage()}
              disabled={!table.getCanPreviousPage()}
            >
              Previous
            </Button>
            <Button
              variant="outline"
              size="sm"
              onclick={() => table.nextPage()}
              disabled={!table.getCanNextPage()}
            >
              Next
            </Button>
          </div>
        </div>
      {/if}
    </CardContent>
  </Card>
</div>

<!-- Modal -->
<Dialog.Root bind:open={showModal}>
  <Dialog.Content class="sm:max-w-[500px]">
    <Dialog.Header>
      <Dialog.Title>{editingLimit ? 'Edit Rate Limit' : 'Create New Rate Limit'}</Dialog.Title>
      <Dialog.Description>
        {editingLimit ? 'Update the rate limit configuration' : 'Add a new rate limit policy'}
      </Dialog.Description>
    </Dialog.Header>

    <form onsubmit={handleSubmit} class="space-y-4">
      <div class="grid gap-4">
        <div class="grid gap-2">
          <Label for="name">
            Name <span class="text-destructive">*</span>
          </Label>
          <Input
            id="name"
            type="text"
            bind:value={formData.name}
            placeholder="e.g., API Rate Limit"
            minlength="1"
            maxlength="100"
            required
          />
          <p class="text-sm text-muted-foreground">Descriptive name for this rate limit</p>
        </div>

        <div class="grid gap-2">
          <Label for="route">API Route (Optional)</Label>
          <select
            id="route"
            bind:value={formData.api_route_id}
            class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
          >
            <option value={undefined}>🌐 Global (All Routes)</option>
            {#if apiRoutes.length > 0}
              <optgroup label="API Routes">
                {#each apiRoutes as route}
                  <option value={route.id}>{route.method} {route.path_pattern}</option>
                {/each}
              </optgroup>
            {/if}
          </select>
          <p class="text-sm text-muted-foreground">Leave as global or select a specific route</p>
        </div>

        <div class="grid gap-2">
          <Label for="identifier">
            Identifier Type <span class="text-destructive">*</span>
          </Label>
          <select
            id="identifier"
            bind:value={formData.identifier_type}
            required
            class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
          >
            {#each identifierTypes as type}
              <option value={type}>
                {#if type === 'Ip'}🌍{:else if type === 'ApiKey'}🔑{:else if type === 'UserId'}👤{:else}🌐{/if}
                {type}
              </option>
            {/each}
          </select>
          <p class="text-sm text-muted-foreground">How to identify requests for rate limiting</p>
        </div>

        <div class="grid gap-2">
          <Label for="max_requests">
            Max Requests <span class="text-destructive">*</span>
          </Label>
          <Input
            id="max_requests"
            type="number"
            bind:value={formData.max_requests}
            min="1"
            max="10000"
            required
          />
          <p class="text-sm text-muted-foreground">Maximum number of requests allowed</p>
        </div>

        <div class="grid gap-2">
          <Label for="window">
            Window (seconds) <span class="text-destructive">*</span>
          </Label>
          <Input
            id="window"
            type="number"
            bind:value={formData.window_seconds}
            min="1"
            max="86400"
            required
          />
          <p class="text-sm text-muted-foreground">Time window for rate limiting</p>
        </div>

        <div class="grid gap-2">
          <Label for="burst">
            Burst Size <span class="text-destructive">*</span>
          </Label>
          <Input
            id="burst"
            type="number"
            bind:value={formData.burst_size}
            min="0"
            max="1000"
            required
          />
          <p class="text-sm text-muted-foreground">Additional requests allowed in bursts</p>
        </div>
      </div>

      <Dialog.Footer>
        <Button type="button" variant="outline" onclick={() => showModal = false}>
          Cancel
        </Button>
        <Button type="submit">
          {editingLimit ? 'Update' : 'Create'}
        </Button>
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
