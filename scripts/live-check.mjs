// 服务商实测，只使用固定测试文本；密钥由服务端环境提供，不写入客户端或输出。
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {SiliconFlowProvider} from '../services/api/provider.mjs';
const provider=new SiliconFlowProvider();
if(!provider.ready)throw new Error('请先在服务端配置模型服务');
const word=await provider.translate('apple','en','zh');assert.ok(word.translation);assert.equal(word.unit,'word');
const sentence=await provider.translate('我想学习外语。','zh','ja',undefined,'conversational');assert.ok(sentence.translation);
const lesson=await provider.coach('I go to school yesterday.','en');assert.ok(lesson.correctedText);assert.ok(lesson.motherMeaning);
const transcript=process.argv[2]?await provider.transcribe(readFileSync(process.argv[2])):undefined;
console.log(JSON.stringify({word:word.translation,japanese:sentence.translation,correction:lesson.correctedText,meaning:lesson.motherMeaning,transcript}));
