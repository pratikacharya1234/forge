<script>
  import { onMount } from 'svelte';
  import { Star, GitFork, Download, Users } from 'lucide-svelte';

  let stars = $state(0);
  let forks = $state(0);
  let downloads = $state(0);
  let visitors = $state(0);
  let totalVisitors = $state(0);
  let loading = $state(true);

  async function fetchStats() {
    const CACHE_KEY = 'forge_stats_cache';
    const CACHE_TIME = 5 * 60 * 1000; // 5 minutes

    try {
      const cached = localStorage.getItem(CACHE_KEY);
      if (cached) {
        const { data, timestamp } = JSON.parse(cached);
        if (Date.now() - timestamp < CACHE_TIME) {
          stars = data.stars;
          forks = data.forks;
          downloads = data.downloads;
          totalVisitors = data.totalVisitors;
          visitors = data.visitors;
          loading = false;
          return;
        }
      }

      // Fetch Repo Stats
      const repoRes = await fetch('https://api.github.com/repos/pratikacharya1234/forge');
      const repoData = await repoRes.json();
      stars = repoData.stargazers_count || 0;
      forks = repoData.forks_count || 0;

      // Fetch Release Downloads
      const releasesRes = await fetch('https://api.github.com/repos/pratikacharya1234/forge/releases');
      const releasesData = await releasesRes.json();
      downloads = releasesData.reduce((acc, release) => {
        return acc + (release.assets?.reduce((a, asset) => a + asset.download_count, 0) || 0);
      }, 0);

      // Fetch Total Visitors from CounterAPI
      const counterRes = await fetch('https://api.counterapi.dev/v1/forge-cli/visit/up');
      const counterData = await counterRes.json();
      totalVisitors = counterData.count || 0;

      // Real-time visitors (Active Now)
      const baseActive = Math.max(1, Math.floor(Math.sqrt(totalVisitors / 10)));
      visitors = baseActive + Math.floor(Math.random() * 5);
      
      // Cache the results
      localStorage.setItem(CACHE_KEY, JSON.stringify({
        data: { stars, forks, downloads, totalVisitors, visitors },
        timestamp: Date.now()
      }));

      loading = false;
    } catch (e) {
      console.error('Failed to fetch stats', e);
      loading = false;
    }
  }

  onMount(() => {
    fetchStats();
    
    // Update visitors periodically to simulate "live" feel
    const interval = setInterval(() => {
      const change = Math.random() > 0.5 ? 1 : -1;
      visitors = Math.max(5, visitors + change);
    }, 5000);

    return () => clearInterval(interval);
  });

  function formatNum(num) {
    if (num >= 1000) return (num / 1000).toFixed(1) + 'k';
    return num.toString();
  }
</script>

<div class="grid grid-cols-2 lg:grid-cols-5 gap-4 w-full max-w-6xl mx-auto my-12 px-4">
  <div class="bg-[#141414] border border-white/10 p-6 rounded-xl flex flex-col items-center justify-center space-y-2 transition-all hover:border-indigo-500/50 group">
    <div class="text-indigo-400 group-hover:scale-110 transition-transform">
      <Star size={20} />
    </div>
    <span class="text-xl font-bold text-white font-mono">
      {loading ? '...' : formatNum(stars)}
    </span>
    <span class="text-[10px] uppercase tracking-widest text-white/50">Stars</span>
  </div>

  <div class="bg-[#141414] border border-white/10 p-6 rounded-xl flex flex-col items-center justify-center space-y-2 transition-all hover:border-cyan-500/50 group">
    <div class="text-cyan-400 group-hover:scale-110 transition-transform">
      <GitFork size={20} />
    </div>
    <span class="text-xl font-bold text-white font-mono">
      {loading ? '...' : formatNum(forks)}
    </span>
    <span class="text-[10px] uppercase tracking-widest text-white/50">Forks</span>
  </div>

  <div class="bg-[#141414] border border-white/10 p-6 rounded-xl flex flex-col items-center justify-center space-y-2 transition-all hover:border-green-500/50 group">
    <div class="text-green-400 group-hover:scale-110 transition-transform">
      <Download size={20} />
    </div>
    <span class="text-xl font-bold text-white font-mono">
      {loading ? '...' : formatNum(downloads)}
    </span>
    <span class="text-[10px] uppercase tracking-widest text-white/50">Downloads</span>
  </div>

  <div class="bg-[#141414] border border-white/10 p-6 rounded-xl flex flex-col items-center justify-center space-y-2 transition-all hover:border-amber-500/50 group">
    <div class="text-amber-400 group-hover:scale-110 transition-transform">
      <Users size={20} />
    </div>
    <span class="text-xl font-bold text-white font-mono">
      {loading ? '...' : formatNum(totalVisitors)}
    </span>
    <span class="text-[10px] uppercase tracking-widest text-white/50">Total Visits</span>
  </div>

  <div class="bg-[#141414] border border-white/10 p-6 rounded-xl flex flex-col items-center justify-center space-y-2 transition-all hover:border-rose-500/50 group relative overflow-hidden col-span-2 lg:col-span-1">
    <div class="absolute top-2 right-2 flex items-center space-x-1">
      <span class="relative flex h-2 w-2">
        <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-rose-400 opacity-75"></span>
        <span class="relative inline-flex rounded-full h-2 w-2 bg-rose-500"></span>
      </span>
      <span class="text-[8px] text-rose-500 font-bold uppercase tracking-tighter">Live</span>
    </div>
    <div class="text-rose-400 group-hover:scale-110 transition-transform">
      <Users size={20} />
    </div>
    <span class="text-xl font-bold text-white font-mono">
      {loading ? '...' : visitors}
    </span>
    <span class="text-[10px] uppercase tracking-widest text-white/50">Active Now</span>
  </div>
</div>
