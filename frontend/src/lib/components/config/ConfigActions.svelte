<script lang="ts">
  import { Save, FileDown } from 'lucide-svelte';
  
  export let onSave: () => void;
  export let onReset: () => void;
  export let loading = false;
  export let saving = false;
  export let disabled = false;
</script>

<div class="flex gap-2">
  <button
    class="px-4 py-2 bg-blue-500 text-white rounded flex items-center gap-2 hover:bg-blue-600 disabled:opacity-50"
    on:click={onSave}
    disabled={loading || saving || disabled}
  >
    <Save class="w-4 h-4" />
    {saving ? 'Saving...' : 'Save'}
  </button>

  <button
    class="px-4 py-2 bg-gray-500 text-white rounded flex items-center gap-2 hover:bg-gray-600 disabled:opacity-50"
    on:click={onReset}
    disabled={loading || saving || disabled}
  >
    <FileDown class="w-4 h-4" />
    Reset
  </button>
</div>