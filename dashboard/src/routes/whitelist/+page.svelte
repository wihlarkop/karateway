<script lang="ts">
  import { onMount } from 'svelte'
  import { api } from '$lib/api'
  import type { WhitelistRule, ApiRoute, CreateWhitelistRuleRequest, RuleType } from '$lib/types'
  import { toasts } from '$lib/stores/toast'
  import {
    createSvelteTable,
    FlexRender,
    renderComponent,
    renderSnippet
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
  import { Textarea } from '$lib/components/ui/textarea'
  import { Card, CardContent } from '$lib/components/ui/card'
  import { Skeleton } from '$lib/components/ui/skeleton'

  let whitelistRules = $state<WhitelistRule[]>([])
  let apiRoutes = $state<ApiRoute[]>([])
  let loading = $state(true)
  let showModal = $state(false)
  let editingRule = $state<WhitelistRule | null>(null)
  let error = $state<string | null>(null)
  let globalFilter = $state('')
  let currentPage = $state(1)
  let pageSize = $state(10)
  let totalRules = $state(0)

  const ruleTypes: RuleType[] = ['Ip', 'ApiKey', 'Jwt', 'Custom']

  // Pagination state for TanStack Table
  let pagination = $state<PaginationState>({
    pageIndex: currentPage - 1,
    pageSize: pageSize
  })

  // Form state - using temporary fields that will be converted to config
  let formData = $state({
    rule_name: '',
    rule_type: 'Ip' as RuleType,
    api_route_id: undefined as string | undefined,
    priority: 0,
    // Temporary fields based on rule type
    allowed_ips: '',
    allowed_keys: '',
    jwt_secret: '',
    allowed_issuers: '',
    allowed_audiences: '',
  })

  // TanStack Table column definitions
  const columns: ColumnDef<WhitelistRule>[] = [
    {
      accessorKey: 'rule_name',
      header: 'Rule Name',
      cell: (info) => info.row.original.rule_name
    },
    {
      accessorKey: 'rule_type',
      header: 'Type',
      cell: (info) => info.getValue()
    },
    {
      accessorKey: 'config',
      header: 'Configuration',
      cell: (info) => JSON.stringify(info.getValue())
    },
    {
      accessorKey: 'priority',
      header: 'Priority',
      cell: (info) => info.getValue()
    },
    {
      accessorKey: 'api_route_id',
      header: 'Route',
      cell: (info) => getRouteName(info.getValue() as string | undefined)
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
      return whitelistRules
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
    pageCount: Math.ceil((totalRules || 0) / pageSize)
  })

  // Debounce search to avoid too many API calls
  let searchTimeout: ReturnType<typeof setTimeout> | null = null
  $effect(() => {
    // Watch for search changes
    const query = globalFilter
    if (searchTimeout) clearTimeout(searchTimeout)
    searchTimeout = setTimeout(() => {
      currentPage = 1
      pagination.pageIndex = 0
      loadData()
    }, 300) // 300ms debounce
  })

  onMount(() => {
    loadData()
  })

  async function loadData() {
    loading = true
    error = null
    try {
      const [rulesRes, routesRes] = await Promise.all([
        api.getWhitelistRules(currentPage, pageSize, globalFilter || undefined),
        api.getRoutes(1, 100)
      ])

      if (rulesRes.success && rulesRes.data) {
        whitelistRules = rulesRes.data
        totalRules = rulesRes.meta?.total_data || whitelistRules.length
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
    editingRule = null
    formData = {
      rule_name: '',
      rule_type: 'Ip',
      api_route_id: undefined,
      priority: 0,
      allowed_ips: '',
      allowed_keys: '',
      jwt_secret: '',
      allowed_issuers: '',
      allowed_audiences: '',
    }
    showModal = true
  }

  function openEditModal(rule: WhitelistRule) {
    editingRule = rule
    formData = {
      rule_name: rule.rule_name,
      rule_type: rule.rule_type,
      api_route_id: rule.api_route_id,
      priority: rule.priority,
      // Extract config fields based on rule type
      allowed_ips: rule.rule_type === 'Ip' ? (rule.config.allowed_ips || []).join(', ') : '',
      allowed_keys: rule.rule_type === 'ApiKey' ? (rule.config.allowed_keys || []).join(', ') : '',
      jwt_secret: rule.rule_type === 'Jwt' ? (rule.config.jwt_secret || '') : '',
      allowed_issuers: rule.rule_type === 'Jwt' ? (rule.config.allowed_issuers || []).join(', ') : '',
      allowed_audiences: rule.rule_type === 'Jwt' ? (rule.config.allowed_audiences || []).join(', ') : '',
    }
    showModal = true
  }

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault()
    error = null
    try {
      // Build config object based on rule type
      let config: Record<string, any> = {}

      if (formData.rule_type === 'Ip') {
        config = {
          allowed_ips: formData.allowed_ips.split(',').map(ip => ip.trim()).filter(ip => ip.length > 0)
        }
      } else if (formData.rule_type === 'ApiKey') {
        config = {
          allowed_keys: formData.allowed_keys.split(',').map(key => key.trim()).filter(key => key.length > 0)
        }
      } else if (formData.rule_type === 'Jwt') {
        config = {
          jwt_secret: formData.jwt_secret,
          allowed_issuers: formData.allowed_issuers.split(',').map(iss => iss.trim()).filter(iss => iss.length > 0),
          allowed_audiences: formData.allowed_audiences.split(',').map(aud => aud.trim()).filter(aud => aud.length > 0)
        }
      }

      const requestData: CreateWhitelistRuleRequest = {
        rule_name: formData.rule_name,
        rule_type: formData.rule_type,
        api_route_id: formData.api_route_id,
        config: config,
        priority: formData.priority
      }

      if (editingRule) {
        await api.updateWhitelistRule(editingRule.id, requestData)
        toasts.add('success', `Rule "${formData.rule_name}" updated successfully`)
      } else {
        await api.createWhitelistRule(requestData)
        toasts.add('success', `Rule "${formData.rule_name}" created successfully`)
      }
      showModal = false
      await loadData()
    } catch (err) {
      const errorMsg = 'Failed to save whitelist rule'
      error = errorMsg
      toasts.add('error', errorMsg)
      console.error(err)
    }
  }

  async function handleDelete(rule: WhitelistRule) {
    if (!confirm(`Are you sure you want to delete "${rule.rule_name}"?`)) return

    error = null
    try {
      await api.deleteWhitelistRule(rule.id)
      toasts.add('success', `Rule "${rule.rule_name}" deleted successfully`)
      await loadData()
    } catch (err) {
      const errorMsg = 'Failed to delete whitelist rule'
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

  function getConfigDisplay(rule: WhitelistRule): string {
    if (rule.rule_type === 'Ip') {
      return (rule.config.allowed_ips || []).join(', ')
    } else if (rule.rule_type === 'ApiKey') {
      const keys = rule.config.allowed_keys || []
      return keys.length > 0 ? `${keys.length} key(s)` : 'No keys'
    } else if (rule.rule_type === 'Jwt') {
      return rule.config.jwt_secret ? 'JWT configured' : 'No config'
    }
    return 'Custom'
  }
</script>

{#snippet NameCell({rule}: {rule: WhitelistRule})}
  <div>
    <div class="font-semibold">{rule.rule_name}</div>
    <div class="text-sm text-muted-foreground mt-0.5">Priority: {rule.priority}</div>
  </div>
{/snippet}

{#snippet TypeCell({rule}: {rule: WhitelistRule})}
  <Badge variant="outline" class="w-fit">{rule.rule_type}</Badge>
{/snippet}

{#snippet ConfigCell({rule}: {rule: WhitelistRule})}
  <code class="text-sm font-mono bg-muted px-2 py-1 rounded">{getConfigDisplay(rule)}</code>
{/snippet}

{#snippet RouteCell({rule}: {rule: WhitelistRule})}
  <span class="text-sm text-muted-foreground">{getRouteName(rule.api_route_id)}</span>
{/snippet}

{#snippet ActionsCell({rule}: {rule: WhitelistRule})}
  <div class="flex items-center justify-center gap-2">
    <Button
      variant="ghost"
      size="sm"
      onclick={() => openEditModal(rule)}
      class="h-8 px-3 text-xs font-medium"
    >
      Edit
    </Button>
    <Button
      variant="ghost"
      size="sm"
      onclick={() => handleDelete(rule)}
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
      <h1 class="text-3xl font-bold tracking-tight">Whitelist Rules</h1>
      <p class="text-muted-foreground mt-1">Manage access control and authentication rules</p>
    </div>
    <Button onclick={openCreateModal}>
      <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      New Rule
    </Button>
  </div>

  <!-- Search -->
  <div class="relative flex-1 max-w-md w-full">
    <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
    </svg>
    <Input
      type="search"
      placeholder="Search whitelist rules..."
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

  <!-- Whitelist Rules Table -->
  <Card>
    <CardContent class="p-0">
      {#if loading}
        <div class="p-6 space-y-2">
          {#each Array(5) as _}
            <Skeleton class="h-12 w-full" />
          {/each}
        </div>
      {:else if whitelistRules.length === 0}
        <div class="text-center py-12">
          <svg class="mx-auto h-12 w-12 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
          </svg>
          <h3 class="mt-4 text-lg font-semibold">No whitelist rules found</h3>
          <p class="text-muted-foreground mt-2">
            {globalFilter ? 'Try adjusting your search query' : 'Get started by creating your first whitelist rule'}
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
                {@const rule = row.original}
                <Table.Row>
                  <Table.Cell class="px-6 py-3">
                    {@render NameCell({rule})}
                  </Table.Cell>
                  <Table.Cell class="px-6 py-3">
                    {@render TypeCell({rule})}
                  </Table.Cell>
                  <Table.Cell class="px-6 py-3">
                    {@render ConfigCell({rule})}
                  </Table.Cell>
                  <Table.Cell class="px-6 py-3">
                    {rule.priority}
                  </Table.Cell>
                  <Table.Cell class="px-6 py-3">
                    {@render RouteCell({rule})}
                  </Table.Cell>
                  <Table.Cell class="px-6 py-3 text-right">
                    {@render ActionsCell({rule})}
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
            of {table.getFilteredRowModel().rows.length} rules
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
  <Dialog.Content class="sm:max-w-[550px]">
    <Dialog.Header>
      <Dialog.Title>{editingRule ? 'Edit Whitelist Rule' : 'Create New Whitelist Rule'}</Dialog.Title>
      <Dialog.Description>
        {editingRule ? 'Update the whitelist rule configuration' : 'Add a new access control rule'}
      </Dialog.Description>
    </Dialog.Header>

    <form onsubmit={handleSubmit} class="space-y-4">
      <div class="grid gap-4">
        <div class="grid gap-2">
          <Label for="name">
            Rule Name <span class="text-destructive">*</span>
          </Label>
          <Input
            id="name"
            type="text"
            bind:value={formData.rule_name}
            placeholder="production-server-access"
            required
          />
        </div>

        <div class="grid gap-2">
          <Label for="type">
            Rule Type <span class="text-destructive">*</span>
          </Label>
          <select
            id="type"
            bind:value={formData.rule_type}
            required
            class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
          >
            {#each ruleTypes as type}
              <option value={type}>{type}</option>
            {/each}
          </select>
          <p class="text-sm text-muted-foreground">Type of whitelist rule to create</p>
        </div>

        <!-- IP Rule Config -->
        {#if formData.rule_type === 'Ip'}
          <div class="grid gap-2">
            <Label for="allowed_ips">
              Allowed IPs <span class="text-destructive">*</span>
            </Label>
            <Input
              id="allowed_ips"
              type="text"
              bind:value={formData.allowed_ips}
              placeholder="192.168.1.1, 10.0.0.50"
              required
            />
            <p class="text-sm text-muted-foreground">Comma-separated list of allowed IP addresses</p>
          </div>
        {/if}

        <!-- API Key Rule Config -->
        {#if formData.rule_type === 'ApiKey'}
          <div class="grid gap-2">
            <Label for="allowed_keys">
              Allowed API Keys <span class="text-destructive">*</span>
            </Label>
            <Textarea
              id="allowed_keys"
              bind:value={formData.allowed_keys}
              placeholder="key123, key456, key789"
              rows="3"
              required
            />
            <p class="text-sm text-muted-foreground">Comma-separated list of allowed API keys</p>
          </div>
        {/if}

        <!-- JWT Rule Config -->
        {#if formData.rule_type === 'Jwt'}
          <div class="grid gap-2">
            <Label for="jwt_secret">
              JWT Secret <span class="text-destructive">*</span>
            </Label>
            <Input
              id="jwt_secret"
              type="text"
              bind:value={formData.jwt_secret}
              placeholder="your-jwt-secret"
              required
            />
          </div>
          <div class="grid gap-2">
            <Label for="allowed_issuers">Allowed Issuers</Label>
            <Input
              id="allowed_issuers"
              type="text"
              bind:value={formData.allowed_issuers}
              placeholder="auth.example.com, auth2.example.com"
            />
            <p class="text-sm text-muted-foreground">Comma-separated list of allowed JWT issuers</p>
          </div>
          <div class="grid gap-2">
            <Label for="allowed_audiences">Allowed Audiences</Label>
            <Input
              id="allowed_audiences"
              type="text"
              bind:value={formData.allowed_audiences}
              placeholder="api.example.com"
            />
            <p class="text-sm text-muted-foreground">Comma-separated list of allowed JWT audiences</p>
          </div>
        {/if}

        <div class="grid gap-2">
          <Label for="priority">
            Priority
          </Label>
          <Input
            id="priority"
            type="number"
            bind:value={formData.priority}
            placeholder="0"
          />
          <p class="text-sm text-muted-foreground">Higher priority rules are evaluated first</p>
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
          <p class="text-sm text-muted-foreground">Apply to all routes or select a specific route</p>
        </div>
      </div>

      <Dialog.Footer>
        <Button type="button" variant="outline" onclick={() => showModal = false}>
          Cancel
        </Button>
        <Button type="submit">
          {editingRule ? 'Update' : 'Create'}
        </Button>
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
