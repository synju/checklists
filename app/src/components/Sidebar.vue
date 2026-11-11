<script setup>
defineProps({ checklists: { type: Array, default: () => [] }, activeId: String })
defineEmits(['select', 'context'])
</script>

<template>
  <div class="panel sidebar">
    <div class="panel-h">
      <span class="panel-t">Checklists</span>
      <span style="font-size:11px;color:#565656">{{ checklists.length }}</span>
    </div>
    <div class="sidebar-list">
      <div
        v-for="cl in checklists"
        :key="cl.id"
        class="cl-card"
        :class="{ active: cl.id === activeId, complete: cl.complete }"
        @click="$emit('select', cl.id)"
        @contextmenu.prevent="$emit('context', cl.id)"
      >
        <div class="cl-top">
          <span class="cl-name">{{ cl.displayName }}</span>
          <span class="cl-pct">{{ cl.percent }}%</span>
        </div>
        <div class="mini-track"><div class="mini-fill" :style="{ width: cl.percent + '%' }"></div></div>
        <div class="cl-sub">
          <span>{{ cl.done }} / {{ cl.total }} done</span>
          <span v-if="cl.id === activeId" style="color:var(--green)">● active</span>
          <svg v-else-if="cl.complete" width="12" height="12" viewBox="0 0 24 24" fill="none">
            <path d="M5 12.5l4.5 4.5L19 7.5" stroke="var(--green)" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </div>
      </div>
    </div>
  </div>
</template>
