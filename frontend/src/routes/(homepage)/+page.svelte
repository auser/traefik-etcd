<!-- src/routes/+page.svelte -->
<script lang="ts">
	import { configStore } from '$lib/stores/configStore';
	import { configListStore } from '$lib/stores/configListStore';
	import { Settings, GitBranch, FileJson, ArrowRight } from 'lucide-svelte';
	import { onMount } from 'svelte';
	import { draftStore } from '@/stores/draftStore';

	let recentConfigs: any[] = [];
	let loading = false;

	const quickActions = [
		{
			title: 'New Configuration',
			description: 'Create a new Traefik configuration from scratch',
			icon: FileJson,
			href: '/editor'
		},
		{
			title: 'Browse Configurations',
			description: 'View and manage existing configurations',
			icon: Settings,
			href: '/config-list'
		},
		{
			title: 'Documentation',
			description: 'Learn more about Traefik configuration',
			icon: GitBranch,
			href: 'https://docs.traefik.io',
			external: true
		}
	];

	onMount(async () => {
		loading = true;
		recentConfigs = await configListStore.loadConfigs();
		loading = false;
		draftStore.subscribe((draft) => {
			console.log('draft', draft);
		});
	});
</script>

<div class="min-h-screen bg-gray-50">
	<!-- Hero Section -->
	<div class="border-b bg-white">
		<div class="mx-auto max-w-7xl px-4 py-12 sm:px-6 lg:px-8">
			<h1 class="text-4xl font-bold text-gray-900">Welcome to TraefikCtl</h1>
			<p class="mt-2 text-xl text-gray-600">Manage your Traefik configurations with ease</p>
		</div>
	</div>

	<main class="mx-auto max-w-7xl px-4 py-12 sm:px-6 lg:px-8">
		<!-- Quick Actions -->
		<div class="mb-12 grid grid-cols-1 gap-6 md:grid-cols-3">
			{#each quickActions as action}
				<a
					href={action.href}
					target={action.external ? '_blank' : '_self'}
					class="group rounded-lg border bg-white p-6 transition-all hover:shadow-md"
				>
					<div class="flex items-center justify-between">
						<div class="rounded-lg bg-blue-50 p-2 text-blue-500">
							<svelte:component this={action.icon} class="h-6 w-6" />
						</div>
						<ArrowRight class="h-5 w-5 text-gray-400 group-hover:text-blue-500" />
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
		<section class="rounded-lg border bg-white p-6">
			<div class="mb-6 flex items-center justify-between">
				<h2 class="text-xl font-semibold">Recent Configurations</h2>
				<a href="/config-list" class="flex items-center text-blue-500 hover:text-blue-600">
					View all
					<ArrowRight class="ml-1 h-4 w-4" />
				</a>
			</div>

			{#if loading}
				<div class="py-8 text-center">
					<div
						class="mx-auto h-8 w-8 animate-spin rounded-full border-4 border-blue-500 border-t-transparent"
					></div>
					<p class="mt-4 text-gray-600">Loading configurations...</p>
				</div>
			{:else if recentConfigs.length === 0}
				<div class="rounded-lg border-2 border-dashed py-8 text-center">
					<p class="text-gray-500">No configurations yet</p>
					<a
						href="/editor"
						class="mt-4 inline-flex items-center rounded-md bg-blue-500 px-4 py-2 text-sm text-white hover:bg-blue-600"
					>
						Create your first configuration
					</a>
				</div>
			{:else}
				<div class="divide-y">
					{#each recentConfigs.slice(0, 5) as config}
						<a
							href={`/editor/${config.id}`}
							class="-mx-6 block px-6 py-4 first:-mt-6 last:-mb-6 hover:bg-gray-50"
						>
							<div class="flex items-center justify-between">
								<div>
									<h4 class="font-medium text-gray-900">{config.name}</h4>
									<p class="text-sm text-gray-500">
										Updated {new Date(config.updated_at).toLocaleDateString()}
									</p>
								</div>
								<ArrowRight class="h-5 w-5 text-gray-400" />
							</div>
						</a>
					{/each}
				</div>
			{/if}
		</section>
	</main>
</div>
