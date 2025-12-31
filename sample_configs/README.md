# 配置文件示例说明

本目录包含多个可直接导入的配置文件示例，帮助您快速开始使用对账引擎。

---

## 📁 配置文件列表

### 1. VIDI-LT-PAYOUT.json ⭐
**用途**：VIDI-LT 支付渠道配置（与示例数据配套）

**特点**：
- ✅ 与 `sample_data/` 中的示例文件完全匹配
- ✅ 包含 DEL_AFTER 清洗规则示例
- ✅ 多状态归一化示例
- ✅ 适合新手测试

**字段映射**：
- 订单：transaction_date, e2e, status, amount
- 银行：date, e2eId, bank_status, transaction_amount

**关键配置**：
- 订单 e2e 字段：保留前 7 个字符（删除后缀）
- 状态归一化：PAID/COMPLETED/SUCCESS → PAID

---

### 2. Xendit-PAYIN.json
**用途**：Xendit 收款渠道配置

**特点**：
- ✅ PAYIN 类型示例
- ✅ 时间格式转换（XENDIT_TIME）
- ✅ 金额单位转换（分转元）
- ✅ 印尼时区（Asia/Jakarta）

**清洗规则示例**：
```json
"formatRules": [
  {
    "type": "pre",
    "operation": "XENDIT_TIME",
    "value": ""
  }
]
```

```json
"formatRules": [
  {
    "type": "pre",
    "operation": "DIVIDE_NUMBER",
    "value": "100"
  }
]
```

---

### 3. Generic-PAYOUT-Template.json
**用途**：通用 PAYOUT 模板

**特点**：
- ✅ 最小化配置
- ✅ 适合作为基础模板
- ✅ 无复杂清洗规则
- ✅ 易于理解和修改

**适用场景**：
- 创建新配置的起点
- 简单对账需求
- 学习配置结构

---

### 4. Advanced-Example.json
**用途**：高级配置示例

**特点**：
- ✅ 展示多种清洗规则
- ✅ 规则叠加示例
- ✅ 复杂状态映射
- ✅ 多字段映射

**包含的清洗规则**：
- BRA_VALUE - 提取括号内容
- DEL_AFTER - 截断字符串
- DEL_CHAR - 删除字符
- DIVIDE_NUMBER - 数值除法
- ABS_VALUE - 绝对值
- ADD_CHAR_PRE - 前置添加

**适用场景**：
- 复杂数据格式
- 需要多步清洗
- 学习高级用法

---

## 🚀 使用指南

### 方法一：直接导入（推荐）

1. **启动应用**
   ```bash
   npm run tauri dev
   ```

2. **导入配置**
   - 主页 → "渠道解析配置"
   - 点击"导入配置"按钮
   - 选择 `sample_configs/` 中的 JSON 文件
   - 配置自动导入并保存

3. **验证配置**
   - 配置列表中查看导入的配置
   - 点击"编辑"查看详细配置

4. **执行对账**
   - 主页 → "执行对账"
   - 选择导入的配置
   - 上传对应的文件进行测试

---

### 方法二：复制修改

1. **复制配置文件**
   ```bash
   cp sample_configs/VIDI-LT-PAYOUT.json my-config.json
   ```

2. **编辑配置**
   - 修改 `name`、`bank` 等基本信息
   - 调整 `sourceColumn` 为实际列名
   - 修改清洗规则和状态映射

3. **导入修改后的配置**
   - 在应用中导入编辑后的文件

---

## 📝 配置文件结构说明

### 基本信息
```json
{
  "id": "唯一标识（导入时会自动生成新ID）",
  "name": "配置名称（显示在界面上）",
  "bank": "银行或渠道名称",
  "type": "PAYOUT 或 PAYIN",
  "createdAt": "创建时间",
  "updatedAt": "更新时间"
}
```

### orderConfig（订单配置）
```json
{
  "header": 1,  // CSV 文件 Header 行号
  "timezone": "America/Sao_Paulo",  // 时区
  "removeDuplicate": true,  // 是否去重
  "mappings": [...]  // 字段映射数组
}
```

### 字段映射（ColumnMapping）
```json
{
  "id": "唯一标识",
  "sourceColumn": "CSV 中的原始列名",
  "fieldType": "OrderTime|OrderStatus|OrderString|OrderAmount",
  "fieldName": "映射后的标准字段名",
  "ruleType": "ORDER_XXX_NORMAL",
  "ruleConfig": "规则配置参数（可选）",
  "saveOriginal": true,  // 是否保存原始数据
  "formatRules": [...]  // 清洗规则数组
}
```

### 清洗规则（FormatRule）
```json
{
  "type": "pre|post",  // pre: 前置处理, post: 后置处理
  "operation": "DEL_AFTER|BRA_VALUE|...",  // 操作类型
  "value": "操作参数"
}
```

### 匹配配置（MatchConfig）
```json
{
  "orderIdField": "用于关联的订单ID字段",
  "orderStatusMapping": [
    {
      "sourceStatus": ["PAID", "SUCCESS"],  // 源状态列表
      "targetStatus": "PAID"  // 归一化后的状态
    }
  ],
  "bankIdField": "用于关联的银行ID字段",
  "bankStatusMapping": [...]
}
```

---

## 🎯 测试流程

### 使用 VIDI-LT-PAYOUT.json 测试

1. **导入配置**
   ```
   导入 sample_configs/VIDI-LT-PAYOUT.json
   ```

2. **准备文件**
   ```
   订单文件: sample_data/orders_sample.csv
   银行文件: sample_data/bank_sample.csv
   ```

3. **执行对账**
   ```
   1. 选择 "VIDI-LT PAYOUT 渠道" 配置
   2. 上传订单文件
   3. 上传银行文件
   4. 点击"执行对账"
   ```

4. **预期结果**
   ```
   完全匹配: 2 条 (E2E-001, E2E-002)
   金额差异: 1 条 (E2E-003)
   仅订单存在: 3 条
   仅银行存在: 1 条
   ```

---

## 🔧 自定义配置指南

### 步骤 1：确定基础信息
```json
{
  "name": "您的渠道名称",
  "bank": "银行名称",
  "type": "PAYOUT 或 PAYIN"
}
```

### 步骤 2：配置字段映射

**订单字段（必须）**：
- ✅ ID 字段（OrderString）- 用于关联
- ✅ 状态字段（OrderStatus）- 用于匹配
- ✅ 金额字段（OrderAmount）- 用于核对
- ⭐ 时间字段（OrderTime）- 可选

**银行字段（必须）**：
- ✅ ID 字段（OrderString）
- ✅ 状态字段（OrderStatus）
- ✅ 金额字段（OrderAmount）

### 步骤 3：添加清洗规则

**常见场景**：

1. **删除 ID 后缀**
   ```json
   {
     "type": "pre",
     "operation": "DEL_AFTER",
     "value": "10"
   }
   ```

2. **金额单位转换（分→元）**
   ```json
   {
     "type": "pre",
     "operation": "DIVIDE_NUMBER",
     "value": "100"
   }
   ```

3. **提取括号内容**
   ```json
   {
     "type": "pre",
     "operation": "BRA_VALUE",
     "value": ""
   }
   ```

4. **删除特殊字符**
   ```json
   {
     "type": "pre",
     "operation": "DEL_CHAR",
     "value": "-"
   }
   ```

### 步骤 4：配置状态映射

```json
{
  "orderStatusMapping": [
    {
      "sourceStatus": ["PAID", "COMPLETED", "SUCCESS"],
      "targetStatus": "PAID"
    },
    {
      "sourceStatus": ["FAILED", "REJECTED"],
      "targetStatus": "FAILED"
    }
  ]
}
```

---

## ⚠️ 常见问题

### Q1：导入后字段名不匹配怎么办？

**A**：导入配置后，在"编辑配置"中修改 `sourceColumn` 为实际的 CSV 列名。

---

### Q2：如何测试配置是否正确？

**A**：
1. 使用小文件测试（10-20 条记录）
2. 检查对账结果是否符合预期
3. 查看"金额差异"和"单边账"的原因

---

### Q3：配置文件可以手动编辑吗？

**A**：可以！JSON 格式，可以用任何文本编辑器修改。注意：
- 保持 JSON 格式正确
- ID 会在导入时自动生成
- 时间戳格式：ISO 8601

---

### Q4：如何备份配置？

**A**：
1. 方法一：导出配置到安全位置
2. 方法二：复制 `~/.file-compare/reconciliation_configs/configs.json`

---

## 📚 相关文档

- **v2.1-更新说明.md** - 配置文件功能详解
- **QUICKSTART.md** - 快速上手指南
- **使用指南.md** - 完整使用教程
- **sample_data/README.md** - 示例数据说明

---

## 💡 最佳实践

1. **命名规范**
   - 格式：`渠道名-类型-环境`
   - 示例：`VIDI-LT-PAYOUT-PROD`、`Xendit-PAYIN-TEST`

2. **版本管理**
   - 定期导出配置
   - 以日期命名：`config-2025-12-30.json`
   - 使用 Git 管理配置文件

3. **团队协作**
   - 共享配置仓库
   - 统一命名规范
   - 添加配置说明注释（在 name 字段）

4. **测试流程**
   - 新配置先用测试数据验证
   - 确认无误后再用生产数据
   - 保留旧配置作为备份

---

**祝您对账顺利！** 🎉

如有问题，请查看完整文档或提交反馈。

