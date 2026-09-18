<script>
  import { onMount, onDestroy } from 'svelte';
  import { BackgroundShader } from '../background-shader.js';

  let canvasRef;
  let shaderInstance = null;

  onMount(() => {
    if (canvasRef) {
      shaderInstance = new BackgroundShader(canvasRef);
    }
  });

  onDestroy(() => {
    if (shaderInstance) {
      shaderInstance.destroy();
      shaderInstance = null;
    }
  });
</script>

<canvas id="bg-canvas" bind:this={canvasRef}></canvas>
