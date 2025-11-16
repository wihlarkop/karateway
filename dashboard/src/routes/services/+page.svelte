<script lang="ts">
    import {onMount} from 'svelte'
    import {api} from '$lib/api'
    import type {BackendService, CreateBackendServiceRequest, ServiceHealth} from '$lib/types'
    import {toasts} from '$lib/stores/toast'
    import {
        createSvelteTable,
        FlexRender,
        renderComponent,
        renderSnippet
    } from '$lib/components/ui/data-table'
    import {
        getCoreRowModel,
        getPaginationRowModel,
        getFilteredRowModel,
        type ColumnDef,
        type PaginationState
    } from '@tanstack/table-core'

    // shadcn components
    import {Button} from '$lib/components/ui/button'
    import {Input} from '$lib/components/ui/input'
    import {Label} from '$lib/components/ui/label'
    import {Badge} from '$lib/components/ui/badge'
    import * as Dialog from '$lib/components/ui/dialog'
    import * as Table from '$lib/components/ui/table'
    import * as Alert from '$lib/components/ui/alert'
    import * as DropdownMenu from '$lib/components/ui/dropdown-menu'
    import {Card, CardContent} from '$lib/components/ui/card'
    import {Skeleton} from '$lib/components/ui/skeleton'

    let services = $state<BackendService[]>([])
    let servicesHealth = $state<ServiceHealth[]>([])
    let lastChecked = $state<string | null>(null)
    let loading = $state(true)
    let healthLoading = $state(false)
    let showModal = $state(false)
    let editingService = $state<BackendService | null>(null)
    let error = $state<string | null>(null)
    let globalFilter = $state('')
    let currentPage = $state(1)
    let pageSize = $state(10)
    let totalServices = $state(0)

    const totalPages = $derived(Math.ceil(totalServices / pageSize))

    // Pagination state for TanStack Table
    let pagination = $state<PaginationState>({
        pageIndex: currentPage - 1,
        pageSize: pageSize
    })

    // Form state
    let formData = $state<CreateBackendServiceRequest>({
        name: '',
        description: '',
        base_url: '',
        health_check_url: '',
        health_check_interval_seconds: 30,
        timeout_ms: 5000,
    })

    // TanStack Table column definitions
    const columns: ColumnDef<BackendService>[] = [
        {
            accessorKey: 'name',
            header: 'Name',
            cell: (info) => info.row.original.name
        },
        {
            accessorKey: 'base_url',
            header: 'Base URL',
            cell: (info) => info.row.original.base_url
        },
        {
            accessorKey: 'health_status',
            header: 'Health Status',
            cell: (info) => {
                const health = getServiceHealth(info.row.original.id)
                if (health) {
                    return health.is_healthy ? 'Healthy' : 'Unhealthy'
                } else if (info.row.original.health_check_url) {
                    return 'Checking...'
                }
                return 'No health check'
            }
        },
        {
            accessorKey: 'is_active',
            header: 'Active',
            cell: (info) => info.getValue() ? 'Active' : 'Inactive'
        },
        {
            accessorKey: 'timeout_ms',
            header: 'Timeout',
            cell: (info) => `${info.getValue()}ms`
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
            return services
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
        pageCount: Math.ceil((totalServices || 0) / pageSize)
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
            loadServices()
        }, 300) // 300ms debounce
    })

    onMount(() => {
        loadServices()
        loadServicesHealth(false)
    })

    async function loadServices() {
        loading = true
        error = null
        try {
            const response = await api.getServices(currentPage, pageSize, globalFilter || undefined)
            if (response.success && response.data) {
                services = response.data
                totalServices = response.meta?.total_data || services.length
            }
        } catch (err) {
            error = 'Failed to load services'
            console.error(err)
        } finally {
            loading = false
        }
    }

    async function loadServicesHealth(forceRefresh = false) {
        healthLoading = true
        try {
            const response = await api.getServicesHealth(forceRefresh)
            if (response.success && response.data) {
                servicesHealth = response.data.services
                lastChecked = response.data.last_checked
            }
        } catch (err) {
            console.error('Failed to load services health:', err)
            toasts.add('error', 'Failed to load health status')
        } finally {
            healthLoading = false
        }
    }

    async function refreshHealthStatus() {
        await loadServicesHealth(true)
        toasts.add('success', 'Health status refreshed')
    }

    function getServiceHealth(serviceId: string): ServiceHealth | undefined {
        return servicesHealth.find(h => h.id === serviceId)
    }

    function getTimeAgo(timestamp: string): string {
        const now = new Date()
        const checked = new Date(timestamp)
        const diffMs = now.getTime() - checked.getTime()
        const diffMins = Math.floor(diffMs / 60000)
        const diffHours = Math.floor(diffMins / 60)
        const diffDays = Math.floor(diffHours / 24)

        if (diffMins < 1) return 'Just now'
        if (diffMins < 60) return `${diffMins} minute${diffMins > 1 ? 's' : ''} ago`
        if (diffHours < 24) return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`
        return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`
    }


    function openCreateModal() {
        editingService = null
        formData = {
            name: '',
            description: '',
            base_url: '',
            health_check_url: '',
            health_check_interval_seconds: 30,
            timeout_ms: 5000,
        }
        showModal = true
    }

    function openEditModal(service: BackendService) {
        editingService = service
        formData = {
            name: service.name,
            description: service.description || '',
            base_url: service.base_url,
            health_check_url: service.health_check_url || '',
            health_check_interval_seconds: service.health_check_interval_seconds || 30,
            timeout_ms: service.timeout_ms || 5000,
        }
        showModal = true
    }

    async function handleSubmit(e: SubmitEvent) {
        e.preventDefault()
        error = null
        try {
            // Clean up empty strings to null for optional fields
            const cleanedData = {
                ...formData,
                description: formData.description?.trim() || null,
                health_check_url: formData.health_check_url?.trim() || null,
            }

            if (editingService) {
                await api.updateService(editingService.id, cleanedData)
                toasts.add('success', `Service "${formData.name}" updated successfully`)
            } else {
                await api.createService(cleanedData)
                toasts.add('success', `Service "${formData.name}" created successfully`)
            }
            showModal = false
            await loadServices()
        } catch (err) {
            const errorMsg = 'Failed to save service'
            error = errorMsg
            toasts.add('error', errorMsg)
            console.error(err)
        }
    }

    async function handleDelete(service: BackendService) {
        if (!confirm(`Are you sure you want to delete "${service.name}"?`)) return

        error = null
        try {
            await api.deleteService(service.id)
            toasts.add('success', `Service "${service.name}" deleted successfully`)
            await loadServices()
        } catch (err) {
            const errorMsg = 'Failed to delete service'
            error = errorMsg
            toasts.add('error', errorMsg)
            console.error(err)
        }
    }
</script>

{#snippet NameCell({service}: {service: BackendService})}
    <div>
        <div class="font-semibold">{service.name}</div>
        {#if service.description}
            <div class="text-sm text-muted-foreground mt-0.5">{service.description}</div>
        {/if}
    </div>
{/snippet}

{#snippet BaseUrlCell({service}: {service: BackendService})}
    <code class="text-sm bg-muted px-2 py-1 rounded">{service.base_url}</code>
    {#if service.health_check_url}
        <div class="text-xs text-muted-foreground mt-1">
            Health: <code class="bg-muted px-1 py-0.5 rounded">{service.health_check_url}</code>
        </div>
    {/if}
{/snippet}

{#snippet HealthCell({service, health}: {service: BackendService, health: ServiceHealth | undefined})}
    {#if health}
        <div class="flex flex-col gap-1">
            <Badge variant={health.is_healthy ? 'default' : 'destructive'} class="w-fit">
                {health.is_healthy ? 'Healthy' : 'Unhealthy'}
            </Badge>
            <span class="text-xs text-muted-foreground">{health.status_message}</span>
        </div>
    {:else if service.health_check_url}
        <Badge variant="secondary" class="w-fit">Checking...</Badge>
    {:else}
        <Badge variant="outline" class="w-fit">No health check</Badge>
    {/if}
{/snippet}

{#snippet ActiveCell({service}: {service: BackendService})}
    <Badge variant={service.is_active ? 'default' : 'secondary'} class="w-fit">
        {service.is_active ? 'Active' : 'Inactive'}
    </Badge>
{/snippet}

{#snippet ActionsCell({service}: {service: BackendService})}
    <div class="flex items-center justify-center gap-2">
        <Button
            variant="ghost"
            size="sm"
            onclick={() => openEditModal(service)}
            class="h-8 px-3 text-xs font-medium"
        >
            Edit
        </Button>
        <Button
            variant="ghost"
            size="sm"
            onclick={() => handleDelete(service)}
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
            <h1 class="text-3xl font-bold tracking-tight">Backend Services</h1>
            <p class="text-muted-foreground mt-1">Manage your backend services and configurations</p>
        </div>
        <Button onclick={openCreateModal}>
            <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
            </svg>
            New Service
        </Button>
    </div>

    <!-- Search & Health Check -->
    <div class="flex flex-col sm:flex-row items-start sm:items-center gap-4 justify-between">
        <div class="relative flex-1 max-w-md w-full">
            <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" fill="none"
                 stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
            </svg>
            <Input
                    type="search"
                    placeholder="Search services..."
                    bind:value={globalFilter}
                    class="pl-10"
            />
        </div>

        <div class="flex items-center gap-3">
            {#if lastChecked}
        <span class="text-sm text-muted-foreground">
          Last checked: {getTimeAgo(lastChecked)}
        </span>
            {/if}
            <Button
                    variant="outline"
                    size="sm"
                    onclick={refreshHealthStatus}
                    disabled={healthLoading}
            >
                <svg class="w-4 h-4 mr-2 {healthLoading ? 'animate-spin' : ''}" fill="none" stroke="currentColor"
                     viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                          d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
                </svg>
                {healthLoading ? 'Checking...' : 'Refresh Health'}
            </Button>
        </div>
    </div>

    <!-- Error Alert -->
    {#if error}
        <Alert.Root variant="destructive">
            <Alert.Title>Error</Alert.Title>
            <Alert.Description>{error}</Alert.Description>
        </Alert.Root>
    {/if}

    <!-- Services Table -->
    <Card>
        <CardContent class="p-0">
            {#if loading}
                <div class="p-6 space-y-2">
                    {#each Array(5) as _}
                        <Skeleton class="h-12 w-full"/>
                    {/each}
                </div>
            {:else if services.length === 0}
                <div class="text-center py-12">
                    <svg class="mx-auto h-12 w-12 text-muted-foreground" fill="none" stroke="currentColor"
                         viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4"/>
                    </svg>
                    <h3 class="mt-4 text-lg font-semibold">No services found</h3>
                    <p class="text-muted-foreground mt-2">
                        {globalFilter ? 'Try adjusting your search query' : 'Get started by creating your first service'}
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
                                                <FlexRender content={header.column.columnDef.header}
                                                            context={header.getContext()}/>
                                            {/if}
                                        </Table.Head>
                                    {/each}
                                </Table.Row>
                            {/each}
                        </Table.Header>
                        <Table.Body>
                            {#each table.getRowModel().rows as row}
                                {@const service = row.original}
                                {@const health = getServiceHealth(service.id)}
                                <Table.Row>
                                    <Table.Cell class="px-6 py-3">
                                        {@render NameCell({service})}
                                    </Table.Cell>
                                    <Table.Cell class="px-6 py-3">
                                        {@render BaseUrlCell({service})}
                                    </Table.Cell>
                                    <Table.Cell class="px-6 py-3">
                                        {@render HealthCell({service, health})}
                                    </Table.Cell>
                                    <Table.Cell class="px-6 py-3">
                                        {@render ActiveCell({service})}
                                    </Table.Cell>
                                    <Table.Cell class="px-6 py-3">
                                        <span class="text-sm text-muted-foreground">{service.timeout_ms}ms</span>
                                    </Table.Cell>
                                    <Table.Cell class="px-6 py-3 text-right">
                                        {@render ActionsCell({service})}
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
                        of {table.getFilteredRowModel().rows.length} services
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
    <Dialog.Content class="sm:max-w-[600px]">
        <Dialog.Header>
            <Dialog.Title>{editingService ? 'Edit Service' : 'Create New Service'}</Dialog.Title>
            <Dialog.Description>
                {editingService ? 'Update the service configuration' : 'Add a new backend service to your gateway'}
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
                            placeholder="my-service"
                            required
                    />
                </div>

                <div class="grid gap-2">
                    <Label for="description">Description</Label>
                    <Input
                            id="description"
                            type="text"
                            bind:value={formData.description}
                            placeholder="Service description"
                    />
                </div>

                <div class="grid gap-2">
                    <Label for="base_url">
                        Base URL <span class="text-destructive">*</span>
                    </Label>
                    <Input
                            id="base_url"
                            type="url"
                            bind:value={formData.base_url}
                            placeholder="http://localhost:3000"
                            required
                    />
                </div>

                <div class="grid gap-2">
                    <Label for="health_check_url">Health Check URL</Label>
                    <Input
                            id="health_check_url"
                            type="url"
                            bind:value={formData.health_check_url}
                            placeholder="http://localhost:3000/health"
                    />
                </div>

                <div class="grid grid-cols-2 gap-4">
                    <div class="grid gap-2">
                        <Label for="interval">Health Check Interval (seconds)</Label>
                        <Input
                                id="interval"
                                type="number"
                                bind:value={formData.health_check_interval_seconds}
                                min="10"
                                max="3600"
                        />
                    </div>

                    <div class="grid gap-2">
                        <Label for="timeout">Timeout (ms)</Label>
                        <Input
                                id="timeout"
                                type="number"
                                bind:value={formData.timeout_ms}
                                min="100"
                                max="60000"
                        />
                    </div>
                </div>
            </div>

            <Dialog.Footer>
                <Button type="button" variant="outline" onclick={() => showModal = false}>
                    Cancel
                </Button>
                <Button type="submit">
                    {editingService ? 'Update' : 'Create'}
                </Button>
            </Dialog.Footer>
        </form>
    </Dialog.Content>
</Dialog.Root>
