<script>
  import { T, useTask } from '@threlte/core';
  import { OrbitControls, Float } from '@threlte/extras';
  import * as THREE from 'three';

  let rotation = 0;
  let offset = 0;
  useTask((delta) => {
    rotation += delta * 0.2;
    offset = Math.sin(Date.now() * 0.001) * 0.5;
  });

  const modules = [
    { name: 'Core', color: '#6366f1', position: [0, 0, 0] },
    { name: 'Models', color: '#22d3ee', position: [0, 1.5, 0] },
    { name: 'Tools', color: '#4ade80', position: [0, -1.5, 0] },
    { name: 'Safety', color: '#f97316', position: [1.5, 0, 0] },
    { name: 'CLI', color: '#a855f7', position: [-1.5, 0, 0] }
  ];
</script>

<T.PerspectiveCamera
  makeDefault
  position={[5, 5, 5]}
  fov={35}
>
  <OrbitControls enableZoom={false} />
</T.PerspectiveCamera>

<T.AmbientLight intensity={0.5} />
<T.PointLight position={[10, 10, 10]} intensity={1} />

<T.Group rotation={[0, rotation, 0]}>
  {#each modules as mod}
    <T.Mesh 
      position={[
        mod.position[0] * (1 + offset), 
        mod.position[1] * (1 + offset), 
        mod.position[2] * (1 + offset)
      ]}
    >
      <T.BoxGeometry args={[0.8, 0.8, 0.8]} />
      <T.MeshStandardMaterial
        color={mod.color}
        emissive={mod.color}
        emissiveIntensity={0.5}
        transparent
        opacity={0.8}
      />
    </T.Mesh>
  {/each}
</T.Group>
