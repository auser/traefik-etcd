<script lang="ts">
	import { configStore } from '$lib/stores/configStore';
	import { Tabs, TabsContent, TabsTrigger } from '@/components/ui/tabs';
	import TabsList from '@/components/ui/tabs/tabs-list.svelte';
	import { DeploymentProtocol, DeploymentTarget, type ServiceConfig } from '@/types';
	import { onMount } from 'svelte';

	let service: ServiceConfig = {
		name: '',
		deployment: {
			name: '',
			target: DeploymentTarget.IP_AND_PORT,
			weight: 0,
			protocol: DeploymentProtocol.HTTP
		},
		passHostHeader: false
	};

	let serviceName = '';
	let isEditing = false;

	onMount(() => {
		// Initialize service if editing existing one
		const currentServices = $configStore.currentConfig?.config.services || {};
		console.log('currentServices', currentServices);
		if (serviceName && currentServices[serviceName]) {
			service = { ...currentServices[serviceName] };
			isEditing = true;
		}
	});

	function saveService() {
		const currentServices = $configStore.currentConfig?.config.services || {};

		console.log('currentServices', currentServices);
		// configStore.updateCurrentConfig({
		// 	content: {
		// 		...$configStore.currentConfig?.content,
		// 		http: {
		// 			...$configStore.currentConfig?.content.http,
		// 			services: {
		// 				...currentServices,
		// 				[serviceName]: service
		// 			}
		// 		}
		// 	}
		// });
	}

	function addServer() {
		console.log('service.servers', service);
		service.servers = [...service.servers, { url: '' }];
	}

	function removeServer(index: number) {
		console.log('service.servers', service.servers);
		service.servers = service.servers.filter((_, i) => i !== index);
	}
</script>

<div class="service-details">
	<div class="form-section">
		<h2 class="section-title">Service Details</h2>

		<div class="form-group">
			<label for="serviceName">Service Name</label>
			<input
				type="text"
				id="serviceName"
				bind:value={serviceName}
				placeholder="Enter service name"
				class="form-input"
			/>
		</div>

		<div class="servers-section">
			<h3 class="subsection-title">Configuration</h3>

			{#each service as server, index}
				<form class="">
					<div class="flex flex-col justify-between">
						<!-- <div class="w-1/2"> -->
						<div class="mt-10 grid grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-2">
							<div class="sm:col-span-1">
								<div
									class="rounded-md bg-white px-3 pb-1.5 pt-2.5 outline outline-1 -outline-offset-1 outline-gray-300 focus-within:outline focus-within:outline-2 focus-within:-outline-offset-2 focus-within:outline-indigo-600"
								>
									<label for="IP" class="block text-xs font-medium text-gray-900">IP</label>
									<input
										type="text"
										name="IP"
										id="IP"
										class="block w-full text-gray-900 placeholder:text-gray-400 focus:outline focus:outline-0 sm:text-sm/6"
										placeholder="192.168.1.1"
									/>
								</div>
							</div>

							<div class="sm:col-span-1">
								<div
									class="rounded-md bg-white px-3 pb-1.5 pt-2.5 outline outline-1 -outline-offset-1 outline-gray-300 focus-within:outline focus-within:outline-2 focus-within:-outline-offset-2 focus-within:outline-indigo-600"
								>
									<label for="port" class="block text-xs font-medium text-gray-900">Port</label>
									<input
										type="text"
										name="port"
										id="port"
										class="block w-full text-gray-900 placeholder:text-gray-400 focus:outline focus:outline-0 sm:text-sm/6"
										placeholder="80"
									/>
								</div>
							</div>
						</div>
						<div class="my-6 flex items-center justify-center">
							<small class="text-center text-gray-400"> Or </small>
						</div>
						<div class="sm:col-span-2">
							<div
								class="rounded-md bg-white px-3 pb-1.5 pt-2.5 outline outline-1 -outline-offset-1 outline-gray-300 focus-within:outline focus-within:outline-2 focus-within:-outline-offset-2 focus-within:outline-indigo-600"
							>
								<label for="hostname" class="block text-xs font-medium text-gray-900">Port</label>
								<input
									type="text"
									name="hostname"
									id="hostname"
									class="block w-full text-gray-900 placeholder:text-gray-400 focus:outline focus:outline-0 sm:text-sm/6"
									placeholder="example.com"
								/>
							</div>
						</div>
					</div>
				</form>
			{/each}

			<div class="flex justify-end gap-2">
				<button class="btn rm-btn bg-white text-red-500" on:click={removeServer}>
					Remove Server
				</button>
				<button class="btn add-btn" on:click={addServer}> Add Server </button>
			</div>
		</div>

		<div class="actions">
			<button class="save-btn" on:click={saveService}>
				{isEditing ? 'Update' : 'Create'} Service
			</button>
		</div>
	</div>
</div>

<style>
	.service-details {
		padding: 1.5rem;
		background: white;
		border-radius: 0.5rem;
		border: 1px solid #e2e8f0;
	}

	.section-title {
		font-size: 1.25rem;
		font-weight: 600;
		color: #1a202c;
		margin-bottom: 1.5rem;
	}

	.form-section {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.form-group {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.form-input {
		padding: 0.5rem;
		border: 1px solid #e2e8f0;
		border-radius: 0.375rem;
		width: 100%;
	}

	.servers-section {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.subsection-title {
		font-size: 1rem;
		font-weight: 500;
		color: #2d3748;
	}

	.server-entry {
		display: flex;
		gap: 0.5rem;
		align-items: center;
	}

	.btn {
		padding: 0.5rem;
		border: 1px dashed #9ca3af;
		border-radius: 0.375rem;
		cursor: pointer;
		width: fit-content;
	}

	.remove-btn {
		padding: 0.5rem;
		background: #fee2e2;
		color: #dc2626;
		border: none;
		border-radius: 0.375rem;
		cursor: pointer;
	}

	.rm-btn:hover {
		background: #fecaca;
	}

	.add-btn {
		padding: 0.5rem;
		background: #f3f4f6;
		border: 1px dashed #9ca3af;
		border-radius: 0.375rem;
		color: #4b5563;
		cursor: pointer;
		width: fit-content;
	}

	.actions {
		display: flex;
		justify-content: flex-end;
		margin-top: 1rem;
	}

	.save-btn {
		padding: 0.5rem 1rem;
		background: #3b82f6;
		color: white;
		border: none;
		border-radius: 0.375rem;
		cursor: pointer;
	}

	.save-btn:hover {
		background: #2563eb;
	}
</style>
