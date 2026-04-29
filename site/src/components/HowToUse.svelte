<script>
  import { Check, Copy } from 'lucide-svelte';
  
  let copied = '';

  async function copyToClipboard(text, id) {
    await navigator.clipboard.writeText(text);
    copied = id;
    setTimeout(() => {
      if (copied === id) copied = '';
    }, 2000);
  }
</script>

<section id="use" class="py-24 px-6">
  <div class="max-w-4xl mx-auto">
    <div class="text-center mb-16 fade-in">
      <h2 class="text-3xl md:text-5xl font-bold mb-4">Get Started</h2>
      <p class="text-white/60 text-lg">From zero to agentic coding in 30 seconds.</p>
    </div>

    <div class="space-y-12">
      <!-- Install -->
      <div class="space-y-4 fade-in">
        <h3 class="text-xl font-bold flex items-center gap-2">
          <span class="text-accent-indigo">01</span> One-liner Install
        </h3>
        <div class="relative group">
          <pre class="bg-black border border-white/10 p-6 rounded-xl font-mono text-sm overflow-x-auto text-terminal-green">curl -fsSL https://raw.githubusercontent.com/pratikacharya1234/forge/main/install.sh | bash</pre>
          <button 
            on:click={() => copyToClipboard('curl -fsSL https://raw.githubusercontent.com/pratikacharya1234/forge/main/install.sh | bash', 'install')}
            class="absolute top-4 right-4 p-2 bg-white/5 hover:bg-white/10 rounded-lg transition-colors"
          >
            {#if copied === 'install'}
              <Check size={16} class="text-terminal-green" />
            {:else}
              <Copy size={16} class="text-white/40" />
            {/if}
          </button>
        </div>
      </div>

      <!-- API Key -->
      <div class="space-y-4 fade-in">
        <h3 class="text-xl font-bold flex items-center gap-2">
          <span class="text-accent-indigo">02</span> Set API Key
        </h3>
        <div class="relative group">
          <pre class="bg-black border border-white/10 p-6 rounded-xl font-mono text-sm overflow-x-auto text-terminal-green">export FORGE_API_KEY="your-gemini-key"</pre>
          <button 
            on:click={() => copyToClipboard('export FORGE_API_KEY="your-gemini-key"', 'apikey')}
            class="absolute top-4 right-4 p-2 bg-white/5 hover:bg-white/10 rounded-lg transition-colors"
          >
            {#if copied === 'apikey'}
              <Check size={16} class="text-terminal-green" />
            {:else}
              <Copy size={16} class="text-white/40" />
            {/if}
          </button>
        </div>
      </div>

      <!-- Run -->
      <div class="space-y-4 fade-in">
        <h3 class="text-xl font-bold flex items-center gap-2">
          <span class="text-accent-indigo">03</span> Run Forge
        </h3>
        <div class="relative group">
          <pre class="bg-black border border-white/10 p-6 rounded-xl font-mono text-sm overflow-x-auto text-terminal-green">forge-cli</pre>
          <button 
            on:click={() => copyToClipboard('forge-cli', 'run')}
            class="absolute top-4 right-4 p-2 bg-white/5 hover:bg-white/10 rounded-lg transition-colors"
          >
            {#if copied === 'run'}
              <Check size={16} class="text-terminal-green" />
            {:else}
              <Copy size={16} class="text-white/40" />
            {/if}
          </button>
        </div>
      </div>
    </div>

    <!-- Commands Table -->
    <div class="mt-24 fade-in">
      <h3 class="text-2xl font-bold mb-8">Key Commands</h3>
      <div class="overflow-x-auto rounded-xl border border-white/10">
        <table class="w-full text-left text-sm">
          <thead class="bg-white/5 text-white/40 uppercase tracking-wider">
            <tr>
              <th class="px-6 py-4 font-medium">Command</th>
              <th class="px-6 py-4 font-medium">Action</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-white/5">
            {#each [
              { cmd: '/task <requirement>', desc: 'Full pipeline: research → decompose → dispatch → consensus' },
              { cmd: '/web', desc: 'Enable real-time web search for the current session' },
              { cmd: '/model auto', desc: 'Auto-route tasks to the best model based on complexity' },
              { cmd: '/test-fix <cmd>', desc: 'Run tests, detect failures, and fix code automatically' },
              { cmd: '/compact', desc: 'Summarize history to free up context window' }
            ] as row}
              <tr class="hover:bg-white/5 transition-colors">
                <td class="px-6 py-4 font-mono text-accent-cyan">{row.cmd}</td>
                <td class="px-6 py-4 text-white/70">{row.desc}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  </div>
</section>
