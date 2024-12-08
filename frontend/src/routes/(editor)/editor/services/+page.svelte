<script lang="ts">
	import { onMount } from 'svelte';
	import { configStore } from '$lib/stores/configStore';
	import ContentContainerLayout from '$lib/components/layouts/ContentContainerLayout.svelte';

	let lastSavedMessage = '';

	$: if ($configStore.lastSaved) {
		lastSavedMessage = `Last saved: ${$configStore.lastSaved.toLocaleString()}`;
	}

	onMount(() => {
		if (!$configStore.currentConfig) {
			window.location.href = '/config-list';
		}
	});

	function updateServices(services: Record<string, any>) {
		configStore.updateCurrentConfig({
			content: {
				...$configStore.currentConfig?.content,
				http: {
					...$configStore.currentConfig?.content.http,
					services
				}
			}
		});
	}
</script>

<ContentContainerLayout>
	<div class="services-editor">
		<div class="editor-header">
			<h1>Edit Services</h1>
			{#if lastSavedMessage}
				<span class="save-status">{lastSavedMessage}</span>
			{/if}
		</div>

		<!-- Add your service editor components here -->

		{#if $configStore.isDirty}
			<div class="unsaved-changes">Unsaved changes</div>
		{/if}
	</div>
</ContentContainerLayout>

<style>
	.services-editor {
		padding: 2rem;
	}

	.editor-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 2rem;
	}

	.save-status {
		color: #666;
		font-size: 0.9rem;
	}

	.unsaved-changes {
		position: fixed;
		bottom: 1rem;
		right: 1rem;
		background: #ff9800;
		color: white;
		padding: 0.5rem 1rem;
		border-radius: 4px;
	}
</style>
