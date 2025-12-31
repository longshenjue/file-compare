<template>
  <div class="min-h-screen bg-gradient-to-br from-purple-50 to-teal-50 py-8 px-4">
    <div class="max-w-7xl mx-auto">
      <!-- Header -->
      <div class="flex items-center justify-between mb-8">
        <div class="flex items-center">
          <button
            @click="$emit('navigate', 'config-list')"
            class="mr-4 p-2 hover:bg-white rounded-lg transition-colors"
          >
            <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
            </svg>
          </button>
          <div>
            <h1 class="text-3xl font-bold text-gray-800">
              {{ configId ? '编辑配置' : '新建配置' }}
            </h1>
            <p class="text-gray-600 mt-1">配置渠道解析规则和字段映射</p>
          </div>
        </div>
        <button
          @click="saveConfig"
          :disabled="!isValid"
          :class="[
            'px-6 py-3 rounded-lg font-medium text-white transition-colors flex items-center space-x-2',
            isValid ? 'bg-green-600 hover:bg-green-700' : 'bg-gray-300 cursor-not-allowed'
          ]"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span>保存配置</span>
        </button>
      </div>
      
      <!-- Stepper -->
      <Stepper :current-step="currentStep" :steps="steps" />
      
      <!-- Step Content -->
      <div class="mt-8">
        <!-- Step 1: Basic Info -->
        <div v-if="currentStep === 1" class="bg-white rounded-lg shadow-lg p-8 max-w-4xl mx-auto">
          <h2 class="text-2xl font-bold text-gray-800 mb-6">基本信息</h2>
          
          <div class="space-y-6">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                配置名称 <span class="text-red-500">*</span>
              </label>
              <input
                v-model="config.name"
                type="text"
                placeholder="例如：VIDI-LT 支付渠道"
                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
              />
            </div>
            
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">
                  数据源A名称 <span class="text-red-500">*</span>
                </label>
                <input
                  v-model="config.sourceAName"
                  type="text"
                  placeholder="例如：内部订单系统"
                  class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
                />
              </div>
              
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">
                  数据源B名称 <span class="text-red-500">*</span>
                </label>
                <input
                  v-model="config.sourceBName"
                  type="text"
                  placeholder="例如：银行对账单"
                  class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
                />
              </div>
            </div>
            
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                类型 <span class="text-red-500">*</span>
              </label>
              <select
                v-model="config.type"
                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
              >
                <option value="">请选择</option>
                <option value="PAYOUT">PAYOUT</option>
                <option value="PAYIN">PAYIN</option>
              </select>
            </div>
            
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">
                  数据源A Header行号 <span class="text-red-500">*</span>
                </label>
                <input
                  v-model.number="config.sourceAConfig.header"
                  type="number"
                  min="1"
                  class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
                />
              </div>
              
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">
                  数据源B Header行号 <span class="text-red-500">*</span>
                </label>
                <input
                  v-model.number="config.sourceBConfig.header"
                  type="number"
                  min="1"
                  class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
                />
              </div>
            </div>
            
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">
                  时区
                </label>
                <select
                  v-model="config.sourceAConfig.timezone"
                  class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
                >
                  <option v-for="tz in timezones" :key="tz" :value="tz">{{ tz }}</option>
                </select>
              </div>
              
              <div class="flex items-end">
                <label class="flex items-center cursor-pointer">
                  <input
                    v-model="config.sourceAConfig.removeDuplicate"
                    type="checkbox"
                    class="w-5 h-5 text-green-600 border-gray-300 rounded focus:ring-green-500"
                  />
                  <span class="ml-2 text-sm text-gray-700">数据源A启用去重</span>
                </label>
              </div>
            </div>
            
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">
                  数据源B时区
                </label>
                <select
                  v-model="config.sourceBConfig.timezone"
                  class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
                >
                  <option v-for="tz in timezones" :key="tz" :value="tz">{{ tz }}</option>
                </select>
              </div>
              
              <div class="flex items-end">
                <label class="flex items-center cursor-pointer">
                  <input
                    v-model="config.sourceBConfig.removeDuplicate"
                    type="checkbox"
                    class="w-5 h-5 text-green-600 border-gray-300 rounded focus:ring-green-500"
                  />
                  <span class="ml-2 text-sm text-gray-700">数据源B启用去重</span>
                </label>
              </div>
            </div>
          </div>
          
          <div class="flex justify-end mt-8">
            <button
              @click="currentStep = 2"
              :disabled="!step1Valid"
              :class="[
                'px-8 py-3 rounded-lg font-medium text-white transition-colors',
                step1Valid ? 'bg-green-600 hover:bg-green-700' : 'bg-gray-300 cursor-not-allowed'
              ]"
            >
              下一步
            </button>
          </div>
        </div>
        
        <!-- Step 2: Source A Mappings -->
        <div v-else-if="currentStep === 2">
          <div class="bg-white rounded-lg shadow-lg p-8 max-w-6xl mx-auto">
            <h2 class="text-2xl font-bold text-gray-800 mb-6">数据源A字段映射（{{ config.sourceAName }}）</h2>
            <FieldMapping
              :mappings="config.sourceAConfig.mappings"
              :available-columns="[]"
              :allow-manual-input="true"
              @update="config.sourceAConfig.mappings = $event"
              @next="currentStep = 3"
              @back="currentStep = 1"
            />
          </div>
        </div>
        
        <!-- Step 3: Source B Mappings -->
        <div v-else-if="currentStep === 3">
          <div class="bg-white rounded-lg shadow-lg p-8 max-w-6xl mx-auto">
            <h2 class="text-2xl font-bold text-gray-800 mb-6">数据源B字段映射（{{ config.sourceBName }}）</h2>
            <FieldMapping
              :mappings="config.sourceBConfig.mappings"
              :available-columns="[]"
              :allow-manual-input="true"
              @update="config.sourceBConfig.mappings = $event"
              @next="currentStep = 4"
              @back="currentStep = 2"
            />
          </div>
        </div>
        
        <!-- Step 4: Match Config -->
        <div v-else-if="currentStep === 4">
          <MatchConfiguration
            :config="config.matchConfig"
            :source-a-fields="getSourceAFieldNames()"
            :source-b-fields="getSourceBFieldNames()"
            @update="config.matchConfig = $event"
            @finish="saveConfig"
            @back="currentStep = 3"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { message } from '@tauri-apps/plugin-dialog';
import Stepper from './Stepper.vue';
import FieldMapping from './FieldMapping.vue';
import MatchConfiguration from './MatchConfiguration.vue';
import { saveConfigToStorage, getConfigById, TIMEZONES, type ChannelConfig } from '../types';

const props = defineProps<{
  configId?: string;
}>();

const emit = defineEmits<{
  navigate: [page: string];
}>();

const currentStep = ref(1);
const steps = ['基本信息', '数据源A字段', '数据源B字段', '匹配配置'];
const timezones = TIMEZONES;

const config = ref<ChannelConfig>({
  id: `config-${Date.now()}`,
  name: '',
  sourceAName: '',
  sourceBName: '',
  type: '',
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString(),
  sourceAConfig: {
    header: 1,
    timezone: 'America/Sao_Paulo',
    removeDuplicate: false,
    mappings: []
  },
  sourceBConfig: {
    header: 1,
    timezone: 'America/Sao_Paulo',
    removeDuplicate: false,
    mappings: []
  },
  matchConfig: {
    sourceAIdField: '',
    sourceAStatusMapping: [],
    sourceBIdField: '',
    sourceBStatusMapping: [],
    useHistoricalSourceA: false,
    useHistoricalSourceB: false,
    historyDays: 5
  }
});

const step1Valid = computed(() => {
  return config.value.name && config.value.sourceAName && config.value.sourceBName && config.value.type;
});

const isValid = computed(() => {
  return step1Valid.value &&
    config.value.sourceAConfig.mappings.length > 0 &&
    config.value.sourceBConfig.mappings.length > 0 &&
    config.value.matchConfig.sourceAIdField &&
    config.value.matchConfig.sourceBIdField;
});

onMounted(async () => {
  if (props.configId) {
    const existing = await getConfigById(props.configId);
    if (existing) {
      config.value = existing;
    }
  }
});

function getSourceAFieldNames(): string[] {
  return config.value.sourceAConfig.mappings.map(m => m.fieldName).filter(n => n);
}

function getSourceBFieldNames(): string[] {
  return config.value.sourceBConfig.mappings.map(m => m.fieldName).filter(n => n);
}

async function saveConfig() {
  if (!isValid.value) {
    await message('请完整填写所有必填项', {
      title: '提示',
      kind: 'info',
    });
    return;
  }
  
  try {
    config.value.updatedAt = new Date().toISOString();
    
    await saveConfigToStorage(config.value);
    await message('配置保存成功！', {
      title: '操作成功',
      kind: 'info',
    });
    emit('navigate', 'config-list');
  } catch (error) {
    console.error('保存配置失败:', error);
    await message('保存配置失败: ' + error, {
      title: '操作失败',
      kind: 'error',
    });
  }
}
</script>

