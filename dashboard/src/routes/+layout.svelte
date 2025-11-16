<script lang="ts">
  import '../app.css'
  import { page } from '$app/stores'

  // Components
  import { Toaster } from '$lib/components/ui/sonner'
  import * as Sidebar from '$lib/components/ui/sidebar'
  import AppSidebar from '$lib/components/AppSidebar.svelte'
  import TopBar from '$lib/components/TopBar.svelte'

  const navItems = [
    {
      path: '/',
      label: 'Dashboard',
      icon: 'M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6',
      description: 'Overview & Stats'
    },
    {
      path: '/services',
      label: 'Services',
      icon: 'M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01',
      description: 'Backend Services'
    },
    {
      path: '/routes',
      label: 'Routes',
      icon: 'M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z',
      description: 'API Routes'
    },
    {
      path: '/rate-limits',
      label: 'Rate Limits',
      icon: 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z',
      description: 'Rate Limiting'
    },
    {
      path: '/whitelist',
      label: 'Whitelist',
      icon: 'M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z',
      description: 'Access Control'
    },
    {
      path: '/audit-logs',
      label: 'Audit Logs',
      icon: 'M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z',
      description: 'Security Events'
    },
  ]
</script>

<Sidebar.Provider>
  <AppSidebar {navItems} />
  <Sidebar.Inset>
    <!-- TopBar Component -->
    <TopBar title={navItems.find(item => item.path === $page.url.pathname)?.label || 'Dashboard'} />

    <!-- Page Content -->
    <div class="p-4 lg:p-8">
      <slot />
    </div>
  </Sidebar.Inset>

  <!-- Toast Notifications -->
  <Toaster />
</Sidebar.Provider>
