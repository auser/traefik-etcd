<!-- src/routes/editor/+page.svelte -->
<script lang="ts">
  import { page } from '$app/stores';
  import { FileText, Server, Globe, Shield } from 'lucide-svelte';

  const editorSections = [
    {
      title: "ETCD Configuration",
      description: "Configure ETCD connection settings and TLS",
      icon: Server,
      href: "/editor/etcd"
    },
    {
      title: "Host Configuration",
      description: "Manage domains, paths, and deployments",
      icon: Globe,
      href: "/editor/hosts"
    },
    {
      title: "Middleware",
      description: "Configure request/response modifications",
      icon: Shield,
      href: "/editor/middleware"
    },
    {
      title: "Services",
      description: "Define backend services and routing",
      icon: FileText,
      href: "/editor/services"
    }
  ];
</script>

<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
  {#each editorSections as section}
    <a 
      href={section.href}
      class="group p-6 bg-white rounded-lg border hover:shadow-md transition-all"
      class:bg-gray-50={$page.url.pathname === section.href}
    >
      <div class="flex items-center gap-4">
        <div class="p-2 bg-blue-50 rounded-lg text-blue-500">
          <svelte:component this={section.icon} class="w-6 h-6" />
        </div>
        <div>
          <h3 class="text-lg font-semibold text-gray-900">
            {section.title}
          </h3>
          <p class="mt-1 text-sm text-gray-600">
            {section.description}
          </p>
        </div>
      </div>
    </a>
  {/each}
</div>