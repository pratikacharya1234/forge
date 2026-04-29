<script>
  import { onMount } from 'svelte';

  export let command = 'forge-cli --prompt "add rate limiting"';
  export let output = [
    'Researching project conventions...',
    'Detected: Rust, Actix-web, Cargo',
    'Searching for rate limiting best practices...',
    'Decomposing task into 3 subtasks:',
    '  1. Add governor dependency to Cargo.toml',
    '  2. Create middleware/ratelimit.rs',
    '  3. Register middleware in main.rs',
    'Executing subtask 1/3...',
    'DONE. API rate limiting added with 100 req/min limit.'
  ];

  let displayedCommand = '';
  let displayedOutput = [];
  let showCursor = true;
  let phase = 'typing'; // typing, thinking, output

  onMount(() => {
    let i = 0;
    const typeCommand = () => {
      if (i < command.length) {
        displayedCommand += command[i];
        i++;
        setTimeout(typeCommand, 50 + Math.random() * 50);
      } else {
        phase = 'thinking';
        setTimeout(showOutput, 1000);
      }
    };

    const showOutput = () => {
      phase = 'output';
      let lineIndex = 0;
      const addLine = () => {
        if (lineIndex < output.length) {
          displayedOutput = [...displayedOutput, output[lineIndex]];
          lineIndex++;
          setTimeout(addLine, 400 + Math.random() * 600);
        }
      };
      addLine();
    };

    typeCommand();

    const cursorInterval = setInterval(() => {
      showCursor = !showCursor;
    }, 500);

    return () => clearInterval(cursorInterval);
  });
</script>

<div class="terminal-window w-full max-w-2xl mx-auto font-mono">
  <div class="terminal-header">
    <div class="terminal-dot bg-red-500"></div>
    <div class="terminal-dot bg-yellow-500"></div>
    <div class="terminal-dot bg-green-500"></div>
    <span class="ml-2 text-xs text-white/40 uppercase tracking-widest">forge-cli — bash</span>
  </div>
  <div class="terminal-body min-h-[300px]">
    <div class="flex items-center">
      <span class="text-terminal-green mr-2">$</span>
      <span>{displayedCommand}</span>
      {#if phase === 'typing' && showCursor}
        <span class="w-2 h-5 bg-white/50 ml-1"></span>
      {/if}
    </div>
    
    <div class="mt-4 space-y-1">
      {#each displayedOutput as line}
        <div class="text-white/80">
          {#if line.startsWith('DONE')}
            <span class="text-terminal-green">✓ {line}</span>
          {:else if line.includes('Executing')}
            <span class="text-accent-cyan">➜ {line}</span>
          {:else}
            <span class="text-white/60">{line}</span>
          {/if}
        </div>
      {/each}
      {#if phase === 'thinking'}
        <div class="text-white/40 animate-pulse">Processing...</div>
      {/if}
    </div>
  </div>
</div>
