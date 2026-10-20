<script setup lang="ts">
import { computed } from "vue";
import { segments, type Ranges } from "../../lib/fuzzy";

// Text with the parts that matched a search highlighted.
const props = defineProps<{ text: string; ranges?: Ranges }>();
const parts = computed(() => segments(props.text, props.ranges));
</script>

<template>
  <span
    ><template v-for="(p, i) in parts" :key="i"
      ><mark v-if="p.hit" class="rounded-[3px] bg-brand-500/20 font-semibold text-inherit">{{ p.text }}</mark
      ><template v-else>{{ p.text }}</template></template
    ></span
  >
</template>
