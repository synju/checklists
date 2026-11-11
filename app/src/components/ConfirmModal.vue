<script setup>
import { ref, onMounted } from 'vue'
import BaseModal from './BaseModal.vue'

defineProps({ message: String, okText: { type: String, default: 'Delete' } })
const emit = defineEmits(['yes', 'close'])

const noEl = ref(null)
onMounted(() => noEl.value?.focus()) // default to Cancel for safety
</script>

<template>
  <BaseModal :width="420" @close="emit('close')">
    <div class="modal-message">{{ message }}</div>
    <div class="modal-actions center">
      <button class="btn danger" @click="emit('yes')">{{ okText }}</button>
      <button ref="noEl" class="btn" @click="emit('close')">Cancel</button>
    </div>
  </BaseModal>
</template>
