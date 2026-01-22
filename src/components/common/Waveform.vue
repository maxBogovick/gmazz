<script setup lang="ts">
import { ref, onMounted } from 'vue';

const canvas = ref<HTMLCanvasElement | null>(null);

onMounted(() => {
  if (!canvas.value) return;

  const ctx = canvas.value.getContext('2d');
  if (!ctx) return;

  const width = canvas.value.width;
  const height = canvas.value.height;
  const barCount = 60;
  const barWidth = width / barCount - 1;

  ctx.fillStyle = '#A9A9A3';

  for (let i = 0; i < barCount; i++) {
    const barHeight = Math.random() * (height * 0.8) + height * 0.1;
    const x = i * (barWidth + 1);
    const y = (height - barHeight) / 2;
    ctx.fillRect(x, y, barWidth, barHeight);
  }
});
</script>

<template>
  <canvas ref="canvas" class="w-full h-full opacity-60" width="300" height="48"></canvas>
</template>
