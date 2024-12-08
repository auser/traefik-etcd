<script lang="ts">
	import { Plus, Trash2 } from 'lucide-svelte';
	import { etcdStore } from '$lib/stores/etcdStore';

	function addEndpoint() {
		$etcdStore.endpoints = [...$etcdStore.endpoints, ''];
	}

	function removeEndpoint(index: number) {
		const newEndpoints = [...$etcdStore.endpoints];
		newEndpoints.splice(index, 1);
		$etcdStore.endpoints = newEndpoints;
	}
</script>

<div class="rounded-lg border bg-white p-6">
	<h2 class="mb-4 text-lg font-semibold">ETCD Configuration</h2>

	<div class="space-y-6">
		<!-- Endpoints -->
		<div class="space-y-2">
			<div class="flex items-center justify-between">
				<label class="font-medium" for="endpoints">Endpoints</label>
				<button class="text-sm text-blue-500 hover:text-blue-600" on:click={addEndpoint}>
					<Plus class="h-4 w-4" />
				</button>
			</div>
			{#each $etcdStore.endpoints as endpoint, i}
				<div class="flex gap-2">
					<input
						id="endpoints"
						type="text"
						class="flex-1 rounded-md border-gray-300"
						value={endpoint}
						on:input={(e) => {
							const newEndpoints = [...$etcdStore.endpoints];
							newEndpoints[i] = e.currentTarget.value;
							$etcdStore.endpoints = newEndpoints;
						}}
					/>
					<button class="text-gray-400 hover:text-red-500" on:click={() => removeEndpoint(i)}>
						<Trash2 class="h-4 w-4" />
					</button>
				</div>
			{/each}
		</div>

		<!-- Timeout & Keep Alive -->
		<div class="grid grid-cols-2 gap-4">
			<div>
				<label class="mb-1 block font-medium" for="timeout">Timeout (ms)</label>
				<input
					id="timeout"
					type="number"
					class="w-full rounded-md border-gray-300"
					value={$etcdStore.timeout}
					on:input={(e) => ($etcdStore.timeout = parseInt(e.currentTarget.value))}
				/>
			</div>
			<div>
				<label class="mb-1 block font-medium" for="keep-alive">Keep Alive (s)</label>
				<input
					id="keep-alive"
					type="number"
					class="w-full rounded-md border-gray-300"
					value={$etcdStore.keep_alive}
					on:input={(e) => ($etcdStore.keep_alive = parseInt(e.currentTarget.value))}
				/>
			</div>
		</div>

		<!-- TLS Configuration -->
		<div class="space-y-4">
			<label class="inline-flex items-center" for="tls">
				<input
					id="tls"
					type="checkbox"
					class="rounded border-gray-300 text-blue-500"
					checked={!!$etcdStore.tls}
					on:change={(e) => {
						etcdStore.setTls(e.currentTarget.checked ? {} : undefined);
					}}
				/>
				<span class="ml-2 font-medium">Enable TLS</span>
			</label>

			{#if $etcdStore.tls}
				<div class="space-y-4 pl-6">
					<div class="grid grid-cols-2 gap-4">
						<div>
							<label class="mb-1 block font-medium" for="cert">Certificate Path</label>
							<input
								id="cert"
								type="text"
								class="w-full rounded-md border-gray-300"
								value={$etcdStore.tls.cert}
								placeholder="./config/tls/cert.pem"
								on:input={(e) => {
									$etcdStore.tls = { ...$etcdStore.tls, cert: e.currentTarget.value };
								}}
							/>
						</div>

						<div>
							<label class="mb-1 block font-medium" for="key">Key Path</label>
							<input
								id="key"
								type="text"
								class="w-full rounded-md border-gray-300"
								value={$etcdStore.tls.key}
								placeholder="./config/tls/key.pem"
								on:input={(e) => {
									$etcdStore.tls = { ...$etcdStore.tls, key: e.currentTarget.value };
								}}
							/>
						</div>

						<div>
							<label class="mb-1 block font-medium" for="ca">CA Path</label>
							<input
								id="ca"
								type="text"
								class="w-full rounded-md border-gray-300"
								value={$etcdStore.tls.ca}
								placeholder="./config/tls/ca.pem"
								on:input={(e) => {
									$etcdStore.tls = { ...$etcdStore.tls, ca: e.currentTarget.value };
								}}
							/>
						</div>

						<div>
							<label class="mb-1 block font-medium" for="domain">Domain</label>
							<input
								id="domain"
								type="text"
								class="w-full rounded-md border-gray-300"
								value={$etcdStore.tls.domain}
								placeholder="etcd"
								on:input={(e) => {
									$etcdStore.tls = { ...$etcdStore.tls, domain: e.currentTarget.value };
								}}
							/>
						</div>
					</div>
				</div>
			{/if}
		</div>
	</div>
</div>
