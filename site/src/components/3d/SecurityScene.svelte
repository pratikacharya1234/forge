<script>
  import { T, useTask } from '@threlte/core';
  import { OrbitControls } from '@threlte/extras';
  import * as THREE from 'three';

  let rotation = 0;
  useTask((delta) => {
    rotation += delta * 0.1;
  });

  const rings = [
    { radius: 4, color: '#4ade80', opacity: 0.1 },
    { radius: 6, color: '#22d3ee', opacity: 0.08 },
    { radius: 8, color: '#6366f1', opacity: 0.06 },
    { radius: 10, color: '#a855f7', opacity: 0.04 }
  ];
</script>

<T.PerspectiveCamera
  makeDefault
  position={[0, 15, 20]}
  fov={45}
>
  <OrbitControls
    enableZoom={false}
    autoRotate
    autoRotateSpeed={0.2}
  />
</T.PerspectiveCamera>

<T.AmbientLight intensity={0.5} />
<T.PointLight position={[0, 10, 0]} intensity={2} color="#6366f1" />

<T.Group rotation={[Math.PI / 2, 0, 0]}>
  {#each rings as ring, i}
    <T.Mesh rotation={[0, 0, rotation * (i + 1) * 0.2]}>
      <T.TorusGeometry args={[ring.radius, 0.02, 16, 100]} />
      <T.MeshStandardMaterial
        color={ring.color}
        emissive={ring.color}
        emissiveIntensity={2}
        transparent
        opacity={0.5}
      />
    </T.Mesh>
    
    <T.Mesh rotation={[0, 0, -rotation * (i + 1) * 0.15]}>
      <T.RingGeometry args={[ring.radius - 0.1, ring.radius + 0.1, 64]} />
      <T.MeshStandardMaterial
        color={ring.color}
        transparent
        opacity={ring.opacity}
        side={THREE.DoubleSide}
      />
    </T.Mesh>
  {/each}

  <!-- Center Shield -->
  <T.Mesh position={[0, 0, 0]} rotation={[-Math.PI / 2, 0, 0]}>
    <T.IcosahedronGeometry args={[2, 1]} />
    <T.MeshStandardMaterial
      color="#6366f1"
      wireframe
      transparent
      opacity={0.2}
    />
  </T.Mesh>
</T.Group>

<T.Fog color="#050510" near={10} far={50} />
