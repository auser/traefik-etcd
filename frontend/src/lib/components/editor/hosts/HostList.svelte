<script lang="ts">
	import { Plus } from 'lucide-svelte';
	import HostCard from '$lib/components/config/HostCard.svelte';
	import type { HostConfig } from '$lib/types';
	import NewResourceContainer from '@/components/page/NewResourceContainer.svelte';

	export let hosts: HostConfig[] = [];
	export let onChange: (hosts: HostConfig[]) => void;

	function addHost() {
		onChange([
			...hosts,
			{
				domain: '',
				paths: [],
				deployments: {},
				middlewares: [],
				forwardHost: false
			}
		]);
	}

	function editHost(hostId: string) {
		goto(`/editor/hosts/${hostId}`);
	}
</script>

<div class="mt-6 space-y-6">
	{#if hosts.length === 0}
		<NewResourceContainer>
			<h3 class="text-lg font-medium text-gray-900">No hosts configured</h3>
			<p class="mt-2 text-gray-500">Add your first host to get started.</p>
			<button class="mt-4 text-blue-500 hover:text-blue-600" on:click={addHost}>
				Add Host Configuration
			</button>
		</NewResourceContainer>
	{:else}
		<div class="space-y-4">
			{#each hosts as host, index (index)}
				<HostCard
					{host}
					onChange={(updatedHost) => {
						const newHosts = [...hosts];
						newHosts[index] = updatedHost;
						onChange(newHosts);
					}}
					onDelete={() => {
						const newHosts = [...hosts];
						newHosts.splice(index, 1);
						onChange(newHosts);
					}}
				/>
			{/each}
		</div>
	{/if}
</div>
