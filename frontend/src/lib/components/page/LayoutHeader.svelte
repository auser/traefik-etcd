<script lang="ts">
	import type { PageAction } from '$lib/stores/pageStore';
	export let title: string;
	export let actions: PageAction[] = [];
	export let lastSavedMessage: string | undefined;
</script>

<header>
	<div class="flex items-center justify-between">
		<div class="flex items-center space-x-4">
			<h2 class="text-xl font-semibold">{title}</h2>
		</div>
		{#if lastSavedMessage}
			<span class="save-status">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="status-icon"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M5 13l4 4L19 7"
					/>
				</svg>
				{lastSavedMessage}
			</span>
		{/if}
		{#if actions.length > 0}
			<div class="flex items-center space-x-4">
				{#each actions as action}
					<button
						class="flex items-center gap-2 rounded-md bg-blue-500 px-3 py-2 text-sm text-white hover:bg-blue-600"
						on:click={action.onClick}
					>
						{#if action.icon}
							<svelte:component this={action.icon} class="h-4 w-4" />
						{/if}

						{action.label}
					</button>
				{/each}
			</div>
		{:else}
			<button class="back-btn" on:click={() => (window.location.href = '/config-list')}>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="btn-icon"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M10 19l-7-7m0 0l7-7m-7 7h18"
					/>
				</svg>
				Back to List
			</button>
		{/if}
	</div>
</header>

<style>
	.status-icon,
	.btn-icon,
	
	.back-btn {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 1rem;
		border: 1px solid #e2e8f0;
		border-radius: 0.375rem;
		background: white;
		color: #4a5568;
		font-size: 0.875rem;
		cursor: pointer;
		transition: all 0.2s;
	}

	.back-btn:hover {
		background: #f7fafc;
		border-color: #cbd5e0;
	}
</style>
