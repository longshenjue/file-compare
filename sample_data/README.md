# 示例数据使用指南

本目录包含用于测试对账引擎的示例数据。

## 数据说明

### orders_sample.csv（订单文件）
包含 6 条订单记录：
- ORD-001: 正常订单，状态 PAID
- ORD-002: 正常订单，状态 COMPLETED
- ORD-003: 金额差异订单（银行 3050 vs 订单 3000）
- ORD-004: ID 带后缀订单（需要用 DEL_AFTER 清洗）
- ORD-005: 被拒绝订单（只在订单系统存在）
- ORD-006: 单边订单（只在订单系统存在）

### bank_sample.csv（银行文件）
包含 5 条银行记录：
- E2E-001: 完全匹配
- E2E-002: 完全匹配（状态需归一化）
- E2E-003: 金额差异（3050 vs 3000）
- E2E-004: 需要 ID 清洗才能匹配
- E2E-007: 单边银行记录（只在银行存在）

## 测试步骤

### Step 1: 上传订单文件

1. 银行：Vidi_month
2. 类型：PAYOUT
3. 日期范围：2025-12-30 ~ 2025-12-30
4. 文件：选择 `orders_sample.csv`
5. Header 行号：1
6. 时区：America/Sao_Paulo
7. 去重：不启用
8. 点击"确认"

### Step 2: 上传银行文件

1. 银行：Vidi Bank
2. 类型：PAYOUT
3. 日期范围：2025-12-30 ~ 2025-12-30
4. 文件：选择 `bank_sample.csv`
5. Header 行号：1
6. 时区：America/Sao_Paulo
7. 去重：不启用
8. 点击"确认"

### Step 3: 订单字段映射

添加以下映射规则：

#### 规则 1：订单 ID
- 选择列：e2e
- 类型：OrderString
- 字段名：orderId
- 规则类型：ORDER_STRING_NORMAL
- 格式规则 → Pre → DEL_AFTER → 值：7
  - 说明：保留前 7 个字符，删除后缀（如 E2E-004_SUFFIX → E2E-004）

#### 规则 2：订单状态
- 选择列：status
- 类型：OrderStatus
- 字段名：orderStatus
- 规则类型：ORDER_STATUS_NORMAL

#### 规则 3：订单金额
- 选择列：amount
- 类型：OrderAmount
- 字段名：orderAmount
- 规则类型：ORDER_AMOUNT_NORMAL

点击"下一步"

### Step 4: 银行字段映射

添加以下映射规则：

#### 规则 1：银行 ID
- 选择列：e2eId
- 类型：OrderString
- 字段名：bankId
- 规则类型：ORDER_STRING_NORMAL

#### 规则 2：银行状态
- 选择列：bank_status
- 类型：OrderStatus
- 字段名：bankStatus
- 规则类型：ORDER_STATUS_NORMAL

#### 规则 3：银行金额
- 选择列：transaction_amount
- 类型：OrderAmount
- 字段名：bankAmount
- 规则类型：ORDER_AMOUNT_NORMAL

点击"下一步"

### Step 5: 匹配配置

#### 订单配置
- ID 字段：orderId
- 状态映射：
  - PAID, COMPLETED, SUCCESS → PAID（输入后按回车添加）

#### 银行配置
- ID 字段：bankId
- 状态映射：
  - PAID → PAID

#### 渠道
- VIDI-LT

点击"完成"

### Step 6: 查看结果

预期结果：
- **完全匹配**：2 条（E2E-001, E2E-002）
- **金额差异**：1 条（E2E-003，订单 3000 vs 银行 3050）
- **仅订单存在**：3 条（E2E-005 被拒绝，E2E-006 未到账）
- **仅银行存在**：1 条（E2E-007 多收款）

注意：E2E-004 应该匹配成功，因为我们在 Step 3 中使用 DEL_AFTER 清洗了后缀。

## 故障排查

### 问题：E2E-004 没有匹配

**原因**：忘记添加 DEL_AFTER 格式规则

**解决**：返回 Step 3，为 e2e 列添加格式规则：
- Pre → DEL_AFTER → 7

### 问题：状态不匹配

**原因**：状态映射配置不正确

**解决**：返回 Step 5，确保：
- 订单状态 PAID、COMPLETED、SUCCESS 都映射到 PAID
- 银行状态 PAID 映射到 PAID

### 问题：金额差异未检测

**原因**：金额字段映射错误

**解决**：确保 Step 3 和 Step 4 中正确映射了金额字段，类型为 OrderAmount

## 高级测试场景

### 测试去重功能

修改 `orders_sample.csv`，添加重复记录：

```csv
2025-12-30 10:30:00,ORD-001,E2E-001,1000.00,PAID,VIDI-LT
2025-12-30 10:30:00,ORD-001,E2E-001,1000.00,PAID,VIDI-LT
```

在 Step 1 启用"去重"，系统应只保留一条记录。

### 测试复杂清洗规则

在订单文件中添加：

```csv
2025-12-30 18:00:00,ORD-008,[E2E-008],6000.00,PAID,VIDI-LT
```

在 Step 3 添加格式规则：
- Pre → BRA_VALUE
- 提取括号内容：[E2E-008] → E2E-008

## 数据统计

| 项目 | 订单文件 | 银行文件 |
|------|----------|----------|
| 总记录数 | 6 | 5 |
| 可匹配记录 | 4 | 4 |
| 单边记录 | 2 | 1 |
| 金额差异 | 0 | 1 |

## 提示

1. 使用示例数据前，先阅读 README.md 中的完整使用流程
2. 首次使用建议完全按照本指南操作
3. 熟悉后可以修改示例数据，测试更复杂的场景
4. 导出结果后，用 Excel 或文本编辑器查看详细信息

