<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

const canvas = ref<HTMLCanvasElement | null>(null);
let resizeObserver: ResizeObserver | null = null;

function drawWaveform() {
  if (!canvas.value) return;

  const ctx = canvas.value.getContext('2d');
  if (!ctx) return;

  const rect = canvas.value.getBoundingClientRect();
  const dpr = window.devicePixelRatio || 1;
  canvas.value.width = Math.max(1, Math.floor(rect.width * dpr));
  canvas.value.height = Math.max(1, Math.floor(rect.height * dpr));
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.scale(dpr, dpr);

  const width = rect.width;
  const height = rect.height;
  const barCount = 60;
  const barWidth = width / barCount - 1;

  ctx.fillStyle = '#C9A227';
  ctx.clearRect(0, 0, width, height);

  for (let i = 0; i < barCount; i++) {
    const barHeight = Math.random() * (height * 0.8) + height * 0.1;
    const x = i * (barWidth + 1);
    const y = (height - barHeight) / 2;
    ctx.fillRect(x, y, barWidth, barHeight);
  }
}

onMounted(() => {
  drawWaveform();
  if (canvas.value && 'ResizeObserver' in window) {
    resizeObserver = new ResizeObserver(() => drawWaveform());
    resizeObserver.observe(canvas.value);
  }
});

onUnmounted(() => {
  if (resizeObserver && canvas.value) {
    resizeObserver.unobserve(canvas.value);
  }
  resizeObserver = null;
});
</script>

<template>
  <canvas ref="canvas" class="w-full h-full opacity-60" width="300" height="48"></canvas>
</template>
