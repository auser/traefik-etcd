<!-- src/routes/+page.svelte -->
<script lang="ts">
    import { configStore } from '$lib/stores/configStore';
    import { configListStore } from '$lib/stores/configListStore';
    import { Settings, GitBranch, FileJson, ArrowRight } from 'lucide-svelte';
    import { onMount } from 'svelte';
  
    let recentConfigs: any[] = [];
    let loading = false;
  
    const quickActions = [
      {
        title: "New Configuration",
        description: "Create a new Traefik configuration from scratch",
        icon: FileJson,
        href: "/editor"
      },
      {
        title: "Browse Configurations",
        description: "View and manage existing configurations",
        icon: Settings,
        href: "/config-list"
      },
      {
        title: "Documentation",
        description: "Learn more about Traefik configuration",
        icon: GitBranch,
        href: "https://docs.traefik.io",
        external: true
      }
    ];
  
    onMount(async () => {
      loading = true;
      recentConfigs = await configListStore.loadConfigs();
      loading = false;
    });
  </script>
  
  <div class="min-h-screen bg-gray-50">
    <!-- Hero Section -->
    <div class="bg-white border-b">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <h1 class="text-4xl font-bold text-gray-900">Welcome to TraefikCtl</h1>
        <p class="mt-2 text-xl text-gray-600">Manage your Traefik configurations with ease</p>
      </div>
    </div>
  
    <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
      <!-- Quick Actions -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
        {#each quickActions as action}
          <a 
            href={action.href}
            target={action.external ? "_blank" : "_self"}
            class="group p-6 bg-white rounded-lg border hover:shadow-md transition-all"
          >
            <div class="flex items-center justify-between">
              <div class="p-2 bg-blue-50 rounded-lg text-blue-500">
                <svelte:component this={action.icon} class="w-6 h-6" />
              </div>
              <ArrowRight class="w-5 h-5 text-gray-400 group-hover:text-blue-500" />
            </div>
            <h3 class="mt-4 text-lg font-semibold text-gray-900">
              {action.title}
            </h3>
            <p class="mt-2 text-sm text-gray-600">
              {action.description}
            </p>
          </a>
        {/each}
      </div>
  
      <!-- Recent Configurations -->
      <section class="bg-white rounded-lg border p-6">
        <div class="flex justify-between items-center mb-6">
          <h2 class="text-xl font-semibold">Recent Configurations</h2>
          <a 
            href="/config-list"
            class="text-blue-500 hover:text-blue-600 flex items-center"
          >
            View all
            <ArrowRight class="w-4 h-4 ml-1" />
          </a>
        </div>
  
        {#if loading}
          <div class="text-center py-8">
            <div class="animate-spin h-8 w-8 border-4 border-blue-500 border-t-transparent rounded-full mx-auto"></div>
            <p class="mt-4 text-gray-600">Loading configurations...</p>
          </div>
        {:else if recentConfigs.length === 0}
          <div class="text-center py-8 border-2 border-dashed rounded-lg">
            <p class="text-gray-500">No configurations yet</p>
            <a 
              href="/editor"
              class="mt-4 inline-flex items-center px-4 py-2 text-sm bg-blue-500 text-white rounded-md hover:bg-blue-600"
            >
              Create your first configuration
            </a>
          </div>
        {:else}
          <div class="divide-y">
            {#each recentConfigs.slice(0, 5) as config}
              <a 
                href={`/editor/${config.id}`}
                class="block py-4 hover:bg-gray-50 -mx-6 px-6 first:-mt-6 last:-mb-6"
              >
                <div class="flex items-center justify-between">
                  <div>
                    <h4 class="font-medium text-gray-900">{config.name}</h4>
                    <p class="text-sm text-gray-500">
                      Updated {new Date(config.updated_at).toLocaleDateString()}
                    </p>
                  </div>
                  <ArrowRight class="w-5 h-5 text-gray-400" />
                </div>
              </a>
            {/each}
          </div>
        {/if}
      </section>
    </main>
  </div>