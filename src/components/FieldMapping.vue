<template>
  <div class="bg-white rounded-lg shadow-lg p-8 max-w-6xl mx-auto">
    <div class="flex justify-between items-center mb-6">
      <h2 class="text-2xl font-bold text-gray-800">字段映射与清洗规则</h2>
      <button
        @click="addMapping"
        class="px-6 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
      >
        添加规则
      </button>
    </div>
    
    <div class="space-y-6">
      <div
        v-for="(mapping, index) in localMappings"
        :key="mapping.id"
        class="border border-gray-200 rounded-lg p-6 relative"
      >
        <!-- Delete Button -->
        <button
          v-if="localMappings.length > 1"
          @click="removeMapping(index)"
          class="absolute top-4 right-4 text-red-500 hover:text-red-700"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
        </button>
        
        <div class="grid grid-cols-2 gap-4 mb-4">
          <!-- Source Column -->
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-2">
              {{ allowManualInput ? '源列名' : '选择样本列' }} <span class="text-red-500">*</span>
            </label>
            <input
              v-if="allowManualInput"
              v-model="mapping.sourceColumn"
              type="text"
              placeholder="输入CSV文件中的列名"
              class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
            />
            <select
              v-else
              v-model="mapping.sourceColumn"
              class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
            >
              <option value="">请选择列</option>
              <option v-for="col in availableColumns" :key="col" :value="col">
                {{ col }}
              </option>
            </select>
          </div>
          
          <!-- Field Type -->
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-2">
              类型 <span class="text-red-500">*</span>
            </label>
            <select
              v-model="mapping.fieldType"
              class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
            >
              <option value="">请选择类型</option>
              <option value="OrderTime">OrderTime</option>
              <option value="OrderStatus">OrderStatus</option>
              <option value="OrderString">OrderString</option>
              <option value="OrderAmount">OrderAmount</option>
            </select>
          </div>
        </div>
        
        <div class="grid grid-cols-2 gap-4 mb-4">
          <!-- Field Name -->
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-2">
              字段名 <span class="text-red-500">*</span>
            </label>
            <input
              v-model="mapping.fieldName"
              type="text"
              placeholder="e.g., orderTime, e2eId"
              class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
            />
          </div>
          
          <!-- Rule Type -->
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-2">
              规则类型 <span class="text-red-500">*</span>
            </label>
            <select
              v-model="mapping.ruleType"
              class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
            >
              <option value="">请选择规则</option>
              <option value="ORDER_TIME_NORMAL">ORDER_TIME_NORMAL</option>
              <option value="ORDER_TIME_TIMESTAMP">ORDER_TIME_TIMESTAMP</option>
              <option value="ORDER_STATUS_NORMAL">ORDER_STATUS_NORMAL</option>
              <option value="ORDER_STRING_NORMAL">ORDER_STRING_NORMAL</option>
              <option value="ORDER_AMOUNT_NORMAL">ORDER_AMOUNT_NORMAL</option>
            </select>
          </div>
        </div>
        
        <!-- Rule Config -->
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-2">
            规则配置
          </label>
          <input
            v-model="mapping.ruleConfig"
            type="text"
            placeholder="配置参数"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
          />
        </div>
        
        <!-- Save Original Data -->
        <div class="mb-4">
          <label class="flex items-center cursor-pointer">
            <input
              v-model="mapping.saveOriginal"
              type="checkbox"
              class="w-5 h-5 text-green-600 border-gray-300 rounded focus:ring-green-500"
            />
            <span class="ml-2 text-sm text-gray-700">保存原始数据</span>
          </label>
        </div>
        
        <!-- Format Rules -->
        <div class="border-t pt-4">
          <div class="flex justify-between items-center mb-4">
            <h3 class="text-lg font-semibold text-gray-800">格式化规则</h3>
            <button
              @click="addFormatRule(index)"
              class="px-4 py-1 text-sm bg-teal-600 text-white rounded hover:bg-teal-700 transition-colors"
            >
              添加格式规则
            </button>
          </div>
          
          <div
            v-for="(rule, rIndex) in mapping.formatRules"
            :key="rIndex"
            class="grid grid-cols-3 gap-4 mb-3"
          >
            <div>
              <select
                v-model="rule.type"
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 text-sm"
              >
                <option value="pre">Pre</option>
                <option value="post">Post</option>
              </select>
            </div>
            <div>
              <select
                v-model="rule.operation"
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 text-sm"
              >
                <option value="">选择操作</option>
                <option value="DEL_PRE">DEL_PRE</option>
                <option value="DEL_AFTER">DEL_AFTER</option>
                <option value="DEL_CHAR">DEL_CHAR</option>
                <option value="REPLACE_TWO_CHAR">REPLACE_TWO_CHAR</option>
                <option value="BRA_VALUE">BRA_VALUE</option>
                <option value="DIVIDE_NUMBER">DIVIDE_NUMBER</option>
                <option value="ABS_VALUE">ABS_VALUE</option>
                <option value="ADD_CHAR_PRE">ADD_CHAR_PRE</option>
                <option value="ADD_CHAR_AFTER">ADD_CHAR_AFTER</option>
                <option value="XENDIT_TIME">XENDIT_TIME</option>
              </select>
            </div>
            <div class="flex items-center space-x-2">
              <input
                v-model="rule.value"
                type="text"
                placeholder="值"
                class="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 text-sm"
              />
              <button
                @click="removeFormatRule(index, rIndex)"
                class="text-red-500 hover:text-red-700"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
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
        @click="handleNext"
        :disabled="!isValid"
        :class="[
          'px-8 py-3 rounded-lg font-medium text-white transition-colors',
          isValid
            ? 'bg-green-600 hover:bg-green-700'
            : 'bg-gray-300 cursor-not-allowed'
        ]"
      >
        下一步
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type { ColumnMapping } from '../types';

const props = defineProps<{
  mappings: ColumnMapping[];
  availableColumns: string[];
  allowManualInput?: boolean; // 允许手动输入列名（配置阶段）
}>();

const emit = defineEmits<{
  update: [mappings: ColumnMapping[]];
  next: [];
  back: [];
}>();

const localMappings = ref<ColumnMapping[]>([...props.mappings]);

watch(
  localMappings,
  (newVal) => {
    emit('update', [...newVal]);
  },
  { deep: true }
);

const isValid = computed(() => {
  return localMappings.value.every(
    (m) => m.sourceColumn && m.fieldType && m.fieldName && m.ruleType
  );
});

function addMapping() {
  localMappings.value.push({
    id: `mapping-${Date.now()}`,
    sourceColumn: '',
    fieldType: 'OrderString',
    fieldName: '',
    ruleType: '',
    ruleConfig: '',
    saveOriginal: false,
    formatRules: []
  });
}

function removeMapping(index: number) {
  localMappings.value.splice(index, 1);
}

function addFormatRule(mappingIndex: number) {
  localMappings.value[mappingIndex].formatRules.push({
    type: 'pre',
    operation: '',
    value: ''
  });
}

function removeFormatRule(mappingIndex: number, ruleIndex: number) {
  localMappings.value[mappingIndex].formatRules.splice(ruleIndex, 1);
}

function handleNext() {
  if (isValid.value) {
    emit('next');
  }
}
</script>

