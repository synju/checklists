<script setup>
import { ref, onMounted } from 'vue'
import BaseModal from './BaseModal.vue'

const props = defineProps({
  title: String,
  label: { type: String, default: '' },
  value: { type: String, default: '' },
  placeholder: { type: String, default: '' },
  okText: { type: String, default: 'Save' },
})
const emit = defineEmits(['submit', 'close'])

const val = ref(props.value)
const el = ref(null)
onMounted(() => { el.value?.focus(); el.value?.select?.() })

function submit() { emit('submit', val.value) }
</script>

<template>
  <BaseModal :width="460" @close="emit('close')">
    <div class="modal-title">{{ title }}</div>
    <label v-if="label" class="modal-label">{{ label }}</label>
    <input ref="el" v-model="val" class="modal-input" :placeholder="placeholder" @keydown.enter="submit" />
    <div class="modal-actions">
      <button class="btn" @click="emit('close')">Cancel</button>
      <button class="btn primary" @click="submit">{{ okText }}</button>
    </div>
    <div class="modal-hint"><kbd>Enter</kbd> {{ okText.toLowerCase() }} · <kbd>Esc</kbd> cancel</div>
  </BaseModal>
</template>
