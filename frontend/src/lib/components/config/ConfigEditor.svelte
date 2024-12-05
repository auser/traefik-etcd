<!-- src/lib/components/config/ConfigEditor.svelte -->
<script lang="ts">
  export let content = '';
  export let loading = false;
  export let saving = false;
  export let onChange: (content: string) => void;
</script>

<div class="mb-4">
  <textarea
    class="w-full h-[600px] p-4 font-mono text-sm border rounded"
    value={content}
    on:input={(e) => onChange(e.currentTarget.value)}
    spellcheck="false"
    disabled={loading || saving}
  />
</div>