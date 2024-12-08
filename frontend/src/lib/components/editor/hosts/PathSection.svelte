<script lang="ts">
	import { Plus, Trash2 } from 'lucide-svelte';
	import type { PathConfig, DeploymentConfig } from '$lib/types';
	import DeploymentSection from './DeploymentSection.svelte';
	import MiddlewareSection from './MiddlewareSection.svelte';

	export let paths: PathConfig[] = [];
	export let onChange: (paths: PathConfig[]) => void;

	function addPath() {
		onChange([
			...paths,
			{
				path: '',
				deployments: {},
				middlewares: [],
				stripPrefix: false,
				passThrough: false
			}
		]);
	}

	function updatePath(index: number, path: PathConfig) {
		const newPaths = [...paths];
		newPaths[index] = path;
		onChange(newPaths);
	}

	function removePath(index: number) {
		const newPaths = [...paths];
		newPaths.splice(index, 1);
		onChange(newPaths);
	}
</script>

<div class="space-y-4">
	<div class="flex items-center justify-between">
		<h3 class="text-sm font-medium text-gray-700">Path Configuration</h3>
		<button class="text-sm text-blue-500 hover:text-blue-600" on:click={addPath}>
			<Plus class="h-4 w-4" />
		</button>
	</div>

	{#if paths.length === 0}
		<div class="rounded border border-dashed py-4 text-center">
			<p class="text-sm text-gray-500">No paths configured</p>
		</div>
	{:else}
		<div class="space-y-4">
			{#each paths as path, index}
				<div class="space-y-4 rounded-lg border p-4">
					<div class="flex items-center justify-between">
						<div class="flex-1">
							<label class="block text-sm font-medium text-gray-700">Path</label>
							<input
								type="text"
								class="mt-1 w-full rounded-md border-gray-300"
								placeholder="/api"
								value={path.path}
								on:input={(e) => updatePath(index, { ...path, path: e.currentTarget.value })}
							/>
						</div>
						<button
							class="ml-2 text-gray-400 hover:text-red-500"
							on:click={() => removePath(index)}
						>
							<Trash2 class="h-4 w-4" />
						</button>
					</div>

					<!-- Path Options -->
					<div class="grid grid-cols-2 gap-4">
						<label class="flex items-center space-x-2">
							<input
								type="checkbox"
								class="rounded border-gray-300 text-blue-500"
								checked={path.stripPrefix}
								on:change={(e) =>
									updatePath(index, {
										...path,
										stripPrefix: e.currentTarget.checked
									})}
							/>
							<span class="text-sm text-gray-700">Strip Prefix</span>
						</label>

						<label class="flex items-center space-x-2">
							<input
								type="checkbox"
								class="rounded border-gray-300 text-blue-500"
								checked={path.passThrough}
								on:change={(e) =>
									updatePath(index, {
										...path,
										passThrough: e.currentTarget.checked
									})}
							/>
							<span class="text-sm text-gray-700">Pass Through</span>
						</label>
					</div>

					<!-- Path Deployments -->
					<div class="space-y-2">
						<h4 class="text-sm font-medium text-gray-700">Path Deployments</h4>
						<DeploymentSection
							deployments={path.deployments}
							onChange={(deployments) => updatePath(index, { ...path, deployments })}
						/>
					</div>

					<!-- Path Middlewares -->
					<div class="space-y-2">
						<h4 class="text-sm font-medium text-gray-700">Path Middlewares</h4>
						<MiddlewareSection
							middlewares={path.middlewares}
							onChange={(middlewares) => updatePath(index, { ...path, middlewares })}
						/>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>
