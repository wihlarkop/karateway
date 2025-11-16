<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import type { AuditLog } from '$lib/types';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import {
		Table,
		TableBody,
		TableCell,
		TableHead,
		TableHeader,
		TableRow
	} from '$lib/components/ui/table';
	import { Skeleton } from '$lib/components/ui/skeleton';

	let logs: AuditLog[] = [];
	let loading = true;
	let error: string | null = null;
	let total = 0;
	let limit = 50;
	let offset = 0;
	let autoRefresh = false;
	let refreshInterval: number;

	$: currentPage = Math.floor(offset / limit) + 1;
	$: totalPages = Math.ceil(total / limit);
	$: hasNextPage = offset + limit < total;
	$: hasPreviousPage = offset > 0;

	async function loadAuditLogs() {
		loading = true;
		error = null;

		try {
			const response = await api.getAuditLogs(limit, offset);
			logs = response.logs;
			total = response.total;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load audit logs';
			console.error('Error loading audit logs:', err);
		} finally {
			loading = false;
		}
	}

	function getSeverityColor(severity: string): string {
		switch (severity.toLowerCase()) {
			case 'critical':
				return 'destructive';
			case 'warning':
				return 'default';
			case 'info':
				return 'secondary';
			default:
				return 'outline';
		}
	}

	function getCategoryColor(category: string): string {
		switch (category.toLowerCase()) {
			case 'rate_limit':
				return 'default';
			case 'whitelist':
				return 'destructive';
			case 'authentication':
				return 'default';
			case 'admin':
				return 'secondary';
			default:
				return 'outline';
		}
	}

	function formatDate(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleString();
	}

	function formatEventType(eventType: string): string {
		return eventType.split('_').map(word =>
			word.charAt(0).toUpperCase() + word.slice(1)
		).join(' ');
	}

	function nextPage() {
		if (hasNextPage) {
			offset += limit;
			loadAuditLogs();
		}
	}

	function previousPage() {
		if (hasPreviousPage) {
			offset = Math.max(0, offset - limit);
			loadAuditLogs();
		}
	}

	function toggleAutoRefresh() {
		autoRefresh = !autoRefresh;
		if (autoRefresh) {
			refreshInterval = setInterval(loadAuditLogs, 10000); // Refresh every 10 seconds
		} else {
			clearInterval(refreshInterval);
		}
	}

	onMount(() => {
		loadAuditLogs();

		return () => {
			if (refreshInterval) {
				clearInterval(refreshInterval);
			}
		};
	});
</script>

<div class="container mx-auto py-6 space-y-4">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold">Security Audit Logs</h1>
			<p class="text-muted-foreground">View security events and access logs</p>
		</div>
		<div class="flex gap-2">
			<Button
				variant={autoRefresh ? 'default' : 'outline'}
				size="sm"
				on:click={toggleAutoRefresh}
			>
				<svg class={`h-4 w-4 mr-2 ${autoRefresh ? 'animate-spin' : ''}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
				</svg>
				{autoRefresh ? 'Auto-Refresh On' : 'Auto-Refresh Off'}
			</Button>
			<Button variant="outline" size="sm" on:click={loadAuditLogs} disabled={loading}>
				<svg class={`h-4 w-4 mr-2 ${loading ? 'animate-spin' : ''}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
				</svg>
				Refresh
			</Button>
		</div>
	</div>

	{#if error}
		<Card>
			<CardContent class="pt-6">
				<div class="text-center text-destructive">
					<p class="font-semibold">Error loading audit logs</p>
					<p class="text-sm">{error}</p>
				</div>
			</CardContent>
		</Card>
	{:else}
		<Card>
			<CardHeader>
				<div class="flex items-center justify-between">
					<CardTitle>Audit Logs ({total.toLocaleString()} total)</CardTitle>
					<div class="text-sm text-muted-foreground">
						Showing {offset + 1} - {Math.min(offset + limit, total)} of {total}
					</div>
				</div>
			</CardHeader>
			<CardContent>
				{#if loading}
					<div class="space-y-2">
						{#each Array(10) as _}
							<Skeleton class="h-12 w-full" />
						{/each}
					</div>
				{:else if logs.length === 0}
					<div class="text-center py-12 text-muted-foreground">
						<p>No audit logs found</p>
					</div>
				{:else}
					<div class="rounded-md border">
						<Table>
							<TableHeader>
								<TableRow>
									<TableHead class="w-[180px]">Timestamp</TableHead>
									<TableHead>Event</TableHead>
									<TableHead>Category</TableHead>
									<TableHead>Severity</TableHead>
									<TableHead>Client IP</TableHead>
									<TableHead>Method</TableHead>
									<TableHead>Path</TableHead>
									<TableHead class="w-[80px]">Status</TableHead>
									<TableHead class="max-w-md">Message</TableHead>
								</TableRow>
							</TableHeader>
							<TableBody>
								{#each logs as log (log.id)}
									<TableRow>
										<TableCell class="font-mono text-xs">
											{formatDate(log.created_at)}
										</TableCell>
										<TableCell>
											<span class="font-medium text-sm">
												{formatEventType(log.event_type)}
											</span>
										</TableCell>
										<TableCell>
											<Badge variant={getCategoryColor(log.event_category)}>
												{log.event_category}
											</Badge>
										</TableCell>
										<TableCell>
											<Badge variant={getSeverityColor(log.severity)}>
												{log.severity.toUpperCase()}
											</Badge>
										</TableCell>
										<TableCell class="font-mono text-xs">
											{log.client_ip || '-'}
										</TableCell>
										<TableCell>
											<span class="font-mono text-xs">
												{log.request_method || '-'}
											</span>
										</TableCell>
										<TableCell class="font-mono text-xs truncate max-w-xs" title={log.request_path}>
											{log.request_path || '-'}
										</TableCell>
										<TableCell>
											{#if log.status_code}
												<Badge
													variant={log.status_code >= 400 ? 'destructive' : 'secondary'}
												>
													{log.status_code}
												</Badge>
											{:else}
												-
											{/if}
										</TableCell>
										<TableCell class="max-w-md">
											<span class="text-sm truncate block" title={log.message}>
												{log.message}
											</span>
										</TableCell>
									</TableRow>
								{/each}
							</TableBody>
						</Table>
					</div>

					<!-- Pagination -->
					<div class="flex items-center justify-between mt-4">
						<div class="text-sm text-muted-foreground">
							Page {currentPage} of {totalPages}
						</div>
						<div class="flex gap-2">
							<Button
								variant="outline"
								size="sm"
								on:click={previousPage}
								disabled={!hasPreviousPage || loading}
							>
								<svg class="h-4 w-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
								</svg>
								Previous
							</Button>
							<Button
								variant="outline"
								size="sm"
								on:click={nextPage}
								disabled={!hasNextPage || loading}
							>
								Next
								<svg class="h-4 w-4 ml-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
								</svg>
							</Button>
						</div>
					</div>
				{/if}
			</CardContent>
		</Card>
	{/if}
</div>
