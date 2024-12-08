<script lang="ts">
	import { configStore } from '$lib/stores/configStore';
	import { pageStore } from '$lib/stores/pageStore';
	import { onMount } from 'svelte';
	import { Save, ArrowLeft } from 'lucide-svelte';
	import type { HostConfig } from '$lib/types';
	import HostOptions from '$lib/components/editor/hosts/HostOptions.svelte';
	import DeploymentSection from '$lib/components/editor/hosts/DeploymentSection.svelte';
	import MiddlewareSection from '$lib/components/editor/hosts/MiddlewareSection.svelte';
	import PathSection from '$lib/components/editor/hosts/PathSection.svelte';
	import { goto } from '$app/navigation';

	export let hostId: string | undefined = undefined;

	let host: HostConfig = {
		domain: '',
		paths: [],
		deployments: {},
		middlewares: [],
		forwardHost: false
	};

	onMount(() => {
		// Set page actions
		pageStore.setTitle(hostId ? 'Edit Host' : 'Add Host');
		pageStore.setActions([
			{
				label: 'Save Host',
				icon: Save,
				onClick: saveHost
			}
		]);

		// Load existing host if editing
		if (hostId && $configStore.currentConfig) {
			const existingHost = $configStore.currentConfig.hosts[parseInt(hostId)];
			if (existingHost) {
				host = { ...existingHost };
			}
		}
	});

	async function saveHost() {
		const currentConfig = $configStore.currentConfig || { hosts: [] };
		let hosts = [...currentConfig.hosts];

		if (hostId) {
			// Update existing host
			hosts[parseInt(hostId)] = host;
		} else {
			// Add new host
			hosts = [...hosts, host];
		}

		await configStore.updateConfig({
			...currentConfig,
			hosts,
			rulePrefix: currentConfig.rulePrefix || '',
			etcd: currentConfig.etcd || {}
		});

		goto('/editor/hosts');
	}
</script>

<div class="space-y-6">
	<!-- Back button -->
	<button
		class="flex items-center text-gray-600 hover:text-gray-900"
		on:click={() => goto('/editor/hosts')}
	>
		<ArrowLeft class="mr-2 h-4 w-4" />
		Back to Hosts
	</button>

	<!-- Editor Card -->
	<div class="rounded-lg border bg-white">
		<!-- Host Header -->
		<div class="bg-gray-50 p-4">
			<label class="block">
				<span class="mb-1 block text-sm font-medium text-gray-700">Domain Name</span>
				<input
					type="text"
					class="w-full rounded-md border-gray-300"
					placeholder="Enter domain name"
					bind:value={host.domain}
				/>
			</label>
		</div>

		<!-- Host Configuration -->
		<div class="space-y-6 border-t p-4">
			<!-- Host Options -->
			<section class="space-y-2">
				<h3 class="text-sm font-medium text-gray-700">Host Options</h3>
				<HostOptions {host} onChange={(updatedHost) => (host = updatedHost)} />
			</section>

			<!-- Paths -->
			<section class="space-y-2">
				<h3 class="text-sm font-medium text-gray-700">Paths</h3>
				<PathSection paths={host.paths} onChange={(paths) => (host = { ...host, paths })} />
			</section>

			<!-- Deployments -->
			<section class="space-y-2">
				<h3 class="text-sm font-medium text-gray-700">Deployments</h3>
				<DeploymentSection
					deployments={host.deployments}
					onChange={(deployments) => (host = { ...host, deployments })}
				/>
			</section>

			<!-- Middlewares -->
			<section class="space-y-2">
				<h3 class="text-sm font-medium text-gray-700">Middlewares</h3>
				<MiddlewareSection
					middlewares={host.middlewares}
					onChange={(middlewares) => (host = { ...host, middlewares })}
				/>
			</section>
		</div>
	</div>
</div>
