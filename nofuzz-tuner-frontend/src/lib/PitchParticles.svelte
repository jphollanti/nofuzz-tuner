<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
  
    /* --------------------------------------------------------------
       Public props
    --------------------------------------------------------------
       • `cents` (‑30 … 30 or null) – tuning offset
       • `baseSpeed` – baseline orbit velocity when |cents| == MAX_CENTS
       • `speedVariance` – per‑particle speed wobble ± percentage
       • `transparent` – if true, canvas has no solid background; trails
         are disabled so the scene always fully clears each frame.
    -------------------------------------------------------------- */
    export let cents: number | null = null;           // null → free‑float mode
    export let particleCount: number = 600;          // ≥ 10
    export let trailStrength: number = 0.06;         // 0 → no trail … 1 → infinite (ignored when transparent)
    export let orbitRadius: number | null = null;    // px; null → auto size
    export let baseSpeed: number = 0.05;             // rad/frame at max error
    export let speedVariance: number = 0.25;         // 0 → uniform speed
    export let transparent: boolean = false;         // true → see‑through canvas
  
    /* Internal constant – full‑speed cents threshold */
    const MAX_CENTS = 30;
  
    /* Canvas plumbing */
    let canvas: HTMLCanvasElement;
    let ctx!: CanvasRenderingContext2D;
    let W = 0, H = 0, DPR = 1;
  
    /* Particle layout per element: [θ, radialNoise, personalAngVel, orbitFactor, speedMult] */
    const STRIDE = 5;
    let data!: Float32Array;
    const TWO_PI = Math.PI * 2;
    const lerp = (a: number, b: number, t: number) => a + (b - a) * t;
  
    const ORBIT_VARIANCE_MIN = 0.8;
    const ORBIT_VARIANCE_MAX = 1.25;
  
    let globalSpeedFactor = 0; // cached each frame
  
    /* ------------------------------ resize ------------------------------ */
    function resize(): void {
      DPR = window.devicePixelRatio || 1;
      W   = canvas.clientWidth;
      H   = canvas.clientHeight;
      canvas.width  = W * DPR;
      canvas.height = H * DPR;
      ctx.setTransform(DPR, 0, 0, DPR, 0, 0);
      if (orbitRadius === null) {
        orbitRadius = Math.min(W, H) * 0.35;
      }
    }
  
    /* ----------------------------- init -------------------------------- */
    function initParticles(): void {
      data = new Float32Array(particleCount * STRIDE);
      for (let i = 0; i < particleCount; ++i) {
        const p = i * STRIDE;
        data[p]     = Math.random() * TWO_PI;                        // θ
        data[p + 1] = (Math.random() - 0.5) * 6;                     // radial jitter
        data[p + 2] = (Math.random() - 0.5) * 0.02;                  // personal angVel
        data[p + 3] = lerp(ORBIT_VARIANCE_MIN, ORBIT_VARIANCE_MAX, Math.random());
        data[p + 4] = 1 + (Math.random() * 2 - 1) * speedVariance;   // speedMult
      }
    }
  
    /* ---------------------------- physics ------------------------------ */
    function update(): void {
      const freeFloat = cents == null;
      let speedFactor = 0;
      let direction   = 1;
  
      if (!freeFloat) {
        const delta = cents as number;
        speedFactor = Math.min(Math.abs(delta) / MAX_CENTS, 1);
        direction   = delta < 0 ? 1 : -1;
      }
  
      globalSpeedFactor = speedFactor;
  
      for (let i = 0, p = 0; i < particleCount; ++i, p += STRIDE) {
        const radialIdx = p + 1;
        const velIdx    = p + 2;
        const multIdx   = p + 4;
  
        if (freeFloat) {
          data[velIdx] += (Math.random() - 0.5) * 0.0005;
          data[velIdx] = Math.max(-0.03, Math.min(0.03, data[velIdx]));
          data[p] += data[velIdx];
          data[radialIdx] += (Math.random() - 0.5) * 0.05;
          continue;
        }
  
        const particleBase = baseSpeed * data[multIdx];
        const personalTurn = data[velIdx] * 0.3;
        data[p] += speedFactor * particleBase * direction + personalTurn;
        data[velIdx] *= 0.985;
        const wobble = speedFactor * 0.3;
        data[radialIdx] = lerp(data[radialIdx], (Math.random() - 0.5) * wobble * 10, 0.05);
      }
    }
  
    /* ----------------------------- render ------------------------------ */
    function draw(): void {
      // Clear / fade previous frame
      if (transparent || trailStrength === 0) {
        ctx.clearRect(0, 0, W, H);
      } else {
        ctx.fillStyle = `rgba(0,0,0,${1 - trailStrength})`;
        ctx.fillRect(0, 0, W, H);
      }
  
      const cx = W * 0.5;
      const cy = H * 0.5;
      const baseR = orbitRadius as number;
  
      const spread = cents == null ? 1 : globalSpeedFactor;
  
      for (let i = 0, p = 0; i < particleCount; ++i, p += STRIDE) {
        const θ = data[p];
        const orbitFactor = data[p + 3];
        const blendedOrbit = 1 + spread * (orbitFactor - 1);
        const radialNoise  = data[p + 1] * spread;
        const r = baseR * blendedOrbit + radialNoise;
        const x = cx + Math.cos(θ) * r;
        const y = cy + Math.sin(θ) * r;
  
        // Speed → alpha mapping
        const angSpeed = cents == null
          ? Math.abs(data[p + 2])
          : baseSpeed * data[p + 4] * globalSpeedFactor + Math.abs(data[p + 2] * 0.3);
        const alpha = 1 - Math.min(1, angSpeed / (baseSpeed * 1.2)) * 0.7;
  
        ctx.fillStyle = `rgba(8,253,216,${alpha.toFixed(3)})`;
        ctx.beginPath();
        ctx.arc(x, y, 1.4, 0, TWO_PI);
        ctx.fill();
      }
    }
  
    /* --------------------------- main loop ----------------------------- */
    let running = true;
    function loop(): void {
      if (!running) return;
      update();
      draw();
      requestAnimationFrame(loop);
    }
  
    /* -------------------------- lifecycle ------------------------------ */
    onMount(() => {
      ctx = canvas.getContext('2d')!;
      resize();
      window.addEventListener('resize', resize);
      initParticles();
      requestAnimationFrame(loop);
    });
  
    onDestroy(() => {
      running = false;
      window.removeEventListener('resize', resize);
    });
  
    /* -------------------- reactive reallocations ----------------------- */
    $: if (data && particleCount * STRIDE !== data.length) initParticles();
  </script>
  
  <!-- Visual stage -->
  <canvas bind:this={canvas} class="w-full h-full block" style="background: transparent;"></canvas>
  
  <style>
    canvas {
      display: block;
      width: 80vw;
      height: 60svh;   /* ≈ “safe”, never pushes off-screen */
      position: relative;
      background: transparent;
    }
  </style>
  