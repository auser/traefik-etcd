<!-- src/lib/components/middleware/HeadersConfig.svelte -->
<script lang="ts">
	import { Plus, Trash2 } from 'lucide-svelte';

	export let headers: {
		custom_request_headers?: Record<string, string>;
		custom_response_headers?: Record<string, string>;
		access_control_allow_methods?: string[];
		access_control_allow_headers?: string[];
		access_control_expose_headers?: string[];
		add_vary_header?: boolean;
	};
	export let onChange: (headers: any) => void;

	function addHeader(type: 'request' | 'response') {
		const target = type === 'request' ? 'custom_request_headers' : 'custom_response_headers';
		onChange({
			...headers,
			[target]: {
				...headers[target],
				'': ''
			}
		});
	}

	function removeHeader(type: 'request' | 'response', key: string) {
		const target = type === 'request' ? 'custom_request_headers' : 'custom_response_headers';
		const newHeaders = { ...headers[target] };
		delete newHeaders[key];
		onChange({
			...headers,
			[target]: newHeaders
		});
	}
</script>

<div class="space-y-6">
	<!-- Request Headers -->
	<div>
		<div class="mb-2 flex items-center justify-between">
			<h4 class="font-medium">Request Headers</h4>
			<button
				class="text-sm text-blue-500 hover:text-blue-600"
				on:click={() => addHeader('request')}
			>
				<Plus class="h-4 w-4" />
			</button>
		</div>

		{#if headers.custom_request_headers && Object.keys(headers.custom_request_headers).length > 0}
			{#each Object.entries(headers.custom_request_headers) as [key, value]}
				<div class="mb-2 flex gap-2">
					<input
						type="text"
						class="flex-1 rounded border-gray-300"
						placeholder="Header name"
						value={key}
					/>
					<input type="text" class="flex-1 rounded border-gray-300" placeholder="Value" {value} />
					<button
						class="text-gray-400 hover:text-red-500"
						on:click={() => removeHeader('request', key)}
					>
						<Trash2 class="h-4 w-4" />
					</button>
				</div>
			{/each}
		{/if}
	</div>

	<!-- Response Headers -->
	<div>
		<div class="mb-2 flex items-center justify-between">
			<h4 class="font-medium">Response Headers</h4>
			<button
				class="text-sm text-blue-500 hover:text-blue-600"
				on:click={() => addHeader('response')}
			>
				<Plus class="h-4 w-4" />
			</button>
		</div>

		{#if headers.custom_response_headers && Object.keys(headers.custom_response_headers).length > 0}
			{#each Object.entries(headers.custom_response_headers) as [key, value]}
				<div class="mb-2 flex gap-2">
					<input
						type="text"
						class="flex-1 rounded border-gray-300"
						placeholder="Header name"
						value={key}
					/>
					<input type="text" class="flex-1 rounded border-gray-300" placeholder="Value" {value} />
					<button
						class="text-gray-400 hover:text-red-500"
						on:click={() => removeHeader('response', key)}
					>
						<Trash2 class="h-4 w-4" />
					</button>
				</div>
			{/each}
		{/if}
	</div>

	<!-- Add more middleware-specific configurations here -->
</div>
