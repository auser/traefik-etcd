<script lang="ts">
	let { children }: { children: any } = $props();
</script>

<div class="rounded border-2 border-dashed py-8 text-center text-gray-500">
	{@render children()}
</div>
