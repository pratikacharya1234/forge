<script>
  import { onMount } from 'svelte';
  import { fade, slide } from 'svelte/transition';

  let tasks = [
    { id: 'rate-limiter', name: 'Rate Limiter' },
    { id: 'data-pipeline', name: 'Data Pipeline' },
    { id: 'auth-system', name: 'Auth System' }
  ];

  let selectedTaskId = 'rate-limiter';
  let replayData = null;
  let isRunning = false;
  let progress = {};
  let finished = {};

  async function loadReplay(id) {
    isRunning = true;
    finished = {};
    progress = {};
    
    const res = await fetch(`/data/arena-replays/${id}.json`);
    replayData = await res.json();
    
    // Initialize progress
    Object.keys(replayData.results).forEach(model => {
      progress[model] = 0;
    });

    // Simulate the live run
    Object.keys(replayData.results).forEach(model => {
      const result = replayData.results[model];
      let current = 0;
      const interval = setInterval(() => {
        current += 50;
        progress[model] = Math.min((current / result.time_ms) * 100, 100);
        if (current >= result.time_ms) {
          clearInterval(interval);
          finished[model] = true;
          if (Object.keys(finished).length === Object.keys(replayData.results).length) isRunning = false;
        }
      }, 50);
    });
  }

  onMount(() => {
    loadReplay(selectedTaskId);
  });

  function handleTaskChange(id) {
    selectedTaskId = id;
    loadReplay(id);
  }
</script>

<div class="container mx-auto px-6">
  <!-- Task Selector -->
  <div class="flex flex-wrap justify-center gap-4 mb-12">
    {#each tasks as task}
      <button 
        on:click={() => handleTaskChange(task.id)}
        class="px-6 py-2 rounded-full border transition-all {selectedTaskId === task.id ? 'bg-accent-primary border-accent-primary text-white' : 'border-white/10 text-white/60 hover:border-white/20'}"
      >
        {task.name}
      </button>
    {/each}
  </div>

  {#if replayData}
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
      {#each Object.entries(replayData.results) as [model, data]}
        <div class="glass rounded-2xl overflow-hidden flex flex-col border-white/10 relative {finished[model] && replayData.winner === model ? 'ring-2 ring-gemini/50 winner-glow' : ''}">
          
          {#if finished[model] && replayData.winner === model}
            <div class="absolute top-4 right-4 z-20 flex items-center gap-2" transition:fade>
              <svg class="w-4 h-4 text-gemini animate-bounce" fill="currentColor" viewBox="0 0 20 20"><path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z"/></svg>
              <span class="bg-gemini text-black text-[10px] font-bold px-2 py-1 rounded uppercase tracking-tighter">Winner</span>
            </div>
          {/if}

          <!-- Model Header -->
          <div class="p-6 border-b border-white/5">
            <div class="flex items-center justify-between mb-4">
              <h3 class="font-bold text-lg uppercase tracking-tight">{model.replace(/-/g, ' ')}</h3>
              <span class="text-xs font-mono {data.cost === 0 ? 'text-gemini' : 'text-white/40'}">
                ${data.cost.toFixed(3)}
              </span>
            </div>
            
            <!-- Progress Bar -->
            <div class="w-full h-1 bg-white/5 rounded-full overflow-hidden">
              <div 
                class="h-full transition-all duration-300 {model.includes('gemini') ? 'bg-gemini' : model.includes('claude') ? 'bg-claude' : 'bg-gpt'}"
                style="width: {progress[model]}%"
              ></div>
            </div>
          </div>

          <!-- Terminal Replay -->
          <div class="flex-1 p-4 bg-black/40 font-mono text-[11px] h-64 overflow-y-auto space-y-2">
            {#each data.tool_calls as call}
              <div class="text-white/40">
                <span class="text-accent-secondary">tool:</span> {call.action}
              </div>
              <div class="text-white/60 mb-2">
                {call.content}
              </div>
            {/each}
            {#if finished[model]}
               <div class="text-gemini" transition:fade>✓ Task completed in {data.time_ms}ms</div>
            {/if}
          </div>

          <!-- Result Code (if finished) -->
          {#if finished[model]}
            <div class="p-4 bg-code-bg border-t border-white/5 max-h-48 overflow-y-auto" transition:slide>
              <pre class="text-[10px] text-white/80"><code>{data.code}</code></pre>
            </div>
            <div class="p-4 bg-white/5 border-t border-white/5 flex justify-between text-[10px] font-mono text-white/40">
              <span>{data.tokens} tokens</span>
              <span>{data.time_ms / 1000}s elapsed</span>
            </div>
          {/if}
        </div>
      {/each}
    </div>

    {#if !isRunning && replayData.winner_reason}
      <div class="mt-12 glass p-8 rounded-2xl border-gemini/20 text-center" transition:fade>
        <h4 class="text-gemini font-bold mb-2 uppercase tracking-widest text-sm">Verdict</h4>
        <p class="text-white/80 italic text-lg">"{replayData.winner_reason}"</p>
      </div>
    {/if}
  {/if}
</div>
