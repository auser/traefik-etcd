<script lang="ts">
	import { pageStore } from '$lib/stores/pageStore';
	import { configStore } from '$lib/stores/configStore';
	import ConfigSidebar from '$lib/components/layouts/config/ConfigSidebar.svelte';
	import { ChevronLeft, Save } from 'lucide-svelte';

	export const showSaveButton = true;

	console.log('$configStore.isDirty', $configStore);
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
{#if $configStore.isDirty}
	<div class="unsaved-changes">
		<svg
			xmlns="http://www.w3.org/2000/svg"
			class="warning-icon"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
		>
			<path
				stroke-linecap="round"
				stroke-linejoin="round"
				stroke-width="2"
				d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
			/>
		</svg>
		Unsaved changes
	</div>
{/if}

<style>
	.unsaved-changes {
		position: fixed;
		bottom: 1.5rem;
		right: 1.5rem;
		display: flex;
		align-items: center;
		gap: 0.5rem;
		background: #fff;
		color: #d69e2e;
		padding: 0.75rem 1.25rem;
		border-radius: 0.5rem;
		box-shadow:
			0 4px 6px -1px rgba(0, 0, 0, 0.1),
			0 2px 4px -1px rgba(0, 0, 0, 0.06);
		border: 1px solid #faf089;
	}
</style>
