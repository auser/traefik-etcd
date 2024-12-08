<!-- src/lib/components/middleware/MiddlewareCard.svelte -->
<script lang="ts">
	import { ChevronDown, Trash2 } from 'lucide-svelte';
	import { slide } from 'svelte/transition';
	import clsx from 'clsx';
	import HeadersConfig from '$lib/components/editor/middleware/HeadersConfig.svelte';

	export let name: string;
	export let config: any;
	export let onChange: (config: any) => void;
	export let onDelete: () => void;

	let expanded = false;
</script>

<div class="rounded-lg border bg-white">
	<div class="flex items-center justify-between p-4">
		<input
			type="text"
			class="rounded border-none bg-transparent text-lg font-medium focus:ring-2 focus:ring-blue-500"
			value={name}
			placeholder="Middleware name"
		/>

		<div class="flex items-center gap-2">
			<button
				class="rounded p-1 text-gray-500 hover:text-gray-700"
				on:click={() => (expanded = !expanded)}
			>
				<ChevronDown
					class={clsx(
						'h-5 w-5 transform transition-transform duration-200',
						expanded && 'rotate-180'
					)}
				/>
			</button>
			<button class="rounded p-1 text-gray-500 hover:text-red-500" on:click={onDelete}>
				<Trash2 class="h-5 w-5" />
			</button>
		</div>
	</div>

	{#if expanded}
		<div class="border-t p-4" transition:slide>
			<div class="space-y-4">
				<HeadersConfig
					headers={config.headers}
					onChange={(headers) => onChange({ ...config, headers })}
				/>
			</div>
		</div>
	{/if}
</div>
