<script>
  let mobileMenuOpen = false;
</script>

<nav class="fixed top-0 left-0 w-full z-50 px-6 py-4 flex items-center justify-between bg-background/80 backdrop-blur-xl border-b border-white/5">
  <div class="flex items-center gap-2 font-bold text-xl">
    <div class="w-8 h-8 bg-accent-indigo rounded flex items-center justify-center text-xs text-white">F</div>
    FORGE
  </div>
  
  <div class="hidden md:flex gap-8 text-sm font-medium text-white/60">
    <a href="#how-it-works" class="hover:text-white transition-colors">How It Works</a>
    <a href="#use" class="hover:text-white transition-colors">Usage</a>
    <a href="#download" class="hover:text-white transition-colors">Download</a>
    <a href="#community" class="hover:text-white transition-colors">Community</a>
  </div>

  <div class="flex items-center gap-4">
    <a href="https://github.com/pratikacharya1234/forge" target="_blank" class="hidden sm:block px-4 py-2 bg-white/5 hover:bg-white/10 border border-white/10 rounded-lg text-sm font-bold transition-all text-white">
      GitHub
    </a>
    
    <button 
      class="md:hidden p-2 text-white/60"
      on:click={() => mobileMenuOpen = !mobileMenuOpen}
    >
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        {#if mobileMenuOpen}
          <path d="M18 6L6 18M6 6l12 12" />
        {:else}
          <path d="M4 6h16M4 12h16M4 18h16" />
        {/if}
      </svg>
    </button>
  </div>
</nav>

{#if mobileMenuOpen}
  <div class="fixed inset-0 z-40 bg-background md:hidden pt-24 px-6 overflow-hidden">
    <div class="flex flex-col gap-6 text-2xl font-bold">
      <a href="#how-it-works" on:click={() => mobileMenuOpen = false} class="text-white">How It Works</a>
      <a href="#use" on:click={() => mobileMenuOpen = false} class="text-white">Usage</a>
      <a href="#download" on:click={() => mobileMenuOpen = false} class="text-white">Download</a>
      <a href="#community" on:click={() => mobileMenuOpen = false} class="text-white">Community</a>
      <a href="https://github.com/pratikacharya1234/forge" target="_blank" class="text-accent-indigo">GitHub ★</a>
    </div>
  </div>
{/if}
