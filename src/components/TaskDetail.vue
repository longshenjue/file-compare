<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { message } from '@tauri-apps/plugin-dialog';
import type { ReconciliationTask, ReconciliationResult } from '../types';

const props = defineProps<{
  taskId: string;
}>();

const emit = defineEmits<{
  navigate: [page: string];
}>();

const task = ref<ReconciliationTask | null>(null);
const result = ref<ReconciliationResult | null>(null);
const loading = ref(false);
const error = ref('');
const doubleCheckDays = ref(5);
const doubleChecking = ref(false);
const activeTab = ref<'matched' | 'onlyInA' | 'onlyInB' | 'diffAmount'>('matched');

const currentData = computed(() => {
  if (!result.value) return [];
  switch (activeTab.value) {
    case 'matched':
      return result.value.matched;
    case 'onlyInA':
      return result.value.onlyInA;
    case 'onlyInB':
      return result.value.onlyInB;
    case 'diffAmount':
      return result.value.diffAmount;
    default:
      return [];
  }
});

// 获取所有列名，按照固定顺序排序
const tableColumns = computed(() => {
  if (currentData.value.length === 0) return [];
  
  // 收集所有行的所有键
  const allKeys = new Set<string>();
  for (const row of currentData.value) {
    for (const key of Object.keys(row)) {
      allKeys.add(key);
    }
  }
  
  // 按照固定顺序排序（参考 exporter.rs 的逻辑）
  const sortedKeys: string[] = [];
  const remainingKeys = Array.from(allKeys);
  
  // 1. ID 字段
  for (const key of remainingKeys) {
    if (key.toLowerCase().includes('id') && !key.includes('normalized') && !sortedKeys.includes(key)) {
      sortedKeys.push(key);
    }
  }
  
  // 2. 时间字段
  for (const key of remainingKeys) {
    if (key.toLowerCase().includes('time') && !sortedKeys.includes(key)) {
      sortedKeys.push(key);
    }
  }
  
  // 3. 日期字段
  for (const key of remainingKeys) {
    if (key.toLowerCase().includes('date') && !sortedKeys.includes(key)) {
      sortedKeys.push(key);
    }
  }
  
  // 4. 金额字段
  for (const key of remainingKeys) {
    if (key.toLowerCase().includes('amount') && !sortedKeys.includes(key)) {
      sortedKeys.push(key);
    }
  }
  
  // 5. 状态字段
  for (const key of remainingKeys) {
    if (key.toLowerCase().includes('status') && !sortedKeys.includes(key)) {
      sortedKeys.push(key);
    }
  }
  
  // 6. 标准化字段
  for (const key of remainingKeys) {
    if (key.startsWith('source') || key.includes('normalized') || key.includes('original')) {
      if (!sortedKeys.includes(key)) {
        sortedKeys.push(key);
      }
    }
  }
  
  // 7. 其他字段（按字母顺序）
  for (const key of remainingKeys) {
    if (!sortedKeys.includes(key)) {
      sortedKeys.push(key);
    }
  }
  
  // 最后按字母顺序排序剩余字段
  sortedKeys.sort();
  
  return sortedKeys;
});

// 格式化时间戳（仅对时间相关字段）
function formatTimestamp(value: any, fieldName?: string): string {
  // 如果字段名包含 amount、price、money 等金额相关关键词，不格式化
  if (fieldName) {
    const fieldLower = fieldName.toLowerCase();
    if (fieldLower.includes('amount') || 
        fieldLower.includes('price') || 
        fieldLower.includes('money') || 
        fieldLower.includes('cost') || 
        fieldLower.includes('fee') ||
        fieldLower.includes('total') ||
        fieldLower.includes('sum')) {
      return String(value);
    }
  }
  
  // 如果字段名不包含 time、date、timestamp 等时间相关关键词，不格式化
  if (fieldName) {
    const fieldLower = fieldName.toLowerCase();
    if (!fieldLower.includes('time') && 
        !fieldLower.includes('date') && 
        !fieldLower.includes('timestamp') &&
        !fieldLower.includes('created') &&
        !fieldLower.includes('updated') &&
        !fieldLower.includes('upload')) {
      return String(value);
    }
  }
  
  // 如果是数字类型的时间戳
  if (typeof value === 'number') {
    let timestamp = value;
    
    // 判断是微秒（16位）还是毫秒（13位）
    if (timestamp > 1000000000000000) {
      // 微秒时间戳，转换为毫秒
      timestamp = Math.floor(timestamp / 1000);
    } else if (timestamp < 1000000000) {
      // 秒时间戳，转换为毫秒
      timestamp = timestamp * 1000;
    }
    
    // 验证时间戳是否在合理范围内（1970-2100年）
    if (timestamp > 0 && timestamp < 4102444800000) {
      try {
        const date = new Date(timestamp);
        // 验证日期是否有效
        if (!isNaN(date.getTime())) {
          return date.toLocaleString('zh-CN', {
            year: 'numeric',
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit',
          });
        }
      } catch (e) {
        // 日期无效，返回原值
      }
    }
  }
  
  // 如果是字符串形式的时间戳
  if (typeof value === 'string' && /^\d+$/.test(value)) {
    let timestamp = parseInt(value);
    
    // 判断是微秒（16位）还是毫秒（13位）
    if (timestamp > 1000000000000000) {
      // 微秒时间戳，转换为毫秒
      timestamp = Math.floor(timestamp / 1000);
    } else if (timestamp < 1000000000) {
      // 秒时间戳，转换为毫秒
      timestamp = timestamp * 1000;
    }
    
    // 验证时间戳是否在合理范围内（1970-2100年）
    if (timestamp > 0 && timestamp < 4102444800000) {
      try {
        const date = new Date(timestamp);
        // 验证日期是否有效
        if (!isNaN(date.getTime())) {
          return date.toLocaleString('zh-CN', {
            year: 'numeric',
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit',
          });
        }
      } catch (e) {
        // 日期无效，返回原值
      }
    }
  }
  
  // 如果不是时间戳，直接返回原值
  return String(value);
}

async function loadTaskDetail() {
  loading.value = true;
  error.value = '';
  try {
    // 加载任务列表找到当前任务
    const tasks = await invoke<ReconciliationTask[]>('load_tasks');
    task.value = tasks.find(t => t.taskId === props.taskId) || null;

    if (!task.value) {
      error.value = '任务不存在';
      return;
    }

    // 加载详细结果
    result.value = await invoke<ReconciliationResult>('load_task_result', { taskId: props.taskId });
  } catch (e: any) {
    error.value = `加载失败: ${e}`;
  } finally {
    loading.value = false;
  }
}

async function performDoubleCheck() {
  const confirmed = await message(`将扩大时间范围到前后 ${doubleCheckDays.value} 天进行重新对账，确认继续？`, {
    title: 'Double Check',
    kind: 'info',
    okLabel: '确认继续',
  });
  
  if (!confirmed) {
    return;
  }

  doubleChecking.value = true;
  error.value = '';
  
  try {
    const [newTask, newResult] = await invoke<[ReconciliationTask, ReconciliationResult]>('double_check_task', {
      taskId: props.taskId,
      extendedDays: doubleCheckDays.value,
    });

    // 更新当前显示
    task.value = newTask;
    result.value = newResult;

    await message(`Double Check 完成！\n\n完全匹配: ${newTask.stats.matchedCount}\n仅数据源A: ${newTask.stats.onlyInSourceACount}\n仅数据源B: ${newTask.stats.onlyInSourceBCount}\n金额差异: ${newTask.stats.diffAmountCount}`, {
      title: '操作成功',
      kind: 'info',
    });
  } catch (e: any) {
    console.error('Double Check 失败:', e);
    error.value = `Double Check 失败: ${e}`;
    await message(`Double Check 失败: ${e}`, {
      title: '操作失败',
      kind: 'error',
    });
  } finally {
    doubleChecking.value = false;
  }
}

async function downloadData(type: string) {
  try {
    const { save } = await import('@tauri-apps/plugin-dialog');
    const filePath = await save({
      defaultPath: `${type}_${task.value?.taskId}.csv`,
      filters: [{ name: 'CSV Files', extensions: ['csv'] }],
    });

    if (filePath) {
      await invoke('export_results', {
        results: result.value,
        exportType: type,
        filePath,
      });
      await message('导出成功！', {
        title: '操作成功',
        kind: 'info',
      });
    }
  } catch (e: any) {
    await message(`导出失败: ${e}`, {
      title: '操作失败',
      kind: 'error',
    });
  }
}

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleString('zh-CN');
}

onMounted(() => {
  loadTaskDetail();
});
</script>

<template>
  <div class="min-h-screen bg-gradient-to-br from-purple-50 to-green-50 p-8">
    <div class="max-w-7xl mx-auto">
      <button
        @click="emit('navigate', 'task-list')"
        class="text-purple-600 hover:text-purple-800 mb-4 flex items-center gap-2"
      >
        ← 返回列表
      </button>

      <div v-if="loading" class="bg-white rounded-xl shadow-lg p-12 text-center">
        <p class="text-gray-500">加载中...</p>
      </div>

      <div v-else-if="error" class="bg-red-50 border border-red-200 rounded-lg p-4">
        <p class="text-red-700">{{ error }}</p>
      </div>

      <div v-else-if="task && result" class="space-y-6">
        <!-- 任务信息卡片 -->
        <div class="bg-white rounded-xl shadow-lg p-6">
          <div class="flex items-start justify-between mb-6">
            <div>
              <div class="flex items-center gap-3 mb-2">
                <h1 class="text-3xl font-bold text-gray-800">{{ task.taskName }}</h1>
                <span
                  class="px-3 py-1 rounded-full text-sm font-medium"
                  :class="task.taskType === 'PAYOUT' ? 'bg-orange-100 text-orange-700' : 'bg-blue-100 text-blue-700'"
                >
                  {{ task.taskType }}
                </span>
              </div>
              <p class="text-gray-600">任务ID: {{ task.taskId }}</p>
              <p class="text-gray-600">创建时间: {{ formatDate(task.createdAt) }}</p>
            </div>
          </div>

          <!-- 统计概览 -->
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
            <div class="bg-green-50 rounded-lg p-4 border-2 border-transparent hover:border-green-500 transition-colors">
              <p class="text-sm text-gray-600">✅ 完全匹配</p>
              <p class="text-3xl font-bold text-green-600">{{ task.stats.matchedCount }}</p>
            </div>
            <div class="bg-orange-50 rounded-lg p-4 border-2 border-transparent hover:border-orange-500 transition-colors">
              <p class="text-sm text-gray-600">📋 仅数据源A存在</p>
              <p class="text-3xl font-bold text-orange-600">{{ task.stats.onlyInSourceACount }}</p>
            </div>
            <div class="bg-blue-50 rounded-lg p-4 border-2 border-transparent hover:border-blue-500 transition-colors">
              <p class="text-sm text-gray-600">🏦 仅数据源B存在</p>
              <p class="text-3xl font-bold text-blue-600">{{ task.stats.onlyInSourceBCount }}</p>
            </div>
            <div class="bg-red-50 rounded-lg p-4 border-2 border-transparent hover:border-red-500 transition-colors">
              <p class="text-sm text-gray-600">💰 金额差异</p>
              <p class="text-3xl font-bold text-red-600">{{ task.stats.diffAmountCount }}</p>
            </div>
          </div>

          <!-- Double Check 功能区 -->
          <div v-if="task.stats.onlyInSourceACount > 0 || task.stats.onlyInSourceBCount > 0" class="bg-gradient-to-r from-purple-50 to-indigo-50 rounded-lg p-6 border-2 border-purple-200">
            <h3 class="text-lg font-semibold text-gray-800 mb-3">🔄 Double Check - 扩大时间范围重新对账</h3>
            <p class="text-gray-600 mb-4">
              当前有 {{ task.stats.onlyInSourceACount + task.stats.onlyInSourceBCount }} 条单边账。可能是因为系统时间延迟导致订单分散在不同日期文件中。使用 Double Check 扩大历史数据范围重新对账，可以消除这类差异。
            </p>

            <div class="flex items-center gap-4">
              <label class="text-sm font-medium text-gray-700">时间范围：前后</label>
              <input
                v-model.number="doubleCheckDays"
                type="number"
                min="1"
                max="30"
                class="w-24 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
              />
              <span class="text-sm text-gray-600">天</span>

              <button
                @click="performDoubleCheck"
                :disabled="doubleChecking"
                class="ml-auto px-6 py-3 bg-gradient-to-r from-purple-600 to-indigo-600 text-white rounded-lg hover:from-purple-700 hover:to-indigo-700 transition-all disabled:opacity-50 disabled:cursor-not-allowed font-semibold shadow-lg"
              >
                {{ doubleChecking ? '处理中...' : '🚀 执行 Double Check' }}
              </button>
            </div>
          </div>

          <!-- 操作按钮 -->
          <div class="flex gap-3 mt-6">
            <button
              @click="downloadData('all')"
              class="px-6 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
            >
              📥 下载全部数据
            </button>
            <button
              @click="downloadData('matched')"
              class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            >
              下载匹配数据
            </button>
            <button
              @click="downloadData('diff')"
              class="px-6 py-2 bg-orange-600 text-white rounded-lg hover:bg-orange-700 transition-colors"
            >
              下载差异数据
            </button>
          </div>
        </div>

        <!-- 详细数据表格 -->
        <div class="bg-white rounded-xl shadow-lg p-6">
          <div class="flex gap-2 mb-6 border-b border-gray-200">
            <button
              @click="activeTab = 'matched'"
              :class="[
                'px-6 py-3 font-medium transition-colors',
                activeTab === 'matched'
                  ? 'text-green-600 border-b-2 border-green-600'
                  : 'text-gray-500 hover:text-gray-700'
              ]"
            >
              完全匹配 ({{ result.matched.length }})
            </button>
            <button
              @click="activeTab = 'onlyInA'"
              :class="[
                'px-6 py-3 font-medium transition-colors',
                activeTab === 'onlyInA'
                  ? 'text-orange-600 border-b-2 border-orange-600'
                  : 'text-gray-500 hover:text-gray-700'
              ]"
            >
              仅订单 ({{ result.onlyInA.length }})
            </button>
            <button
              @click="activeTab = 'onlyInB'"
              :class="[
                'px-6 py-3 font-medium transition-colors',
                activeTab === 'onlyInB'
                  ? 'text-blue-600 border-b-2 border-blue-600'
                  : 'text-gray-500 hover:text-gray-700'
              ]"
            >
              仅银行 ({{ result.onlyInB.length }})
            </button>
            <button
              @click="activeTab = 'diffAmount'"
              :class="[
                'px-6 py-3 font-medium transition-colors',
                activeTab === 'diffAmount'
                  ? 'text-red-600 border-b-2 border-red-600'
                  : 'text-gray-500 hover:text-gray-700'
              ]"
            >
              金额差异 ({{ result.diffAmount.length }})
            </button>
          </div>

          <div v-if="currentData.length === 0" class="text-center py-12 text-gray-500">
            暂无数据
          </div>

          <div v-else class="overflow-x-auto">
            <table class="w-full text-sm">
              <thead class="bg-gray-50">
                <tr>
                  <th v-for="(key, index) in tableColumns" :key="index" class="px-4 py-3 text-left font-medium text-gray-700 border-b">
                    {{ key }}
                  </th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(row, rowIndex) in currentData.slice(0, 100)" :key="rowIndex" class="border-b hover:bg-gray-50">
                  <td v-for="(key, colIndex) in tableColumns" :key="colIndex" class="px-4 py-3 text-gray-600">
                    {{ formatTimestamp(row[key], key) }}
                  </td>
                </tr>
              </tbody>
            </table>
            <p v-if="currentData.length > 100" class="text-center text-gray-500 mt-4">
              仅显示前 100 条，共 {{ currentData.length }} 条。请下载完整数据查看。
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

