<!-- src/routes/config-list/+page.svelte -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { configListStore } from '$lib/stores/configListStore';
	import { Plus, Search, FileText, Trash2, CalendarDays } from 'lucide-svelte';
	import { AlertDialog } from '$lib/components/ui/alert-dialog';
	import AreYouSure from '@/components/AreYouSure.svelte';

	let searchTerm = '';
	let deleteConfigId: number | null = null;
	let showDeleteDialog = false;

	onMount(() => {
		configListStore.loadConfigs();
	});

	$: if (searchTerm) {
		configListStore.loadConfigs(searchTerm);
	}

	async function handleDelete(id: number) {
		await configListStore.deleteConfig(id);
		deleteConfigId = null;
	}
</script>

<div class="mx-auto max-w-7xl p-6">
	<!-- Header -->
	<div class="mb-6 flex items-center justify-between">
		<h1 class="text-2xl font-bold">Configurations</h1>
		<a
			href="/editor"
			class="flex items-center gap-2 rounded-lg bg-blue-500 px-4 py-2 text-white hover:bg-blue-600"
		>
			<Plus class="h-4 w-4" />
			New Configuration
		</a>
	</div>

	<!-- Search Bar -->
	<div class="relative mb-6">
		<Search class="absolute left-3 top-1/2 h-5 w-5 -translate-y-1/2 transform text-gray-400" />
		<input
			type="text"
			placeholder="Search configurations..."
			bind:value={searchTerm}
			class="w-full rounded-lg border py-2 pl-10 pr-4 focus:border-blue-500 focus:ring-2 focus:ring-blue-500"
		/>
	</div>

	<!-- Configurations List -->
	{#if $configListStore.loading}
		<div class="py-12 text-center">
			<div
				class="mx-auto h-8 w-8 animate-spin rounded-full border-4 border-blue-500 border-t-transparent"
			/>
			<p class="mt-4 text-gray-600">Loading configurations...</p>
		</div>
	{:else if $configListStore.configs.length === 0}
		<div class="rounded-lg border-2 border-dashed bg-white py-12 text-center">
			<FileText class="mx-auto h-12 w-12 text-gray-400" />
			<h3 class="mt-4 text-lg font-medium text-gray-900">No configurations found</h3>
			<p class="mt-2 text-gray-500">Get started by creating a new configuration.</p>
			<a
				href="/editor"
				class="mt-4 inline-flex items-center rounded-md bg-blue-500 px-4 py-2 text-white hover:bg-blue-600"
			>
				<Plus class="mr-2 h-4 w-4" />
				Create Configuration
			</a>
		</div>
	{:else}
		<div class="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
			{#each $configListStore.configs as config}
				<div class="overflow-hidden rounded-lg border bg-white transition-shadow hover:shadow-md">
					<div class="p-6">
						<div class="flex items-start justify-between">
							<h3 class="text-lg font-medium text-gray-900">{config.name}</h3>
							<div class="flex items-center gap-2">
								<a
									href={`/editor/${config.id}`}
									class="rounded-full p-1 text-gray-400 hover:text-blue-500"
								>
									<FileText class="h-4 w-4" />
								</a>
								<button
									class="rounded-full p-1 text-gray-400 hover:text-red-500"
									on:click={() => (deleteConfigId = config.id)}
								>
									<Trash2 class="h-4 w-4" />
								</button>
							</div>
						</div>

						<div class="mt-4 flex items-center text-sm text-gray-500">
							<CalendarDays class="mr-2 h-4 w-4" />
							Updated {new Date(config.updated_at).toLocaleDateString()}
						</div>

						<div class="mt-4 flex justify-end">
							<a href={`/editor/${config.id}`} class="text-sm text-blue-500 hover:text-blue-600">
								Edit Configuration
							</a>
						</div>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<!-- Delete Confirmation Dialog -->
<AreYouSure
	title="Delete Configuration"
	message="Are you sure you want to delete this configuration? This action cannot be undone."
	open={showDeleteDialog}
	onConfirm={() => handleDelete(deleteConfigId)}
/>
<!-- <AlertDialog open={deleteConfigId !== null}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Delete Configuration</AlertDialog.Title>
      <AlertDialog.Description>
        Are you sure you want to delete this configuration? This action cannot be undone.
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel on:click={() => deleteConfigId = null}>
        Cancel
      </AlertDialog.Cancel>
      <AlertDialog.Action 
        class="bg-red-500 hover:bg-red-600"
        on:click={() => {
          if (deleteConfigId) handleDelete(deleteConfigId);
        }}
      >
        Delete
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog> -->
