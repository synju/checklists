<script setup>
defineProps({
  items: { type: Array, default: () => [] },
  cursor: { type: Number, default: 0 },
})
defineEmits(['toggle', 'select', 'open', 'context'])
</script>

<template>
  <div class="panel checklist focused">
    <div class="panel-h">
      <span class="panel-t">Checklist</span>
      <span class="panel-meta" v-if="items.length">item {{ cursor + 1 }} of {{ items.length }}</span>
    </div>
    <div class="cl-items">
      <div
        v-for="(it, i) in items"
        :key="i"
        class="row"
        :class="{ active: i === cursor }"
        @click="$emit('select', i)"
        @dblclick="$emit('open')"
        @contextmenu.prevent="$emit('context', i)"
      >
        <button class="cbx" :aria-label="it.done ? 'Mark not done' : 'Mark done'" @click.stop="$emit('toggle', i)">
          <svg v-if="it.done" width="18" height="18" viewBox="0 0 24 24" fill="none">
            <rect x="3" y="3" width="18" height="18" rx="5" fill="var(--green)" />
            <path d="M8 12.5l2.5 2.5L16 9" stroke="var(--bg)" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
          <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none">
            <rect x="3.5" y="3.5" width="17" height="17" rx="5" :stroke="i === cursor ? 'var(--green)' : '#4a4a4a'" stroke-width="1.8" />
          </svg>
        </button>
        <span class="label" :class="{ done: it.done }">{{ it.label }}</span>
      </div>
      <div v-if="!items.length" style="padding:16px;color:#6a6a6a;font-size:12.5px">This checklist is empty</div>
    </div>
  </div>
</template>
