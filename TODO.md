# Jed 开发 TODO

> 按优先级排序的待实现功能清单
> 目标：让 AI Agent 能更低成本地使用 jed

---

## ✅ 已完成

### P0 任务 (全部完成)
- ✅ P0-1: 命令行顺序修正 - 支持 `jed cmd path file.json`
- ✅ P0-2: 抑制编译输出 - 添加 --quiet 选项
- ✅ P0-3: schema JSON 输出 - 使用 --json 输出标准 JSON Schema
- ✅ P0-4: --lang 选项 - 支持 en, zh-CN, zh-TW

### P1 任务 (大部分完成)
- ✅ P1-5: Filter/Query 命令
- ✅ P1-6: tree 命令
- ✅ P1-7: validate 命令
- ⚠️ P1-8: diff --json (已有 --json 支持)

### P2 任务 (部分完成)
- ✅ P2-9: 机器可读的自我发现 (commands/explain)
- ✅ P2-10: 输出格式转换 (yaml 支持)
- ⚠️ P2-13: 语法高亮 (TUI 已实现，但可以增强)
- ⚠️ P2-14: 大文件支持 (未开始)

### 清理任务
- ✅ 清理编译警告

---

## 📋 待完成

### P1
- P1-8: diff 命令添加 --json 模式

### P2
- P2-11: 批处理编辑模式
- P2-12: 交互式 Shell

### P3 (长期)
- P3-15: 多文件标签页
- P3-16: 文档和示例
- P3-17: 发布到包管理器

---

## 使用示例

```bash
# 命令行顺序 - 支持两种方式
jed get .name file.json              # 推荐
jed --file file.json get .name      # 使用选项

# 自我发现
jed commands                         # 列出所有命令
jed explain get                      # 获取命令帮助

# 新命令
jed tree file.json                   # 树形展示
jed query '.users[0]' file.json      # 查询过滤
jed validate schema.json data.json   # Schema 验证
jed convert yaml file.json           # 输出 YAML

# 语言控制
jed --lang en get .name file.json    # 英文输出

# JSON 输出 (适用于所有支持命令)
jed --json schema file.json
```

---

## 修改历史

### 2024-xx-xx
- 所有 P0 任务完成
- 新增 commands, explain, tree, query, validate, convert 命令
- 支持灵活的文件参数位置
