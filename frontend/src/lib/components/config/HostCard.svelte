<script lang="ts">
	import { Trash2 } from 'lucide-svelte';
	import { slide } from 'svelte/transition';
	import type { HostConfig } from '$lib/types';
	import HostOptions from '$lib/components/editor/hosts/HostOptions.svelte';
	import DeploymentSection from '$lib/components/editor/hosts/DeploymentSection.svelte';
	import MiddlewareSection from '$lib/components/editor/hosts/MiddlewareSection.svelte';

	export let host: HostConfig;
	export let onChange: (host: HostConfig) => void;
	export let onDelete: () => void;
</script>

<div class="rounded-lg border bg-white" transition:slide>
	<!-- Host Header -->
	<div class="flex items-center justify-between bg-gray-50 p-4">
		<input
			type="text"
			class="flex-1 rounded border-none bg-transparent focus:ring-2 focus:ring-blue-500"
			placeholder="Enter domain"
			value={host.domain}
			on:input={(e) => onChange({ ...host, domain: e.currentTarget.value })}
		/>

		<button class="rounded p-1 text-gray-400 hover:text-red-500" on:click={onDelete}>
			<Trash2 class="h-4 w-4" />
		</button>
	</div>

	<!-- Host Configuration -->
	<div class="space-y-4 border-t p-4">
		<HostOptions {host} {onChange} />

		<DeploymentSection
			deployments={host.deployments}
			onChange={(deployments) => onChange({ ...host, deployments })}
		/>

		<MiddlewareSection
			middlewares={host.middlewares}
			onChange={(middlewares) => onChange({ ...host, middlewares })}
		/>
	</div>
</div>
