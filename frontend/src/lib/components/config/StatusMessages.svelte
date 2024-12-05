<!-- src/lib/components/config/StatusMessages.svelte -->
<script lang="ts">
  import { Alert } from '$lib/components/ui/alert';
  import { AlertCircle } from 'lucide-svelte';

  export let error = '';
  export let success = '';
</script>

{#if error}
  <Alert variant="destructive" class="mt-4">
    <AlertCircle class="h-4 w-4" />
    <svelte:fragment slot="title">Error</svelte:fragment>
    <svelte:fragment slot="description">{error}</svelte:fragment>
  </Alert>
{/if}

{#if success}
  <Alert class="mt-4 bg-green-50">
    <svelte:fragment slot="title">Success</svelte:fragment>
    <svelte:fragment slot="description">{success}</svelte:fragment>
  </Alert>
{/if}