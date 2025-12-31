<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { message } from '@tauri-apps/plugin-dialog';
import type { ReconciliationTask } from '../types';

const emit = defineEmits<{
  navigate: [page: string, taskId?: string];
}>();

const tasks = ref<ReconciliationTask[]>([]);
const loading = ref(false);
const error = ref('');

async function loadTasks() {
  loading.value = true;
  error.value = '';
  try {
    tasks.value = await invoke<ReconciliationTask[]>('load_tasks');
    // 按创建时间倒序排序
    tasks.value.sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());
  } catch (e: any) {
    error.value = `加载任务失败: ${e}`;
  } finally {
    loading.value = false;
  }
}

async function deleteTask(taskId: string) {
  const confirmed = await message('确认删除此对账记录？', {
    title: '确认删除',
    kind: 'warning',
    okLabel: '删除',
  });
  
  if (!confirmed) {
    return;
  }

  try {
    await invoke('delete_task', { taskId });
    await loadTasks();
  } catch (e: any) {
    await message(`删除失败: ${e}`, {
      title: '操作失败',
      kind: 'error',
    });
  }
}

function viewTask(taskId: string) {
  emit('navigate', 'task-detail', taskId);
}

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleString('zh-CN');
}

onMounted(() => {
  loadTasks();
});
</script>

<template>
  <div class="min-h-screen bg-gradient-to-br from-purple-50 to-green-50 p-8">
    <div class="max-w-7xl mx-auto">
      <div class="flex items-center justify-between mb-8">
        <div>
          <button
            @click="emit('navigate', 'home')"
            class="text-purple-600 hover:text-purple-800 mb-4 flex items-center gap-2"
          >
            ← 返回首页
          </button>
          <h1 class="text-3xl font-bold text-gray-800">对账历史记录</h1>
          <p class="text-gray-600 mt-2">查看所有对账任务记录</p>
        </div>

        <button
          @click="loadTasks"
          class="px-6 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
          :disabled="loading"
        >
          {{ loading ? '加载中...' : '刷新' }}
        </button>
      </div>

      <div v-if="error" class="bg-red-50 border border-red-200 rounded-lg p-4 mb-6">
        <p class="text-red-700">{{ error }}</p>
      </div>

      <div v-if="tasks.length === 0 && !loading" class="bg-white rounded-xl shadow-lg p-12 text-center">
        <p class="text-gray-500 text-lg">暂无对账记录</p>
        <button
          @click="emit('navigate', 'reconcile')"
          class="mt-4 px-6 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
        >
          开始对账
        </button>
      </div>

      <div v-else class="grid gap-4">
        <div
          v-for="task in tasks"
          :key="task.taskId"
          class="bg-white rounded-xl shadow-lg p-6 hover:shadow-xl transition-shadow cursor-pointer"
          @click="viewTask(task.taskId)"
        >
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-center gap-3 mb-2">
                <h3 class="text-xl font-semibold text-gray-800">{{ task.taskName }}</h3>
                <span
                  class="px-3 py-1 rounded-full text-sm font-medium"
                  :class="task.taskType === 'PAYOUT' ? 'bg-orange-100 text-orange-700' : 'bg-blue-100 text-blue-700'"
                >
                  {{ task.taskType }}
                </span>
                <span v-if="task.usedHistoricalSourceA || task.usedHistoricalSourceB" class="px-3 py-1 rounded-full text-sm font-medium bg-purple-100 text-purple-700">
                  使用历史数据
                </span>
              </div>

              <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mt-4">
                <div class="bg-green-50 rounded-lg p-3">
                  <p class="text-sm text-gray-600">完全匹配</p>
                  <p class="text-2xl font-bold text-green-600">{{ task.stats.matchedCount }}</p>
                </div>
                <div class="bg-orange-50 rounded-lg p-3">
                  <p class="text-sm text-gray-600">仅数据源A</p>
                  <p class="text-2xl font-bold text-orange-600">{{ task.stats.onlyInSourceACount }}</p>
                </div>
                <div class="bg-blue-50 rounded-lg p-3">
                  <p class="text-sm text-gray-600">仅数据源B</p>
                  <p class="text-2xl font-bold text-blue-600">{{ task.stats.onlyInSourceBCount }}</p>
                </div>
                <div class="bg-red-50 rounded-lg p-3">
                  <p class="text-sm text-gray-600">金额差异</p>
                  <p class="text-2xl font-bold text-red-600">{{ task.stats.diffAmountCount }}</p>
                </div>
              </div>

              <div class="mt-4 text-sm text-gray-600 space-y-1">
                <p><span class="font-medium">配置：</span>{{ task.configName }}</p>
                <p><span class="font-medium">数据源A：</span>{{ task.sourceAName }}</p>
                <p><span class="font-medium">数据源B：</span>{{ task.sourceBName }}</p>
                <p><span class="font-medium">日期：</span>{{ task.dateRange.start }} ~ {{ task.dateRange.end }}</p>
                <p><span class="font-medium">创建时间：</span>{{ formatDate(task.createdAt) }}</p>
                <p><span class="font-medium">数据源A文件：</span>{{ task.sourceAFileName }}</p>
                <p><span class="font-medium">数据源B文件：</span>{{ task.sourceBFileName }}</p>
              </div>
            </div>

            <button
              @click.stop="deleteTask(task.taskId)"
              class="ml-4 px-4 py-2 text-red-600 hover:bg-red-50 rounded-lg transition-colors"
            >
              删除
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

