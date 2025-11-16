<script lang="ts">
    import * as Sidebar from '$lib/components/ui/sidebar'
    import * as DropdownMenu from '$lib/components/ui/dropdown-menu'
    import { Separator } from '$lib/components/ui/separator'

    interface Props {
        title: string
    }

    let {title}: Props = $props()

    function handleLogout() {
        // TODO: Implement logout functionality
        console.log('Logout clicked')
    }

    function handleProfile() {
        // TODO: Navigate to profile page
        console.log('Profile clicked')
    }

    function handleSettings() {
        // TODO: Navigate to settings page
        console.log('Settings clicked')
    }
</script>

<header class="sticky top-0 z-30 border-b bg-background/80 backdrop-blur-xl supports-[backdrop-filter]:bg-background/60">
    <div class="flex h-16 items-center justify-between px-4 lg:px-8">
        <!-- Left Section -->
        <div class="flex items-center gap-4">
            <!-- Sidebar Toggle -->
            <Sidebar.Trigger />

            <!-- Page Title -->
            <div>
                <h2 class="text-xl font-bold tracking-tight bg-gradient-to-r from-foreground to-foreground/70 bg-clip-text">
                    {title}
                </h2>
            </div>
        </div>

        <!-- Right Section -->
        <div class="flex items-center gap-3">
            <!-- User Dropdown Menu -->
            <DropdownMenu.Root>
                <DropdownMenu.Trigger asChild>
                    {#snippet child({ props })}
                        <button
                            {...props}
                            class="hidden sm:flex items-center gap-3 px-3 py-1.5 rounded-lg bg-accent/50 border border-border/50 hover:bg-accent transition-colors cursor-pointer"
                        >
                            <div class="w-8 h-8 rounded-full bg-gradient-to-br from-primary to-primary/70 flex items-center justify-center shadow-lg">
                                <svg class="w-4 h-4 text-primary-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                          d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"/>
                                </svg>
                            </div>
                            <div class="text-sm text-left">
                                <div class="font-semibold">Admin User</div>
                                <div class="text-xs text-muted-foreground">Administrator</div>
                            </div>
                            <svg class="w-4 h-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
                            </svg>
                        </button>
                    {/snippet}
                </DropdownMenu.Trigger>
                <DropdownMenu.Content align="end" class="w-56">
                    <DropdownMenu.Label>
                        <div class="flex flex-col space-y-1">
                            <p class="text-sm font-medium">Admin User</p>
                            <p class="text-xs text-muted-foreground">admin@karateway.local</p>
                        </div>
                    </DropdownMenu.Label>
                    <DropdownMenu.Separator />
                    <DropdownMenu.Group>
                        <DropdownMenu.Item onclick={handleProfile}>
                            <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                      d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"/>
                            </svg>
                            <span>Profile</span>
                        </DropdownMenu.Item>
                        <DropdownMenu.Item onclick={handleSettings}>
                            <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                      d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
                            </svg>
                            <span>Settings</span>
                        </DropdownMenu.Item>
                    </DropdownMenu.Group>
                    <DropdownMenu.Separator />
                    <DropdownMenu.Item onclick={handleLogout} class="text-destructive focus:text-destructive">
                        <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                  d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"/>
                        </svg>
                        <span>Logout</span>
                    </DropdownMenu.Item>
                </DropdownMenu.Content>
            </DropdownMenu.Root>
        </div>
    </div>
</header>
