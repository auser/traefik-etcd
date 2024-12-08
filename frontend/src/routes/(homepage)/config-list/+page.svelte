<!-- src/routes/config-list/+page.svelte -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { configStore } from '$lib/stores/configStore';
	import NewResourceContainer from '$lib/components/page/NewResourceContainer.svelte';
	import ContentContainerLayout from '$lib/components/layouts/ContentContainerLayout.svelte';

	onMount(() => {
		configStore.loadConfigs();
	});
</script>

<ContentContainerLayout>
	<div class="config-list">
		<h1>Traefik Configurations</h1>

		<NewResourceContainer>
			<button
				class="new-config-btn"
				on:click={() => {
					configStore.setCurrentConfig({
						name: 'New Configuration',
						content: {}
					});
					window.location.href = '/editor/services';
				}}
			>
				Create New Configuration
			</button>
		</NewResourceContainer>

		{#if $configStore.configs.length === 0}
			<p>No configurations found. Create one to get started!</p>
		{:else}
			<div class="config-grid">
				{#each $configStore.configs as config}
					<div class="config-card">
						<h3>{config.name}</h3>
						<p>Last updated: {new Date(config.updated_at).toLocaleString()}</p>
						<div class="card-actions">
							<button
								on:click={() => {
									configStore.setCurrentConfig(config);
									window.location.href = '/editor/services';
								}}
							>
								Edit
							</button>
							<button class="delete" on:click={() => configStore.deleteConfig(config.id)}>
								Delete
							</button>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</div>
</ContentContainerLayout>

<style>
	.config-list {
		padding: 2rem;
	}

	.config-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
		gap: 1rem;
		margin-top: 2rem;
	}

	.config-card {
		border: 1px solid #ddd;
		border-radius: 8px;
		padding: 1rem;
		background: white;
	}

	.card-actions {
		display: flex;
		gap: 1rem;
		margin-top: 1rem;
	}

	button {
		padding: 0.5rem 1rem;
		border-radius: 4px;
		border: none;
		cursor: pointer;
	}

	button.delete {
		background: #ff4444;
		color: white;
	}

	.new-config-btn {
		background: #4caf50;
		color: white;
		font-size: 1.1rem;
		padding: 0.75rem 1.5rem;
	}
</style>
