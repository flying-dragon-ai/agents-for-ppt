# PptxGen Sidecar

PptxGenJS sidecar for PPT Master - 使用 Node.js + PptxGenJS 生成 PPTX 文件。

## 安装

```bash
cd sidecars/pptxgen
npm install
```

## 使用

### 命令行测试

```bash
echo '{"slides":[{"number":1,"title":"Test","content":{"type":"png","data":"..."},"notes":"Test notes"}],"output":"test.pptx","config":{"width":1280,"height":720,"enableTransitions":true,"transitionType":"fade"}}' | node index.js
```

### 从 Rust 调用

Rust 代码会自动通过 stdin/stdout 与 sidecar 通信。

## 请求格式

```json
{
  "slides": [
    {
      "number": 1,
      "title": "Slide Title",
      "content": {
        "type": "svg" | "png",
        "data": "SVG string or base64 PNG"
      },
      "notes": "Speaker notes (optional)"
    }
  ],
  "output": "/path/to/output.pptx",
  "config": {
    "width": 1280,
    "height": 720,
    "enableTransitions": true,
    "transitionType": "fade"
  }
}
```

## 响应格式

成功：
```json
{
  "success": true,
  "output": "/path/to/output.pptx"
}
```

失败：
```json
{
  "success": false,
  "error": "Error message",
  "stack": "Stack trace"
}
```

## 支持的切换效果

- fade
- push
- wipe
- split
- reveal

## 注意事项

1. 需要 Node.js 18+ 环境
2. SVG 内容目前不支持直接嵌入，会转换为占位符文本
3. PNG 内容需要 base64 编码
4. 演讲备注支持 Markdown 格式
