<script setup>
import { computed } from 'vue'
import { marked } from 'marked'

const props = defineProps({ item: { type: Object, default: null } })
marked.setOptions({ breaks: true, gfm: true })

// Local, trusted checklist files. TODO: sanitize if the store ever holds untrusted input.
const html = computed(() => (props.item?.detail ? marked.parse(props.item.detail) : ''))
</script>

<template>
  <div class="panel detail">
    <div class="panel-h">
      <span class="panel-t">Detail</span>
      <span style="font-size:10px;color:#565656">markdown</span>
    </div>
    <div class="detail-body">
      <template v-if="item">
        <div v-if="html" class="md" v-html="html"></div>
        <div v-else class="detail-empty">(no detail)</div>
      </template>
      <div v-else class="detail-empty">No item selected</div>
    </div>
  </div>
</template>
