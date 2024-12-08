<script lang="ts">
	import { page } from '$app/stores';
	import { FileText, Server, Globe, Shield } from 'lucide-svelte';

	const editorSections = [
		{
			title: 'ETCD Configuration',
			description: 'Configure ETCD connection settings and TLS',
			icon: Server,
			href: '/editor/etcd'
		},
		{
			title: 'Host Configuration',
			description: 'Manage domains, paths, and deployments',
			icon: Globe,
			href: '/editor/hosts'
		},
		{
			title: 'Middleware',
			description: 'Configure request/response modifications',
			icon: Shield,
			href: '/editor/middleware'
		},
		{
			title: 'Services',
			description: 'Define backend services and routing',
			icon: FileText,
			href: '/editor/services'
		}
	];
</script>

<div class="grid grid-cols-1 gap-6 md:grid-cols-2">
	{#each editorSections as section}
		<a
			href={section.href}
			class="group rounded-lg border bg-white p-6 transition-all hover:shadow-md"
			class:bg-gray-50={$page.url.pathname === section.href}
		>
			<div class="flex items-center gap-4">
				<div class="rounded-lg bg-blue-50 p-2 text-blue-500">
					<svelte:component this={section.icon} class="h-6 w-6" />
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
