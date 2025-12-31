<template>
  <div class="bg-white rounded-lg shadow-lg p-8 max-w-6xl mx-auto">
    <h2 class="text-2xl font-bold text-gray-800 mb-6">匹配配置与状态归一化</h2>
    
    <div class="space-y-8">
      <!-- Source A Configuration -->
      <div class="border border-gray-200 rounded-lg p-6">
        <h3 class="text-xl font-semibold text-gray-800 mb-4">数据源A配置</h3>
        
        <!-- Source A ID Field -->
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-2">
            数据源A ID字段 <span class="text-red-500">*</span>
          </label>
          <select
            v-model="localConfig.sourceAIdField"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
          >
            <option value="">请选择字段</option>
            <option v-for="field in sourceAFields" :key="field" :value="field">
              {{ field }}
            </option>
          </select>
        </div>
        
        <!-- Source A Status Mapping -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">
            状态映射
          </label>
          <div class="space-y-3">
            <div
              v-for="(mapping, index) in localConfig.sourceAStatusMapping"
              :key="index"
              class="flex items-center space-x-2"
            >
              <div class="flex-1">
                <div class="flex flex-wrap gap-2 p-2 border border-gray-300 rounded-lg min-h-[42px]">
                  <span
                    v-for="status in mapping.sourceStatus"
                    :key="status"
                    class="inline-flex items-center px-3 py-1 bg-green-600 text-white text-sm rounded-full"
                  >
                    {{ status }}
                    <button
                      @click="removeSourceAStatus(index, status)"
                      class="ml-2 hover:text-gray-200"
                    >
                      ×
                    </button>
                  </span>
                  <input
                    v-model="sourceAStatusInput[index]"
                    @keydown.enter="addSourceAStatus(index)"
                    type="text"
                    placeholder="输入状态后按回车"
                    class="flex-1 min-w-[120px] outline-none text-sm"
                  />
                </div>
              </div>
              <span class="text-gray-400">→</span>
              <input
                v-model="mapping.targetStatus"
                type="text"
                placeholder="目标状态"
                class="w-40 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500"
              />
              <button
                @click="removeSourceAMapping(index)"
                class="text-red-500 hover:text-red-700"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
          </div>
          <button
            @click="addSourceAMapping"
            class="mt-3 px-4 py-2 text-sm text-teal-600 border border-teal-600 rounded-lg hover:bg-teal-50 transition-colors"
          >
            + 添加映射
          </button>
        </div>
      </div>
      
      <!-- Source B Configuration -->
      <div class="border border-gray-200 rounded-lg p-6">
        <h3 class="text-xl font-semibold text-gray-800 mb-4">数据源B配置</h3>
        
        <!-- Source B ID Field -->
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-2">
            数据源B ID字段 <span class="text-red-500">*</span>
          </label>
          <select
            v-model="localConfig.sourceBIdField"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
          >
            <option value="">请选择字段</option>
            <option v-for="field in sourceBFields" :key="field" :value="field">
              {{ field }}
            </option>
          </select>
        </div>
        
        <!-- Source B Status Mapping -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">
            状态映射
          </label>
          <div class="space-y-3">
            <div
              v-for="(mapping, index) in localConfig.sourceBStatusMapping"
              :key="index"
              class="flex items-center space-x-2"
            >
              <div class="flex-1">
                <div class="flex flex-wrap gap-2 p-2 border border-gray-300 rounded-lg min-h-[42px]">
                  <span
                    v-for="status in mapping.sourceStatus"
                    :key="status"
                    class="inline-flex items-center px-3 py-1 bg-green-600 text-white text-sm rounded-full"
                  >
                    {{ status }}
                    <button
                      @click="removeSourceBStatus(index, status)"
                      class="ml-2 hover:text-gray-200"
                    >
                      ×
                    </button>
                  </span>
                  <input
                    v-model="sourceBStatusInput[index]"
                    @keydown.enter="addSourceBStatus(index)"
                    type="text"
                    placeholder="输入状态后按回车"
                    class="flex-1 min-w-[120px] outline-none text-sm"
                  />
                </div>
              </div>
              <span class="text-gray-400">→</span>
              <input
                v-model="mapping.targetStatus"
                type="text"
                placeholder="目标状态"
                class="w-40 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500"
              />
              <button
                @click="removeSourceBMapping(index)"
                class="text-red-500 hover:text-red-700"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
          </div>
          <button
            @click="addSourceBMapping"
            class="mt-3 px-4 py-2 text-sm text-teal-600 border border-teal-600 rounded-lg hover:bg-teal-50 transition-colors"
          >
            + 添加映射
          </button>
        </div>
      </div>
      
      <!-- Historical Data Options -->
      <div class="border border-indigo-200 rounded-lg p-6 bg-indigo-50">
        <h3 class="text-xl font-semibold text-gray-800 mb-4 flex items-center gap-2">
          <svg class="w-6 h-6 text-indigo-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          历史数据对账选项
        </h3>
        <p class="text-sm text-gray-600 mb-4">
          启用后，对账时会自动加载历史数据，帮助消除因时间延迟导致的单边账问题（例如订单在1号文件，银行流水延迟到2号文件）
        </p>
        
        <div class="grid grid-cols-2 gap-6">
          <!-- Historical Source A Option -->
          <div class="bg-white rounded-lg p-4 border-2 border-gray-200">
            <label class="flex items-center cursor-pointer">
              <input
                v-model="localConfig.useHistoricalSourceA"
                type="checkbox"
                class="w-5 h-5 text-indigo-600 border-gray-300 rounded focus:ring-indigo-500"
              />
              <span class="ml-3 text-sm font-medium text-gray-700">使用历史数据源A数据</span>
            </label>
            <p class="mt-2 text-xs text-gray-500 ml-8">
              启用后，对账时会加载前后几天的历史数据源A数据进行匹配
            </p>
          </div>
          
          <!-- Historical Source B Option -->
          <div class="bg-white rounded-lg p-4 border-2 border-gray-200">
            <label class="flex items-center cursor-pointer">
              <input
                v-model="localConfig.useHistoricalSourceB"
                type="checkbox"
                class="w-5 h-5 text-indigo-600 border-gray-300 rounded focus:ring-indigo-500"
              />
              <span class="ml-3 text-sm font-medium text-gray-700">使用历史数据源B数据</span>
            </label>
            <p class="mt-2 text-xs text-gray-500 ml-8">
              启用后，对账时会加载前后几天的历史数据源B数据进行匹配
            </p>
          </div>
        </div>
        
        <!-- History Days Range -->
        <div class="mt-4 bg-white rounded-lg p-4 border-2 border-gray-200">
          <label class="block text-sm font-medium text-gray-700 mb-2">
            历史数据时间范围
          </label>
          <div class="flex items-center gap-3">
            <span class="text-sm text-gray-600">前后</span>
            <input
              v-model.number="localConfig.historyDays"
              type="number"
              min="1"
              max="30"
              class="w-24 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
            />
            <span class="text-sm text-gray-600">天</span>
            <span class="text-xs text-gray-500 ml-4">
              （建议5-10天，过大会影响性能）
            </span>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Navigation Buttons -->
    <div class="flex justify-center space-x-4 mt-8">
      <button
        @click="$emit('back')"
        class="px-8 py-3 border-2 border-teal-600 text-teal-600 rounded-lg hover:bg-teal-50 transition-colors"
      >
        返回
      </button>
      <button
        @click="handleFinish"
        :disabled="!isValid"
        :class="[
          'px-8 py-3 rounded-lg font-medium text-white transition-colors',
          isValid
            ? 'bg-green-600 hover:bg-green-700'
            : 'bg-gray-300 cursor-not-allowed'
        ]"
      >
        完成
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type { MatchConfig } from '../types';

const props = defineProps<{
  config: MatchConfig;
  sourceAFields: string[];
  sourceBFields: string[];
}>();

const emit = defineEmits<{
  update: [config: MatchConfig];
  finish: [];
  back: [];
}>();

const localConfig = ref<MatchConfig>({ 
  ...props.config,
  useHistoricalSourceA: props.config.useHistoricalSourceA ?? false,
  useHistoricalSourceB: props.config.useHistoricalSourceB ?? false,
  historyDays: props.config.historyDays ?? 5,
});
const sourceAStatusInput = ref<string[]>([]);
const sourceBStatusInput = ref<string[]>([]);

watch(
  localConfig,
  (newVal) => {
    emit('update', { ...newVal });
  },
  { deep: true }
);

const isValid = computed(() => {
  return (
    localConfig.value.sourceAIdField &&
    localConfig.value.sourceBIdField &&
    localConfig.value.sourceAStatusMapping.length > 0 &&
    localConfig.value.sourceBStatusMapping.length > 0
  );
});

function addSourceAMapping() {
  localConfig.value.sourceAStatusMapping.push({
    sourceStatus: [],
    targetStatus: ''
  });
  sourceAStatusInput.value.push('');
}

function removeSourceAMapping(index: number) {
  localConfig.value.sourceAStatusMapping.splice(index, 1);
  sourceAStatusInput.value.splice(index, 1);
}

function addSourceAStatus(index: number) {
  const value = sourceAStatusInput.value[index]?.trim();
  if (value && !localConfig.value.sourceAStatusMapping[index].sourceStatus.includes(value)) {
    localConfig.value.sourceAStatusMapping[index].sourceStatus.push(value);
    sourceAStatusInput.value[index] = '';
  }
}

function removeSourceAStatus(index: number, status: string) {
  const statusIndex = localConfig.value.sourceAStatusMapping[index].sourceStatus.indexOf(status);
  if (statusIndex > -1) {
    localConfig.value.sourceAStatusMapping[index].sourceStatus.splice(statusIndex, 1);
  }
}

function addSourceBMapping() {
  localConfig.value.sourceBStatusMapping.push({
    sourceStatus: [],
    targetStatus: ''
  });
  sourceBStatusInput.value.push('');
}

function removeSourceBMapping(index: number) {
  localConfig.value.sourceBStatusMapping.splice(index, 1);
  sourceBStatusInput.value.splice(index, 1);
}

function addSourceBStatus(index: number) {
  const value = sourceBStatusInput.value[index]?.trim();
  if (value && !localConfig.value.sourceBStatusMapping[index].sourceStatus.includes(value)) {
    localConfig.value.sourceBStatusMapping[index].sourceStatus.push(value);
    sourceBStatusInput.value[index] = '';
  }
}

function removeSourceBStatus(index: number, status: string) {
  const statusIndex = localConfig.value.sourceBStatusMapping[index].sourceStatus.indexOf(status);
  if (statusIndex > -1) {
    localConfig.value.sourceBStatusMapping[index].sourceStatus.splice(statusIndex, 1);
  }
}

function handleFinish() {
  if (isValid.value) {
    emit('finish');
  }
}
</script>

