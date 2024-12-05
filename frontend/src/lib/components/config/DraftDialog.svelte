<script lang="ts">
  import { AlertDialog, AlertDialogContent } from '$lib/components/ui/alert-dialog';

  export let show = false;
  export let onDiscard: () => void;
  export let onResume: () => void;
  export let lastSaved: string;
</script>

{#if show}
  <AlertDialog open={true}>
    <AlertDialogContent>
      <h2 class="text-lg font-semibold">Resume Editing?</h2>
      <p class="mt-2">
        You have an unsaved draft from {new Date(lastSaved).toLocaleString()}.
        Would you like to continue editing?
      </p>
      <div class="flex justify-end gap-2 mt-4">
        <button
          class="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded"
          on:click={onDiscard}
        >
          Discard
        </button>
        <button
          class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
          on:click={onResume}
        >
          Resume Editing
        </button>
      </div>
    </AlertDialogContent>
  </AlertDialog>
{/if}