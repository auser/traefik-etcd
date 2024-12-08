<script lang="ts">
  import { page } from '$app/stores';
  import { Settings, Save, ChevronLeft } from 'lucide-svelte';
  import { configStore } from '$lib/stores/configStore';

  let sidebarElements = [
    {
      label: 'Host Configuration',
      href: '/editor/hosts'
    },
    {
      label: 'Middlewares',
      href: '/editor/middlewares'
    },
    {
      label: 'ETCD',
      href: '/editor/etcd'
    }
  ]
</script>

<!-- Left Sidebar -->
  <nav class="w-64 border-r bg-white flex flex-col items-start justify-between h-full">
    <div class="p-4 border-b">
      <h1 class="text-xl font-semibold">TraefikCtl</h1>
      
      <div class="p-4">
        <div class="space-y-2">
          {#each sidebarElements as element}
          <a 
          href={element.href}
          class="flex items-center w-full p-2 text-left text-gray-700 rounded hover:bg-gray-100"
          class:bg-gray-100={$page.url.pathname.includes(element.href)}
          >
          <Settings class="w-4 h-4 mr-2" />
          {element.label}
        </a>
        {/each}
      </div>
    </div>
    
  </nav>
