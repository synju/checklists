<script setup>
import { onMounted, onUnmounted } from 'vue'
defineProps({ width: { type: Number, default: 480 } })
const emit = defineEmits(['close'])
function onKey(e) {
  if (e.key === 'Escape') { e.stopPropagation(); e.preventDefault(); emit('close') }
}
onMounted(() => window.addEventListener('keydown', onKey, true))
onUnmounted(() => window.removeEventListener('keydown', onKey, true))
</script>

<template>
  <div class="modal-overlay" @mousedown.self="emit('close')">
    <div class="modal-dialog" :style="{ width: width + 'px' }" @mousedown.stop>
      <slot />
    </div>
  </div>
</template>
