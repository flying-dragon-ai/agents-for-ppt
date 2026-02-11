#!/usr/bin/env node

/**
 * PptxGenJS Sidecar
 *
 * 通过 stdin 接收 JSON 请求，生成 PPTX 文件，通过 stdout 返回结果
 */

const PptxGenJS = require('pptxgenjs');
const fs = require('fs');
const path = require('path');

/**
 * 读取 stdin 的所有数据
 */
async function readStdin() {
  return new Promise((resolve, reject) => {
    const chunks = [];
    process.stdin.on('data', chunk => chunks.push(chunk));
    process.stdin.on('end', () => resolve(Buffer.concat(chunks).toString('utf8')));
    process.stdin.on('error', reject);
  });
}

/**
 * 生成 PPTX 文件
 */
async function generatePptx(request) {
  const { slides, output, config } = request;

  // 创建 PPTX 实例
  const pptx = new PptxGenJS();

  // 设置画布尺寸（转换为英寸）
  const widthInches = config.width / 96; // 96 DPI
  const heightInches = config.height / 96;
  pptx.layout = 'LAYOUT_CUSTOM';
  pptx.defineLayout({ name: 'CUSTOM', width: widthInches, height: heightInches });
  pptx.layout = 'CUSTOM';

  // 处理每个幻灯片
  for (const slideData of slides) {
    const slide = pptx.addSlide();

    // 添加内容
    if (slideData.content.type === 'svg') {
      // SVG 内容
      // 注意：PptxGenJS 不直接支持 SVG，需要转换为图片或形状
      // 这里我们将 SVG 作为文本添加（临时方案）
      slide.addText('SVG Content (需要转换)', {
        x: 0.5,
        y: 0.5,
        w: widthInches - 1,
        h: heightInches - 1,
        fontSize: 12,
        color: '666666',
      });
    } else if (slideData.content.type === 'png') {
      // PNG 内容（base64）
      const pngData = slideData.content.data;
      slide.addImage({
        data: `data:image/png;base64,${pngData}`,
        x: 0,
        y: 0,
        w: widthInches,
        h: heightInches,
      });
    }

    // 添加演讲备注
    if (slideData.notes) {
      slide.addNotes(slideData.notes);
    }

    // 添加切换效果
    if (config.enableTransitions && config.transitionType) {
      const transitionMap = {
        'fade': 'fade',
        'push': 'push',
        'wipe': 'wipe',
        'split': 'split',
        'reveal': 'reveal',
      };
      const transition = transitionMap[config.transitionType] || 'fade';
      slide.transition = { type: transition };
    }
  }

  // 确保输出目录存在
  const outputDir = path.dirname(output);
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }

  // 保存 PPTX 文件
  await pptx.writeFile({ fileName: output });

  return { success: true, output };
}

/**
 * 主函数
 */
async function main() {
  try {
    // 读取请求
    const input = await readStdin();
    const request = JSON.parse(input);

    // 生成 PPTX
    const result = await generatePptx(request);

    // 返回结果
    console.log(JSON.stringify(result));
    process.exit(0);
  } catch (error) {
    // 返回错误
    console.log(JSON.stringify({
      success: false,
      error: error.message,
      stack: error.stack,
    }));
    process.exit(1);
  }
}

// 运行主函数
main();
