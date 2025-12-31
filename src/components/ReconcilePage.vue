<template>
  <div class="min-h-screen bg-gradient-to-br from-purple-50 to-teal-50 py-8 px-4">
    <div class="max-w-6xl mx-auto">
      <!-- Header -->
      <div class="flex items-center mb-8">
        <button
          @click="$emit('navigate', 'home')"
          class="mr-4 p-2 hover:bg-white rounded-lg transition-colors"
        >
          <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
          </svg>
        </button>
        <div>
          <h1 class="text-3xl font-bold text-gray-800">执行对账</h1>
          <p class="text-gray-600 mt-1">选择配置并上传文件进行对账</p>
        </div>
      </div>
      
      <div v-if="step === 'select-config'" class="space-y-6">
        <!-- Select Config -->
        <div class="bg-white rounded-lg shadow-lg p-8">
          <h2 class="text-2xl font-bold text-gray-800 mb-6">选择渠道配置</h2>
          
          <div v-if="configs.length === 0" class="text-center py-12">
            <svg class="w-16 h-16 text-gray-300 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <p class="text-gray-600 mb-4">暂无可用配置</p>
            <button
              @click="$emit('navigate', 'config-list')"
              class="px-6 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
            >
              去创建配置
            </button>
          </div>
          
          <div v-else class="space-y-4">
            <div
              v-for="config in configs"
              :key="config.id"
              @click="selectConfig(config)"
              :class="[
                'border-2 rounded-lg p-6 cursor-pointer transition-all',
                selectedConfig?.id === config.id
                  ? 'border-green-500 bg-green-50'
                  : 'border-gray-200 hover:border-green-300'
              ]"
            >
              <div class="flex items-center justify-between">
                <div class="flex-1">
                  <div class="flex items-center space-x-3 mb-2">
                    <h3 class="text-xl font-bold text-gray-800">{{ config.name }}</h3>
                    <span
                      :class="[
                        'px-3 py-1 rounded-full text-xs font-medium',
                        config.type === 'PAYOUT' ? 'bg-orange-100 text-orange-600' : 'bg-blue-100 text-blue-600'
                      ]"
                    >
                      {{ config.type }}
                    </span>
                  </div>
                  <div class="text-sm text-gray-600 space-y-1">
                    <p>数据源A: {{ config.sourceAName }} | 数据源B: {{ config.sourceBName }}</p>
                    <p>数据源A字段: {{ config.sourceAConfig.mappings.length }} 个 | 数据源B字段: {{ config.sourceBConfig.mappings.length }} 个</p>
                  </div>
                </div>
                <div v-if="selectedConfig?.id === config.id" class="text-green-600">
                  <svg class="w-8 h-8" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                  </svg>
                </div>
              </div>
            </div>
          </div>
          
          <div v-if="selectedConfig" class="flex justify-end mt-6">
            <button
              @click="step = 'upload-files'"
              class="px-8 py-3 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
            >
              下一步：上传文件
            </button>
          </div>
        </div>
      </div>
      
      <div v-else-if="step === 'upload-files' && selectedConfig" class="space-y-6">
        <!-- Upload Files -->
        <div class="bg-white rounded-lg shadow-lg p-8">
          <div class="flex items-center justify-between mb-6">
            <h2 class="text-2xl font-bold text-gray-800">上传对账文件</h2>
            <span class="text-sm text-gray-500">
              配置: <strong class="text-gray-800">{{ selectedConfig.name }}</strong>
            </span>
          </div>
          
          <!-- Source A File -->
          <div class="mb-8">
            <h3 class="text-lg font-semibold text-gray-800 mb-4">数据源A文件（{{ selectedConfig.sourceAName }}）</h3>
            <div class="grid grid-cols-2 gap-4 mb-4">
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">日期范围</label>
                <div class="flex items-center space-x-2">
                  <input
                    v-model="sourceADateRange.start"
                    type="date"
                    class="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500"
                  />
                  <span>~</span>
                  <input
                    v-model="sourceADateRange.end"
                    type="date"
                    class="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500"
                  />
                </div>
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">选择文件</label>
                <div class="flex items-center space-x-2">
                  <div class="flex-1 px-4 py-2 border border-gray-300 rounded-lg bg-gray-50 text-gray-600">
                    {{ sourceAFileName || '未选择文件' }}
                  </div>
                  <button
                    @click="selectSourceAFile"
                    class="px-6 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
                  >
                    选择
                  </button>
                </div>
              </div>
            </div>
          </div>
          
          <!-- Source B File -->
          <div>
            <h3 class="text-lg font-semibold text-gray-800 mb-4">数据源B文件（{{ selectedConfig.sourceBName }}）</h3>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">日期范围</label>
                <div class="flex items-center space-x-2">
                  <input
                    v-model="sourceBDateRange.start"
                    type="date"
                    class="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500"
                  />
                  <span>~</span>
                  <input
                    v-model="sourceBDateRange.end"
                    type="date"
                    class="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500"
                  />
                </div>
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">选择文件</label>
                <div class="flex items-center space-x-2">
                  <div class="flex-1 px-4 py-2 border border-gray-300 rounded-lg bg-gray-50 text-gray-600">
                    {{ sourceBFileName || '未选择文件' }}
                  </div>
                  <button
                    @click="selectSourceBFile"
                    class="px-6 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
                  >
                    选择
                  </button>
                </div>
              </div>
            </div>
          </div>
          
          <!-- Actions -->
          <div class="flex justify-between mt-8">
            <button
              @click="step = 'select-config'"
              class="px-8 py-3 border-2 border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors"
            >
              返回
            </button>
            <button
              @click="executeReconcile"
              :disabled="!canExecute"
              :class="[
                'px-8 py-3 rounded-lg font-medium text-white transition-colors',
                canExecute ? 'bg-green-600 hover:bg-green-700' : 'bg-gray-300 cursor-not-allowed'
              ]"
            >
              执行对账
            </button>
          </div>
        </div>
      </div>
      
      <div v-else-if="step === 'results'" class="bg-white rounded-lg shadow-lg p-8">
        <h2 class="text-2xl font-bold text-gray-800 mb-6">对账结果</h2>
        
        <div v-if="loading" class="text-center py-12">
          <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-green-600"></div>
          <p class="mt-4 text-gray-600">正在处理对账数据...</p>
        </div>
        
        <div v-else-if="results">
          <!-- Summary Cards -->
          <div class="grid grid-cols-4 gap-4 mb-8">
            <div class="bg-green-50 border border-green-200 rounded-lg p-4">
              <div class="text-sm text-green-600 font-medium">完全匹配</div>
              <div class="text-3xl font-bold text-green-700 mt-2">
                {{ results.matched?.length || 0 }}
              </div>
            </div>
            <div class="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
              <div class="text-sm text-yellow-600 font-medium">金额差异</div>
              <div class="text-3xl font-bold text-yellow-700 mt-2">
                {{ results.diffAmount?.length || 0 }}
              </div>
            </div>
            <div class="bg-red-50 border border-red-200 rounded-lg p-4">
              <div class="text-sm text-red-600 font-medium">仅订单存在</div>
              <div class="text-3xl font-bold text-red-700 mt-2">
                {{ results.onlyInA?.length || 0 }}
              </div>
            </div>
            <div class="bg-orange-50 border border-orange-200 rounded-lg p-4">
              <div class="text-sm text-orange-600 font-medium">仅银行存在</div>
              <div class="text-3xl font-bold text-orange-700 mt-2">
                {{ results.onlyInB?.length || 0 }}
              </div>
            </div>
          </div>
          
          <!-- Export Buttons -->
          <div class="flex flex-wrap justify-center gap-4">
            <button
              @click="exportResults('all')"
              class="px-6 py-3 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
            >
              导出完整结果
            </button>
            <button
              @click="exportResults('matched')"
              class="px-6 py-3 bg-teal-600 text-white rounded-lg hover:bg-teal-700 transition-colors"
            >
              导出匹配数据
            </button>
            <button
              @click="exportResults('onlyInA')"
              class="px-6 py-3 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
            >
              导出单边A
            </button>
            <button
              @click="exportResults('onlyInB')"
              class="px-6 py-3 bg-orange-600 text-white rounded-lg hover:bg-orange-700 transition-colors"
            >
              导出单边B
            </button>
          </div>
          
          <!-- New Reconciliation -->
          <div class="flex justify-center mt-8 space-x-4">
            <button
              @click="reset"
              class="px-6 py-2 border-2 border-gray-400 text-gray-600 rounded-lg hover:bg-gray-50 transition-colors"
            >
              继续对账
            </button>
            <button
              @click="$emit('navigate', 'home')"
              class="px-6 py-2 border-2 border-green-600 text-green-600 rounded-lg hover:bg-green-50 transition-colors"
            >
              返回首页
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open, save, message } from '@tauri-apps/plugin-dialog';
import { loadConfigsFromStorage, type ChannelConfig, type ReconciliationResult } from '../types';

defineEmits<{
  navigate: [page: string];
}>();

const step = ref<'select-config' | 'upload-files' | 'results'>('select-config');
const configs = ref<ChannelConfig[]>([]);
const selectedConfig = ref<ChannelConfig | null>(null);
const loading = ref(false);
const results = ref<ReconciliationResult | null>(null);

const sourceAFilePath = ref('');
const sourceAFileName = ref('');
const sourceBFilePath = ref('');
const sourceBFileName = ref('');

const today = new Date().toISOString().split('T')[0];
const sourceADateRange = ref({ start: today, end: today });
const sourceBDateRange = ref({ start: today, end: today });

const canExecute = computed(() => {
  return sourceAFilePath.value && sourceBFilePath.value;
});

onMounted(async () => {
  configs.value = await loadConfigsFromStorage();
});

function selectConfig(config: ChannelConfig) {
  selectedConfig.value = config;
}

async function selectSourceAFile() {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'CSV Files', extensions: ['csv'] }]
  });
  
  if (selected && typeof selected === 'string') {
    sourceAFilePath.value = selected;
    sourceAFileName.value = selected.split('/').pop() || '';
  }
}

async function selectSourceBFile() {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'CSV Files', extensions: ['csv'] }]
  });
  
  if (selected && typeof selected === 'string') {
    sourceBFilePath.value = selected;
    sourceBFileName.value = selected.split('/').pop() || '';
  }
}

async function executeReconcile() {
  if (!selectedConfig.value || !canExecute.value) return;
  
  try {
    loading.value = true;
    step.value = 'results';
    
    const sourceAConfig = {
      sourceName: selectedConfig.value.sourceAName,
      type: selectedConfig.value.type,
      dateRange: sourceADateRange.value,
      filePath: sourceAFilePath.value,
      fileName: sourceAFileName.value,
      header: selectedConfig.value.sourceAConfig.header,
      timezone: selectedConfig.value.sourceAConfig.timezone,
      removeDuplicate: selectedConfig.value.sourceAConfig.removeDuplicate,
    };
    
    const sourceBConfig = {
      sourceName: selectedConfig.value.sourceBName,
      type: selectedConfig.value.type,
      dateRange: sourceBDateRange.value,
      filePath: sourceBFilePath.value,
      fileName: sourceBFileName.value,
      header: selectedConfig.value.sourceBConfig.header,
      timezone: selectedConfig.value.sourceBConfig.timezone,
      removeDuplicate: selectedConfig.value.sourceBConfig.removeDuplicate,
    };
    
    const taskName = `${selectedConfig.value.name} - ${sourceADateRange.value.start}`;
    
    const [task, result] = await invoke<[import('../types').ReconciliationTask, ReconciliationResult]>('reconcile', {
      sourceAConfig,
      sourceBConfig,
      sourceAMappings: selectedConfig.value.sourceAConfig.mappings,
      sourceBMappings: selectedConfig.value.sourceBConfig.mappings,
      matchConfig: selectedConfig.value.matchConfig,
      configId: selectedConfig.value.id,
      configName: selectedConfig.value.name,
      taskName,
    });
    
    results.value = result;
    
    // 提示用户任务已保存
    console.log('对账任务已保存，任务ID:', task.taskId);
  } catch (error) {
    console.error('对账失败:', error);
    await message('对账失败: ' + error, {
      title: '操作失败',
      kind: 'error',
    });
    step.value = 'upload-files';
  } finally {
    loading.value = false;
  }
}

async function exportResults(type: string) {
  try {
    const filePath = await save({
      filters: [{ name: 'CSV Files', extensions: ['csv'] }]
    });
    
    if (filePath) {
      await invoke('export_results', {
        results: results.value,
        exportType: type,
        filePath
      });
      await message('导出成功!', {
        title: '操作成功',
        kind: 'info',
      });
    }
  } catch (error) {
    console.error('导出失败:', error);
    await message('导出失败: ' + error, {
      title: '操作失败',
      kind: 'error',
    });
  }
}

function reset() {
  step.value = 'upload-files';
  results.value = null;
  sourceAFilePath.value = '';
  sourceAFileName.value = '';
  sourceBFilePath.value = '';
  sourceBFileName.value = '';
}
</script>

