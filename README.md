# EzShark

## feature

- [ ] 默认配置文件
- [ ] session 功能
- [ ] 导入 session
- [ ] 导出 session
- [ ] 请求断点
- [ ] 关闭开启上级代理
- [ ] 关闭开启抓包显示
- [ ] 请求过滤
- [ ] 配置自定义端口

## todo

1. 去除全局的 unwarp 改为在 UI 报错
2. 考虑去除 anyhow
3. 写第一个版本 UI
4. 增加默认配置文件 启动读取配置文件

## 前端

1. 用 pnpm 代替 npm

### 启动命令

```bash
pnpm tauri dev
```

### 打包

```bash
pnpm tauri build
```

#### 打包输出位置：

- Windows: src-tauri/target/release/bundle/
