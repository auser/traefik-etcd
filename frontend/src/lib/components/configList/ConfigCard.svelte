<!-- src/lib/components/configurations/ConfigCard.svelte -->
<script lang="ts">
  import { Edit2, Trash2, Calendar, ArrowRight } from 'lucide-svelte';
  import { configStore } from '$lib/stores/configStore';
  import { AlertDialog } from '$lib/components/ui/alert-dialog';
	import AreYouSure from '../AreYouSure.svelte';

  export let config: {
    id: number;
    name: string;
    updated_at: string;
    source: string;
  };
  export let onDeleted: () => void;

  let showDeleteDialog = false;

  async function deleteConfig() {
    await configStore.deleteCurrentConfig();
    showDeleteDialog = false;
    onDeleted();
  }

  function formatDate(dateString: string) {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    });
  }
</script>

<div class="bg-white rounded-lg border overflow-hidden hover:shadow-md transition-shadow">
  <div class="p-6">
    <div class="flex justify-between items-start">
      <h3 class="text-lg font-medium text-gray-900">{config.name}</h3>
      <div class="flex items-center gap-2">
        
        <a
          href={`/config-editor/${config.id}`}
          class="p-1.5 text-gray-500 hover:text-blue-500 rounded-full"
          title="Edit"
        >
          <Edit2 class="w-4 h-4" />
        </a>
        <button
          class="p-1.5 text-gray-500 hover:text-red-500 rounded-full"
          title="Delete"
          on:click={() => showDeleteDialog = true}
        >
          <Trash2 class="w-4 h-4" />
        </button>
      </div>
    </div>

    <div class="mt-4 flex items-center text-sm text-gray-500">
      <Calendar class="w-4 h-4 mr-2" />
      Last updated {formatDate(config.updated_at)}
    </div>

    <div class="mt-4 flex items-center justify-between">
      <span class="text-sm px-2 py-1 bg-gray-100 text-gray-600 rounded">
        {config.source}
      </span>
      
      <a
        href={`/config-editor/${config.id}`}
        class="flex items-center text-blue-500 hover:text-blue-600"
      >
        Edit
        <ArrowRight class="w-4 h-4 ml-1" />
      </a>
    </div>
  </div>
</div>

<AreYouSure
  open={showDeleteDialog}
  title="Delete Configuration"
  message="Are you sure you want to delete this configuration?"
  onConfirm={deleteConfig}
/>
<!-- <AlertDialog
  open={showDeleteDialog}
  onOpenChange={(open) => showDeleteDialog = open}
>
  <AlertDialogContent>
    <AlertDialogHeader>
      <AlertDialogTitle>Delete Configuration</AlertDialogTitle>
      <AlertDialogDescription>
        Are you sure you want to delete "{config.name}"? This action cannot be undone.
      </AlertDialogDescription>
    </AlertDialog>
    <AlertDialog.Footer>
      <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
      <AlertDialog.Action destructive on:click={deleteConfig}>
        Delete
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog>
</AlertDialog> -->