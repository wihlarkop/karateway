<script lang="ts">
    import {onMount} from 'svelte'
    import {api} from '$lib/api'
    import type {ApiRoute, BackendService, CreateApiRouteRequest, HttpMethod} from '$lib/types'
    import {toasts} from '$lib/stores/toast'
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
    import {Button} from '$lib/components/ui/button'
    import {Input} from '$lib/components/ui/input'
    import {Label} from '$lib/components/ui/label'
    import {Badge} from '$lib/components/ui/badge'
    import * as Dialog from '$lib/components/ui/dialog'
    import * as Table from '$lib/components/ui/table'
    import * as Alert from '$lib/components/ui/alert'
    import {Card, CardContent} from '$lib/components/ui/card'
    import {Skeleton} from '$lib/components/ui/skeleton'

    let routes = $state<ApiRoute[]>([])
    let services = $state<BackendService[]>([])
    let loading = $state(true)
    let showModal = $state(false)
    let editingRoute = $state<ApiRoute | null>(null)
    let error = $state<string | null>(null)
    let globalFilter = $state('')
    let currentPage = $state(1)
    let pageSize = $state(10)
    let totalRoutes = $state(0)

    const httpMethods: HttpMethod[] = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS']

    // Pagination state for TanStack Table
    let pagination = $state<PaginationState>({
        pageIndex: currentPage - 1,
        pageSize: pageSize
    })

    // Form state
    let formData = $state<CreateApiRouteRequest>({
        path_pattern: '',
        method: 'GET',
        backend_service_id: '',
        strip_path_prefix: false,
        preserve_host_header: false,
        timeout_ms: 5000,
        priority: 100,
        metadata: {},
    })

    // TanStack Table column definitions
    const columns: ColumnDef<ApiRoute>[] = [
        {
            accessorKey: 'path_pattern',
            header: 'Path',
            cell: (info) => info.getValue()
        },
        {
            accessorKey: 'method',
            header: 'Method',
            cell: (info) => info.getValue()
        },
        {
            accessorKey: 'backend_service_id',
            header: 'Service',
            cell: (info) => getServiceName(info.getValue() as string)
        },
        {
            accessorKey: 'priority',
            header: 'Priority',
            cell: (info) => info.getValue()
        },
        {
            accessorKey: 'is_active',
            header: 'Status',
            cell: (info) => info.getValue() ? 'Active' : 'Inactive'
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
            return routes
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
        pageCount: Math.ceil((totalRoutes || 0) / pageSize)
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
            const [routesRes, servicesRes] = await Promise.all([
                api.getRoutes(currentPage, pageSize, globalFilter || undefined),
                api.getServices(1, 100)
            ])

            if (routesRes.success && routesRes.data) {
                routes = routesRes.data
                totalRoutes = routesRes.meta?.total_data || routes.length
            }
            if (servicesRes.success && servicesRes.data) {
                services = servicesRes.data
            }
        } catch (err) {
            error = 'Failed to load data'
            console.error(err)
        } finally {
            loading = false
        }
    }

    function openCreateModal() {
        editingRoute = null
        formData = {
            path_pattern: '',
            method: 'GET',
            backend_service_id: services[0]?.id || '',
            strip_path_prefix: false,
            preserve_host_header: false,
            timeout_ms: 5000,
            priority: 100,
            metadata: {},
        }
        showModal = true
    }

    function openEditModal(route: ApiRoute) {
        editingRoute = route
        formData = {
            path_pattern: route.path_pattern,
            method: route.method,
            backend_service_id: route.backend_service_id,
            strip_path_prefix: route.strip_path_prefix,
            preserve_host_header: route.preserve_host_header,
            timeout_ms: route.timeout_ms || 5000,
            priority: route.priority,
            metadata: route.metadata,
        }
        showModal = true
    }

    async function handleSubmit(e: SubmitEvent) {
        e.preventDefault()
        error = null
        try {
            if (editingRoute) {
                await api.updateRoute(editingRoute.id, formData)
                toasts.add('success', `Route "${formData.path_pattern}" updated successfully`)
            } else {
                await api.createRoute(formData)
                toasts.add('success', `Route "${formData.path_pattern}" created successfully`)
            }
            showModal = false
            await loadData()
        } catch (err) {
            const errorMsg = 'Failed to save route'
            error = errorMsg
            toasts.add('error', errorMsg)
            console.error(err)
        }
    }

    async function handleDelete(route: ApiRoute) {
        if (!confirm(`Are you sure you want to delete route "${route.path_pattern} ${route.method}"?`)) return

        error = null
        try {
            await api.deleteRoute(route.id)
            toasts.add('success', `Route "${route.path_pattern}" deleted successfully`)
            await loadData()
        } catch (err) {
            const errorMsg = 'Failed to delete route'
            error = errorMsg
            toasts.add('error', errorMsg)
            console.error(err)
        }
    }

    function getServiceName(serviceId: string): string {
        return services.find(s => s.id === serviceId)?.name || 'Unknown'
    }

    function getMethodVariant(method: HttpMethod): 'default' | 'secondary' | 'destructive' | 'outline' {
        switch (method) {
            case 'GET':
                return 'default'
            case 'POST':
                return 'secondary'
            case 'PUT':
            case 'PATCH':
                return 'outline'
            case 'DELETE':
                return 'destructive'
            default:
                return 'secondary'
        }
    }
</script>

{#snippet PathCell({route}: {route: ApiRoute})}
    <code class="text-sm font-mono bg-muted px-2 py-1 rounded">{route.path_pattern}</code>
{/snippet}

{#snippet MethodCell({route}: {route: ApiRoute})}
    <Badge variant={getMethodVariant(route.method)} class="w-fit">
        {route.method}
    </Badge>
{/snippet}

{#snippet StatusCell({route}: {route: ApiRoute})}
    <Badge variant={route.is_active ? 'default' : 'secondary'} class="w-fit">
        {route.is_active ? 'Active' : 'Inactive'}
    </Badge>
{/snippet}

{#snippet ActionsCell({route}: {route: ApiRoute})}
    <div class="flex items-center justify-center gap-2">
        <Button
            variant="ghost"
            size="sm"
            onclick={() => openEditModal(route)}
            class="h-8 px-3 text-xs font-medium"
        >
            Edit
        </Button>
        <Button
            variant="ghost"
            size="sm"
            onclick={() => handleDelete(route)}
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
            <h1 class="text-3xl font-bold tracking-tight">API Routes</h1>
            <p class="text-muted-foreground mt-1">Configure routing rules for your gateway</p>
        </div>
        <Button onclick={openCreateModal} disabled={services.length === 0}>
            <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
            </svg>
            New Route
        </Button>
    </div>

    <!-- Search -->
    <div class="relative flex-1 max-w-md w-full">
        <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" fill="none"
             stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
        </svg>
        <Input
                type="search"
                placeholder="Search routes..."
                bind:value={globalFilter}
                class="pl-10"
        />
    </div>

    <!-- No Services Warning -->
    {#if services.length === 0 && !loading}
        <Alert.Root variant="destructive">
            <Alert.Title>No Backend Services</Alert.Title>
            <Alert.Description>
                You need to create a backend service first before adding routes.
                <a href="/services" class="underline ml-1">Go to Services</a>
            </Alert.Description>
        </Alert.Root>
    {/if}

    <!-- Error Alert -->
    {#if error}
        <Alert.Root variant="destructive">
            <Alert.Title>Error</Alert.Title>
            <Alert.Description>{error}</Alert.Description>
        </Alert.Root>
    {/if}

    <!-- Routes Table -->
    <Card>
        <CardContent class="p-0">
            {#if loading}
                <div class="p-6 space-y-2">
                    {#each Array(5) as _}
                        <Skeleton class="h-12 w-full"/>
                    {/each}
                </div>
            {:else if routes.length === 0}
                <div class="text-center py-12">
                    <svg class="mx-auto h-12 w-12 text-muted-foreground" fill="none" stroke="currentColor"
                         viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4"/>
                    </svg>
                    <h3 class="mt-4 text-lg font-semibold">No routes found</h3>
                    <p class="text-muted-foreground mt-2">
                        {globalFilter ? 'Try adjusting your search query' : services.length === 0 ? 'Create a backend service first' : 'Get started by creating your first route'}
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
                                {@const route = row.original}
                                <Table.Row>
                                    <Table.Cell class="px-6 py-3">
                                        {@render PathCell({route})}
                                    </Table.Cell>
                                    <Table.Cell class="px-6 py-3">
                                        {@render MethodCell({route})}
                                    </Table.Cell>
                                    <Table.Cell class="px-6 py-3">
                                        <span class="text-sm">{getServiceName(route.backend_service_id)}</span>
                                    </Table.Cell>
                                    <Table.Cell class="px-6 py-3">
                                        <span class="text-sm text-muted-foreground">{route.priority}</span>
                                    </Table.Cell>
                                    <Table.Cell class="px-6 py-3">
                                        {@render StatusCell({route})}
                                    </Table.Cell>
                                    <Table.Cell class="px-6 py-3">
                                        {@render ActionsCell({route})}
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
                        of {table.getFilteredRowModel().rows.length} routes
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
            <Dialog.Title>{editingRoute ? 'Edit Route' : 'Create New Route'}</Dialog.Title>
            <Dialog.Description>
                {editingRoute ? 'Update the route configuration' : 'Add a new API route to your gateway'}
            </Dialog.Description>
        </Dialog.Header>

        <form onsubmit={handleSubmit} class="space-y-4">
            <div class="grid gap-4">
                <div class="grid gap-2">
                    <Label for="path">
                        Path Pattern <span class="text-destructive">*</span>
                    </Label>
                    <Input
                            id="path"
                            type="text"
                            bind:value={formData.path_pattern}
                            placeholder="/api/v1/users"
                            required
                    />
                </div>

                <div class="grid gap-2">
                    <Label for="method">
                        HTTP Method <span class="text-destructive">*</span>
                    </Label>
                    <select
                            id="method"
                            bind:value={formData.method}
                            required
                            class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                    >
                        {#each httpMethods as method}
                            <option value={method}>{method}</option>
                        {/each}
                    </select>
                    <p class="text-sm text-muted-foreground">HTTP method for this route</p>
                </div>

                <div class="grid gap-2">
                    <Label for="service">
                        Backend Service <span class="text-destructive">*</span>
                    </Label>
                    <select
                            id="service"
                            bind:value={formData.backend_service_id}
                            required
                            class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                    >
                        <option value="" disabled>Select a backend service</option>
                        {#each services as service}
                            <option value={service.id}>{service.name} - {service.target_url}</option>
                        {/each}
                    </select>
                    <p class="text-sm text-muted-foreground">Backend service to route requests to</p>
                </div>

                <div class="grid grid-cols-2 gap-4">
                    <div class="grid gap-2">
                        <Label for="priority">Priority</Label>
                        <Input
                                id="priority"
                                type="number"
                                bind:value={formData.priority}
                                min="0"
                                max="1000"
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

                <div class="flex items-center space-x-2">
                    <input
                            type="checkbox"
                            id="strip_path"
                            bind:checked={formData.strip_path_prefix}
                            class="rounded"
                    />
                    <Label for="strip_path" class="cursor-pointer">Strip path prefix</Label>
                </div>

                <div class="flex items-center space-x-2">
                    <input
                            type="checkbox"
                            id="preserve_host"
                            bind:checked={formData.preserve_host_header}
                            class="rounded"
                    />
                    <Label for="preserve_host" class="cursor-pointer">Preserve host header</Label>
                </div>
            </div>

            <Dialog.Footer>
                <Button type="button" variant="outline" onclick={() => showModal = false}>
                    Cancel
                </Button>
                <Button type="submit">
                    {editingRoute ? 'Update' : 'Create'}
                </Button>
            </Dialog.Footer>
        </form>
    </Dialog.Content>
</Dialog.Root>
