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
		},
		{
			label: 'Services',
			href: '/editor/services'
		}
	];
	export let showSaveButton = true;
</script>

<!-- Left Sidebar -->
<nav class="flex h-full w-64 flex-col items-start justify-between border-r bg-white">
	<div class="border-b p-4">
		<h1 class="text-xl font-semibold">TraefikCtl</h1>

		<div class="p-4">
			<div class="space-y-2">
				{#each sidebarElements as element}
					<a
						href={element.href}
						class="flex w-full items-center rounded p-2 text-left text-gray-700 hover:bg-gray-100"
						class:bg-gray-100={$page.url.pathname.includes(element.href)}
					>
						<Settings class="mr-2 h-4 w-4" />
						{element.label}
					</a>
				{/each}
			</div>
		</div>
	</div>
</nav>
