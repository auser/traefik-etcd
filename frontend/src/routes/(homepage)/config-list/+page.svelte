<script lang="ts">
	import { onMount } from 'svelte';
	import { configStore } from '$lib/stores/configStore';
	import NewResourceContainer from '$lib/components/page/NewResourceContainer.svelte';
	import ContentContainerLayout from '$lib/components/layouts/ContentContainerLayout.svelte';
	import { goto } from '$app/navigation';

	onMount(() => {
		configStore.loadConfigs();
	});
</script>

<ContentContainerLayout>
	<div class="config-list">
		<div class="header">
			<div>
				<h1 class="title">Traefik Configurations</h1>
				<p class="subtitle">Manage your Traefik routing configurations</p>
			</div>

			<div>
				<button
					class="new-config-btn"
					on:click={() => {
						configStore.createNewConfig();
						goto('/editor/services');
					}}
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="icon"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M12 4v16m8-8H4"
						/>
					</svg>
					Create New Configuration
				</button>
			</div>
		</div>

		{#if $configStore.configs.length === 0}
			<div class="empty-state">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="empty-icon"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M9 13h6m-3-3v6m-9 1V7a2 2 0 012-2h6l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z"
					/>
				</svg>
				<p>No configurations found. Create one to get started!</p>
			</div>
		{:else}
			<div class="config-grid">
				{#each $configStore.configs as config}
					<div class="config-card">
						<div class="card-content">
							<h3 class="card-title">{config.name}</h3>
							<p class="card-date">
								Last updated: {new Date(config.updated_at || Date.now()).toLocaleString()}
							</p>
							<div class="card-stats">
								<div class="stat">
									<span class="stat-label">Services</span>
									<span class="stat-value"
										>{Object.keys(config.content?.http?.services || {}).length}</span
									>
								</div>
								<div class="stat">
									<span class="stat-label">Middlewares</span>
									<span class="stat-value"
										>{Object.keys(config.content?.http?.middlewares || {}).length}</span
									>
								</div>
							</div>
						</div>
						<div class="card-actions">
							<button
								class="edit-btn"
								on:click={() => {
									configStore.setCurrentConfig(config);
									goto('/editor/services');
								}}
							>
								Edit
							</button>
							<button class="delete-btn" on:click={() => configStore.deleteConfig(config?.id || 0)}>
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

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 3rem;
	}

	.title {
		font-size: 2rem;
		font-weight: 600;
		color: #1a202c;
		margin: 0;
	}

	.subtitle {
		color: #4a5568;
		margin-top: 0.5rem;
	}

	.new-config-btn {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		background: #4caf50;
		color: white;
		font-size: 1rem;
		font-weight: 500;
		padding: 0.75rem 1.5rem;
		border: none;
		border-radius: 0.5rem;
		cursor: pointer;
		transition: background-color 0.2s;
	}

	.new-config-btn:hover {
		background: #43a047;
	}

	.icon {
		width: 1.25rem;
		height: 1.25rem;
	}

	.empty-state {
		text-align: center;
		padding: 4rem 2rem;
		background: #f7fafc;
		border: 2px dashed #e2e8f0;
		border-radius: 0.75rem;
	}

	.empty-icon {
		width: 4rem;
		height: 4rem;
		color: #a0aec0;
		margin-bottom: 1rem;
	}

	.config-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
		gap: 1.5rem;
		margin-top: 2rem;
	}

	.config-card {
		border: 1px solid #e2e8f0;
		border-radius: 0.75rem;
		background: white;
		transition:
			transform 0.2s,
			box-shadow 0.2s;
	}

	.config-card:hover {
		transform: translateY(-2px);
		box-shadow:
			0 4px 6px -1px rgba(0, 0, 0, 0.1),
			0 2px 4px -1px rgba(0, 0, 0, 0.06);
	}

	.card-content {
		padding: 1.5rem;
	}

	.card-title {
		font-size: 1.25rem;
		font-weight: 600;
		color: #2d3748;
		margin: 0;
	}

	.card-date {
		color: #718096;
		font-size: 0.875rem;
		margin: 0.5rem 0 1rem;
	}

	.card-stats {
		display: flex;
		gap: 1rem;
		margin-top: 1rem;
	}

	.stat {
		flex: 1;
		background: #f7fafc;
		padding: 0.75rem;
		border-radius: 0.5rem;
		text-align: center;
	}

	.stat-label {
		display: block;
		font-size: 0.75rem;
		color: #4a5568;
		margin-bottom: 0.25rem;
	}

	.stat-value {
		font-size: 1.25rem;
		font-weight: 600;
		color: #2d3748;
	}

	.card-actions {
		display: flex;
		gap: 0.75rem;
		padding: 1rem 1.5rem;
		background: #f7fafc;
		border-top: 1px solid #e2e8f0;
		border-radius: 0 0 0.75rem 0.75rem;
	}

	.edit-btn,
	.delete-btn {
		flex: 1;
		padding: 0.5rem 1rem;
		border: none;
		border-radius: 0.375rem;
		font-weight: 500;
		cursor: pointer;
		transition: background-color 0.2s;
	}

	.edit-btn {
		background: #3182ce;
		color: white;
	}

	.edit-btn:hover {
		background: #2c5282;
	}

	.delete-btn {
		background: #fff;
		color: #e53e3e;
		border: 1px solid #e53e3e;
	}

	.delete-btn:hover {
		background: #e53e3e;
		color: white;
	}
</style>
