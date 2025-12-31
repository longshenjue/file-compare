<template>
  <div class="bg-white rounded-lg shadow-lg p-8 max-w-4xl mx-auto">
    <h2 class="text-2xl font-bold text-gray-800 mb-6">上传银行对账单</h2>
    
    <div class="space-y-6">
      <!-- Source Name and Type -->
      <div class="grid grid-cols-2 gap-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">
            数据源名称
          </label>
          <input
            v-model="localConfig.sourceName"
            type="text"
            placeholder="请输入数据源名称"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">
            类型
          </label>
          <select
            v-model="localConfig.type"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
          >
            <option value="">请选择类型</option>
            <option value="PAYOUT">PAYOUT</option>
            <option value="PAYIN">PAYIN</option>
          </select>
        </div>
      </div>
      
      <!-- Date Range -->
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-2">
          日期范围
        </label>
        <div class="flex items-center space-x-2">
          <input
            v-model="localConfig.dateRange.start"
            type="date"
            class="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
          />
          <span class="text-gray-500">~</span>
          <input
            v-model="localConfig.dateRange.end"
            type="date"
            class="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
          />
        </div>
      </div>
      
      <!-- File Upload -->
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-2">
          请上传您的银行对账单
        </label>
        <div class="flex items-center space-x-2">
          <div class="flex-1 px-4 py-2 border border-gray-300 rounded-lg bg-gray-50 text-gray-600">
            {{ localConfig.fileName || '未选择文件' }}
          </div>
          <button
            @click="selectFile"
            class="px-6 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
          >
            上传
          </button>
        </div>
      </div>
      
      <!-- Advanced Options -->
      <div class="grid grid-cols-3 gap-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">
            Header 行号 <span class="text-red-500">*</span>
          </label>
          <input
            v-model.number="localConfig.header"
            type="number"
            min="0"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">
            时区 <span class="text-red-500">*</span>
          </label>
          <select
            v-model="localConfig.timezone"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
          >
            <option v-for="tz in timezones" :key="tz" :value="tz">{{ tz }}</option>
          </select>
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">
            去重规则
          </label>
          <div class="flex items-center h-10">
            <label class="flex items-center cursor-pointer">
              <input
                v-model="localConfig.removeDuplicate"
                type="checkbox"
                class="w-5 h-5 text-green-600 border-gray-300 rounded focus:ring-green-500"
              />
              <span class="ml-2 text-sm text-gray-700">移除重复</span>
            </label>
          </div>
        </div>
      </div>
      
      <!-- Confirm Button -->
      <div class="flex justify-center pt-4">
        <button
          @click="handleConfirm"
          :disabled="!isValid"
          :class="[
            'px-12 py-3 rounded-lg font-medium text-white transition-colors',
            isValid
              ? 'bg-green-600 hover:bg-green-700'
              : 'bg-gray-300 cursor-not-allowed'
          ]"
        >
          确认
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import type { FileConfig } from '../types';
import { TIMEZONES } from '../types';

const props = defineProps<{
  config: FileConfig;
}>();

const emit = defineEmits<{
  update: [config: FileConfig];
  confirm: [];
}>();

const localConfig = ref<FileConfig>({ ...props.config });
const timezones = TIMEZONES;

watch(
  localConfig,
  (newVal) => {
    emit('update', { ...newVal });
  },
  { deep: true }
);

const isValid = computed(() => {
  return (
    localConfig.value.sourceName &&
    localConfig.value.type &&
    localConfig.value.filePath &&
    localConfig.value.header >= 0 &&
    localConfig.value.timezone
  );
});

async function selectFile() {
  const selected = await open({
    multiple: false,
    filters: [{
      name: 'CSV Files',
      extensions: ['csv']
    }]
  });
  
  if (selected && typeof selected === 'string') {
    localConfig.value.filePath = selected;
    localConfig.value.fileName = selected.split('/').pop() || '';
  }
}

function handleConfirm() {
  if (isValid.value) {
    emit('confirm');
  }
}
</script>

