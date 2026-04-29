<script>
  import { Canvas } from '@threlte/core';
  import DocsScene from './DocsScene.svelte';
</script>

<div class="w-full h-full">
  <Canvas>
    <DocsScene />
    <div slot="fallback" class="w-full h-full flex items-center justify-center bg-black/20">
      <div class="text-white/20 font-mono text-[10px] uppercase tracking-widest">WebGL Not Supported</div>
    </div>
  </Canvas>
</div>
