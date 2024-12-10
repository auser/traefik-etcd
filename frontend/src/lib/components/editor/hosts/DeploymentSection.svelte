<!-- src/lib/components/hosts/DeploymentSection.svelte -->
<script lang="ts">
	import { Plus, Trash2 } from 'lucide-svelte';
	import { DeploymentProtocol, DeploymentTarget, type DeploymentConfig } from '$lib/types';

	export let deployments: Record<string, DeploymentConfig> = {};
	export let onChange: (deployments: Record<string, DeploymentConfig>) => void;

	function addDeployment() {
		const newDeployments = { ...deployments };
		newDeployments[`deployment-${Object.keys(newDeployments).length + 1}`] = {
			name: '',
			target: DeploymentTarget.IP_AND_PORT,
			weight: 100,
			protocol: DeploymentProtocol.HTTP
		};
		onChange(newDeployments);
	}

	function updateDeployment(name: string, deployment: DeploymentConfig) {
		onChange({
			...deployments,
			[name]: deployment
		});
	}

	function removeDeployment(name: string) {
		const newDeployments = { ...deployments };
		delete newDeployments[name];
		onChange(newDeployments);
	}
</script>

<div class="space-y-2">
	<div class="flex items-center justify-between">
		<h4 class="font-medium">Deployments</h4>
		<button class="text-sm text-blue-500 hover:text-blue-600" on:click={addDeployment}>
			<Plus class="h-4 w-4" />
		</button>
	</div>

	{#if Object.keys(deployments).length === 0}
		<div class="rounded border border-dashed py-4 text-center">
			<p class="text-sm text-gray-500">No deployments configured</p>
		</div>
	{:else}
		<div class="space-y-3">
			{#each Object.entries(deployments) as [name, deployment]}
				<div class="space-y-3 rounded border p-3">
					<div class="flex items-center justify-between">
						<input
							type="text"
							class="border-none bg-transparent text-sm"
							value={name}
							placeholder="Deployment name"
						/>
						<button
							class="text-gray-400 hover:text-red-500"
							on:click={() => removeDeployment(name)}
						>
							<Trash2 class="h-4 w-4" />
						</button>
					</div>

					<div class="grid grid-cols-2 gap-3">
						<div>
							<label for={`deployment-${name}-ip`} class="text-sm text-gray-600">IP Address</label>
							<input
								id={`deployment-${name}-ip`}
								type="text"
								class="mt-1 w-full rounded-md border-gray-300 text-sm"
								value={deployment.ip}
								placeholder="127.0.0.1"
								on:input={(e) =>
									updateDeployment(name, {
										...deployment,
										ip: e.currentTarget.value
									})}
							/>
						</div>

						<div>
							<label for={`deployment-${name}-port`} class="text-sm text-gray-600">Port</label>
							<input
								id={`deployment-${name}-port`}
								type="number"
								class="mt-1 w-full rounded-md border-gray-300 text-sm"
								value={deployment.port}
								on:input={(e) =>
									updateDeployment(name, {
										...deployment,
										port: parseInt(e.currentTarget.value)
									})}
							/>
						</div>
					</div>

					<div class="grid grid-cols-2 gap-3">
						<div>
							<label for={`deployment-${name}-weight`} class="text-sm text-gray-600">Weight</label>
							<input
								id={`deployment-${name}-weight`}
								type="number"
								class="mt-1 w-full rounded-md border-gray-300 text-sm"
								min="0"
								max="100"
								value={deployment.weight}
								on:input={(e) =>
									updateDeployment(name, {
										...deployment,
										weight: parseInt(e.currentTarget.value)
									})}
							/>
						</div>

						<div>
							<label for={`deployment-${name}-protocol`} class="text-sm text-gray-600">Protocol</label>
							<select
								id={`deployment-${name}-protocol`}
								class="mt-1 w-full rounded-md border-gray-300 text-sm"
								value={deployment.protocol}
								on:change={(e) =>
									updateDeployment(name, {
										...deployment,
										protocol: e.currentTarget.value
									})}
							>
								<option value="http">HTTP</option>
								<option value="https">HTTPS</option>
								<option value="tcp">TCP</option>
							</select>
						</div>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>
