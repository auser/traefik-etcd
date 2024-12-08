<script lang="ts">
	import type { HostConfig } from '$lib/types';

	export let host: HostConfig;
	export let onChange: (host: HostConfig) => void;
</script>

<div class="grid grid-cols-2 gap-4">
	<label class="flex items-center space-x-2">
		<input
			type="checkbox"
			class="rounded border-gray-300 text-blue-500"
			checked={host.passThrough}
			on:change={(e) =>
				onChange({
					...host,
					passThrough: e.currentTarget.checked
				})}
		/>
		<span>Pass Through</span>
	</label>

	<label class="flex items-center space-x-2">
		<input
			type="checkbox"
			class="rounded border-gray-300 text-blue-500"
			checked={host.wwwRedirect}
			on:change={(e) =>
				onChange({
					...host,
					wwwRedirect: e.currentTarget.checked
				})}
		/>
		<span>WWW Redirect</span>
	</label>

	<label class="flex items-center space-x-2">
		<input
			type="checkbox"
			class="rounded border-gray-300 text-blue-500"
			checked={host.forwardHost}
			on:change={(e) =>
				onChange({
					...host,
					forwardHost: e.currentTarget.checked
				})}
		/>
		<span>Forward Host</span>
	</label>
</div>
