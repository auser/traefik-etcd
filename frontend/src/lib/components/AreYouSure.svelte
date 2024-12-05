<!-- src/lib/components/config/TemplateSelector.svelte -->
<script lang="ts">
  import { AlertDialog, AlertDialogContent } from '$lib/components/ui/alert-dialog';
  import { onMount } from 'svelte';

  export let title: string;
  export let message: string;
  export let onConfirm: () => void;
  export let open: boolean = false;
</script>

{#if open}
<div class="space-y-4">
  <h3 class="text-xl font-semibold">{title}</h3>
  

  <!-- Delete Confirmation Dialog -->
    <AlertDialog {open}>
      <AlertDialogContent>
        <h2 class="text-lg font-semibold text-red-600">{title}</h2>
        <p class="mt-2">
          {message}
        </p>
        <div class="flex justify-end gap-2 mt-4">
          <button
            class="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded"
            on:click={() => open = false}
          >
            Cancel
          </button>
          <button
            class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600"
            on:click={onConfirm}
          >
            {message}
          </button>
        </div>
      </AlertDialogContent>
    </AlertDialog>
  </div>
  {/if}