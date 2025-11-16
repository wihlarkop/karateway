<script lang="ts">
  import { page } from '$app/stores'
  import * as Sidebar from '$lib/components/ui/sidebar'
  import { COPYRIGHT_YEAR, TEAM_NAME } from '$lib/version'

  interface NavItem {
    path: string
    label: string
    icon: string
    description?: string
  }

  interface Props {
    navItems: NavItem[]
  }

  let { navItems }: Props = $props()

  const isActive = (path: string) => $page.url.pathname === path
</script>

<Sidebar.Root>
  <Sidebar.Header>
    <div class="flex items-center gap-2 px-2">
      <div class="w-8 h-8 bg-gradient-to-br from-sidebar-primary to-sidebar-primary/70 rounded-lg flex items-center justify-center">
        <svg class="w-5 h-5 text-sidebar-primary-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>
        </svg>
      </div>
      <div class="flex flex-col">
        <h1 class="text-lg font-bold tracking-tight">Karateway</h1>
        <p class="text-xs text-sidebar-foreground/70">API Gateway Admin</p>
      </div>
    </div>
  </Sidebar.Header>

  <Sidebar.Content>
    <Sidebar.Group>
      <Sidebar.GroupLabel>Navigation</Sidebar.GroupLabel>
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          {#each navItems as item (item.path)}
            <Sidebar.MenuItem>
              <Sidebar.MenuButton isActive={isActive(item.path)} asChild>
                {#snippet child({ props })}
                  <a href={item.path} {...props} class="flex items-center gap-3">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={item.icon}/>
                    </svg>
                    <div class="flex flex-col">
                      <span class="font-medium">{item.label}</span>
                      {#if item.description}
                        <span class="text-xs text-sidebar-foreground/60">{item.description}</span>
                      {/if}
                    </div>
                  </a>
                {/snippet}
              </Sidebar.MenuButton>
            </Sidebar.MenuItem>
          {/each}
        </Sidebar.Menu>
      </Sidebar.GroupContent>
    </Sidebar.Group>
  </Sidebar.Content>

  <Sidebar.Footer>
    <Sidebar.Separator />
    <div class="px-3 py-2 text-xs text-sidebar-foreground/60 text-center">
      © {COPYRIGHT_YEAR} {TEAM_NAME}
    </div>
  </Sidebar.Footer>
</Sidebar.Root>
