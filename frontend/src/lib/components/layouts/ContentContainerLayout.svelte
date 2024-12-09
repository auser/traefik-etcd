<script lang="ts">
	let { children } = $props();
</script>

<div class="rounded-lg bg-white p-6">
	<div class="space-y-6">
		{@render children()}
	</div>
</div>
