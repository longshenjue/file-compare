<template>
  <div class="min-h-screen bg-gradient-to-br from-purple-50 to-teal-50 py-8 px-4">
    <div class="max-w-7xl mx-auto">
      <!-- Header -->
      <div class="text-center mb-8">
        <h1 class="text-4xl font-bold text-gray-800 mb-2">对账引擎</h1>
        <p class="text-gray-600">企业级账单对账与数据清洗平台</p>
      </div>
      
      <!-- Stepper -->
      <Stepper :current-step="currentStep" :steps="steps" />
      
      <!-- Step Content -->
      <div class="mt-8">
        <!-- Step 1: Order File Upload -->
        <div v-if="currentStep === 1">
          <FileUpload
            :config="orderFileConfig"
            @update="orderFileConfig = $event"
            @confirm="handleOrderFileConfirm"
          />
        </div>
        
        <!-- Step 2: Bank File Upload -->
        <div v-else-if="currentStep === 2">
          <FileUpload
            :config="bankFileConfig"
            @update="bankFileConfig = $event"
            @confirm="handleBankFileConfirm"
          />
        </div>
        
        <!-- Step 3: Order Field Mapping -->
        <div v-else-if="currentStep === 3">
          <FieldMapping
            :mappings="orderMappings"
            :available-columns="orderColumns"
            @update="orderMappings = $event"
            @next="handleOrderMappingNext"
            @back="currentStep = 2"
          />
        </div>
        
        <!-- Step 4: Bank Field Mapping -->
        <div v-else-if="currentStep === 4">
          <FieldMapping
            :mappings="bankMappings"
            :available-columns="bankColumns"
            @update="bankMappings = $event"
            @next="handleBankMappingNext"
            @back="currentStep = 3"
          />
        </div>
        
        <!-- Step 5: Match Configuration -->
        <div v-else-if="currentStep === 5">
          <MatchConfiguration
            :config="matchConfig"
            :source-a-fields="getOrderFieldNames()"
            :source-b-fields="getBankFieldNames()"
            @update="matchConfig = $event"
            @preview="handlePreview"
            @finish="handleFinish"
            @back="currentStep = 4"
          />
        </div>
        
        <!-- Step 6: Results -->
        <div v-else-if="currentStep === 6">
          <div class="bg-white rounded-lg shadow-lg p-8 max-w-6xl mx-auto">
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
              <div class="flex justify-center space-x-4">
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
              
              <!-- Reset Button -->
              <div class="flex justify-center mt-8">
                <button
                  @click="reset"
                  class="px-6 py-2 border-2 border-gray-400 text-gray-600 rounded-lg hover:bg-gray-50 transition-colors"
                >
                  重新开始
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { save, message } from '@tauri-apps/plugin-dialog';
import Stepper from './Stepper.vue';
import FileUpload from './FileUpload.vue';
import FieldMapping from './FieldMapping.vue';
import MatchConfiguration from './MatchConfiguration.vue';
import type { FileConfig, ColumnMapping, MatchConfig, ReconciliationResult } from '../types';

const currentStep = ref(1);
const loading = ref(false);
const results = ref<ReconciliationResult | null>(null);

const steps = ['订单文件', '银行文件', '订单映射', '银行映射', '匹配配置', '对账结果'];

// Order file configuration
const orderFileConfig = ref<FileConfig>({
  sourceName: '',
  type: '',
  dateRange: {
    start: new Date().toISOString().split('T')[0],
    end: new Date().toISOString().split('T')[0]
  },
  filePath: '',
  fileName: '',
  header: 1,
  timezone: 'America/Sao_Paulo',
  removeDuplicate: false
});

// Bank file configuration
const bankFileConfig = ref<FileConfig>({
  sourceName: '',
  type: '',
  dateRange: {
    start: new Date().toISOString().split('T')[0],
    end: new Date().toISOString().split('T')[0]
  },
  filePath: '',
  fileName: '',
  header: 1,
  timezone: 'America/Sao_Paulo',
  removeDuplicate: false
});

// Column mappings
const orderColumns = ref<string[]>([]);
const bankColumns = ref<string[]>([]);
const orderMappings = ref<ColumnMapping[]>([
  {
    id: 'mapping-1',
    sourceColumn: '',
    fieldType: 'OrderString',
    fieldName: '',
    ruleType: '',
    ruleConfig: '',
    saveOriginal: false,
    formatRules: []
  }
]);
const bankMappings = ref<ColumnMapping[]>([
  {
    id: 'mapping-1',
    sourceColumn: '',
    fieldType: 'OrderString',
    fieldName: '',
    ruleType: '',
    ruleConfig: '',
    saveOriginal: false,
    formatRules: []
  }
]);

// Match configuration
const matchConfig = ref<MatchConfig>({
  sourceAIdField: '',
  sourceAStatusMapping: [
    {
      sourceStatus: [],
      targetStatus: ''
    }
  ],
  sourceBIdField: '',
  sourceBStatusMapping: [
    {
      sourceStatus: [],
      targetStatus: ''
    }
  ]
});

async function handleOrderFileConfirm() {
  try {
    loading.value = true;
    const columns = await invoke<string[]>('read_csv_headers', {
      filePath: orderFileConfig.value.filePath,
      headerRow: orderFileConfig.value.header
    });
    orderColumns.value = columns;
    currentStep.value = 2;
  } catch (error) {
    console.error('读取订单文件失败:', error);
    await message('读取文件失败: ' + error, {
      title: '操作失败',
      kind: 'error',
    });
  } finally {
    loading.value = false;
  }
}

async function handleBankFileConfirm() {
  try {
    loading.value = true;
    const columns = await invoke<string[]>('read_csv_headers', {
      filePath: bankFileConfig.value.filePath,
      headerRow: bankFileConfig.value.header
    });
    bankColumns.value = columns;
    currentStep.value = 3;
  } catch (error) {
    console.error('读取银行文件失败:', error);
    await message('读取文件失败: ' + error, {
      title: '操作失败',
      kind: 'error',
    });
  } finally {
    loading.value = false;
  }
}

function handleOrderMappingNext() {
  currentStep.value = 4;
}

function handleBankMappingNext() {
  currentStep.value = 5;
}

async function handlePreview() {
  // Preview functionality - could show sample data
  await handleFinish();
}

async function handleFinish() {
  try {
    loading.value = true;
    currentStep.value = 6;
    
    const result = await invoke<ReconciliationResult>('reconcile', {
      orderConfig: orderFileConfig.value,
      bankConfig: bankFileConfig.value,
      orderMappings: orderMappings.value,
      bankMappings: bankMappings.value,
      matchConfig: matchConfig.value
    });
    
    results.value = result;
  } catch (error) {
    console.error('对账失败:', error);
    await message('对账失败: ' + error, {
      title: '操作失败',
      kind: 'error',
    });
    currentStep.value = 5;
  } finally {
    loading.value = false;
  }
}

async function exportResults(type: 'all' | 'matched' | 'onlyInA' | 'onlyInB') {
  try {
    const filePath = await save({
      filters: [{
        name: 'CSV Files',
        extensions: ['csv']
      }]
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

function getOrderFieldNames(): string[] {
  return orderMappings.value.map(m => m.fieldName).filter(n => n);
}

function getBankFieldNames(): string[] {
  return bankMappings.value.map(m => m.fieldName).filter(n => n);
}

function reset() {
  currentStep.value = 1;
  results.value = null;
  orderFileConfig.value = {
    sourceName: '',
    type: '',
    dateRange: {
      start: new Date().toISOString().split('T')[0],
      end: new Date().toISOString().split('T')[0]
    },
    filePath: '',
    fileName: '',
    header: 1,
    timezone: 'America/Sao_Paulo',
    removeDuplicate: false
  };
  bankFileConfig.value = { ...orderFileConfig.value };
  orderMappings.value = [{
    id: 'mapping-1',
    sourceColumn: '',
    fieldType: 'OrderString',
    fieldName: '',
    ruleType: '',
    ruleConfig: '',
    saveOriginal: false,
    formatRules: []
  }];
  bankMappings.value = [{ ...orderMappings.value[0] }];
}
</script>

