<script>
  import { fade } from 'svelte/transition';
  export let code = '';
  export let language = 'bash';

  let copied = false;

  async function copy() {
    await navigator.clipboard.writeText(code);
    copied = true;
    setTimeout(() => copied = false, 2000);
  }
</script>

<div class="bg-black/40 rounded-lg p-4 font-mono text-sm flex justify-between items-center group relative border border-white/5">
  <div class="overflow-x-auto">
    <code class="text-accent-secondary whitespace-pre">{code}</code>
  </div>
  <button 
    on:click={copy}
    class="ml-4 p-2 rounded bg-white/5 hover:bg-white/10 transition-colors text-white/40 hover:text-white shrink-0"
    title="Copy to clipboard"
  >
    {#if copied}
      <span in:fade class="text-gemini text-[10px] font-bold uppercase">Copied!</span>
    {:else}
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/></svg>
    {/if}
  </button>
</div>