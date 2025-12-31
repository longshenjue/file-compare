<template>
  <div class="flex items-center justify-center py-8">
    <div v-for="(step, index) in steps" :key="index" class="flex items-center">
      <!-- Step Circle -->
      <div class="flex flex-col items-center">
        <div
          :class="[
            'w-16 h-16 rounded-2xl flex items-center justify-center text-xl font-bold transition-all',
            currentStep === index + 1
              ? 'bg-green-600 text-white'
              : currentStep > index + 1
              ? 'bg-teal-100 text-teal-600'
              : 'bg-gray-200 text-gray-400'
          ]"
        >
          {{ index + 1 }}
        </div>
        <div
          :class="[
            'mt-2 text-sm font-medium',
            currentStep === index + 1
              ? 'text-green-600'
              : currentStep > index + 1
              ? 'text-teal-600'
              : 'text-gray-400'
          ]"
        >
          {{ step }}
        </div>
      </div>
      
      <!-- Arrow -->
      <div
        v-if="index < steps.length - 1"
        :class="[
          'mx-4 text-2xl',
          currentStep > index + 1 ? 'text-teal-600' : 'text-gray-300'
        ]"
      >
        ›
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  currentStep: number;
  steps: string[];
}>();
</script>

