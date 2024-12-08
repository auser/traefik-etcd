<script lang="ts">
	import { pageStore } from '$lib/stores/pageStore';
	import { configStore } from '$lib/stores/configStore';
	import ConfigSidebar from '$lib/components/layouts/config/ConfigSidebar.svelte';
	import { ChevronLeft, Save } from 'lucide-svelte';

	export let showSaveButton = true;
</script>

<div class="flex h-screen bg-gray-50">
	<!-- Left Sidebar -->
	<ConfigSidebar {showSaveButton} />

	<!-- Main Content -->
	<div class="flex flex-1 flex-col">
		<header class="border-b bg-white px-6 py-4">
			<div class="flex items-center justify-between">
				<div class="flex items-center space-x-4">
					<a href="/config-list" class="text-gray-600 hover:text-gray-900">
						<ChevronLeft class="h-5 w-5" />
					</a>
					<h2 class="text-xl font-semibold">
						{$configStore.configName || 'New Configuration'}
					</h2>
					{#if $configStore.currentVersion}
						<span class="rounded bg-blue-100 px-2 py-1 text-sm text-blue-800">
							v{$configStore.currentVersion}
						</span>
					{/if}
				</div>

				{#if showSaveButton}
					<div class="flex items-center space-x-3">
						<button
							class="px-4 py-2 text-gray-600 hover:text-gray-900"
							on:click={() => history.back()}
						>
							Cancel
						</button>
						<button
							class="flex items-center rounded bg-blue-500 px-4 py-2 text-white hover:bg-blue-600 disabled:opacity-50"
							disabled={!$configStore.hasUnsavedChanges}
							on:click={() => configStore.save()}
						>
							<Save class="mr-2 h-4 w-4" />
							Save Changes
						</button>
					</div>
				{/if}
			</div>
		</header>

		<main class="border-b px-6 py-4">
			<div class="mx-auto max-w-4xl bg-white px-6 py-4">
				<slot />
			</div>
		</main>
	</div>
</div>
