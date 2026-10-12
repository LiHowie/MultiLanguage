import http from 'node:http';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { resolve, extname, sep } from 'node:path';
import { randomUUID } from 'node:crypto';
import { Store } from './database.mjs';
import { SiliconFlowProvider } from './provider.mjs';
import { ApiError, bad, text, language, consent } from './errors.mjs';
import { decodeAudio } from './audio.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url));
const assets = { '/':'apps/learning/index.html', '/index.html':'apps/learning/index.html', '/learn':'apps/learning/index.html', '/style.css':'apps/learning/style.css', '/learning.js':'apps/learning/learning.js' };
const contentTypes = { '.html':'text/html; charset=utf-8', '.js':'text/javascript; charset=utf-8', '.css':'text/css; charset=utf-8', '.json':'application/json; charset=utf-8' };
async function body(req) {
  if (!(req.headers['content-type'] || '').toLowerCase().startsWith('application/json')) throw new ApiError(415,'json_required','请求需要使用 JSON');
  const chunks = []; let size = 0;
  for await (const chunk of req) { size += chunk.length; if(size > 5_600_000) throw new ApiError(413,'body_too_large','请求体过大'); chunks.push(chunk); }
  try { const value = JSON.parse(Buffer.concat(chunks).toString('utf8')); if (!value || Array.isArray(value) || typeof value !== 'object') throw new Error(); return value; }
  catch { throw bad('JSON 格式不正确'); }
}
function json(res, status, value) { res.writeHead(status,{'Content-Type':'application/json; charset=utf-8','Cache-Control':'no-store'}); res.end(JSON.stringify(value)); }
export function createServer(options = {}) {
  const store = options.store || new Store(); const provider = options.provider || new SiliconFlowProvider();
  const origins = new Set(options.origins || (process.env.ALLOWED_ORIGINS || '').split(',').filter(Boolean));
  const configuredHosts = new Set((process.env.ALLOWED_HOSTS || '').split(',').filter(Boolean));
  const buckets = new Map(); let requests = 0;
  const limiter = (key, max, windowMs) => {
    const now = Date.now(); const current = buckets.get(key);
    if (!current || current.reset <= now) { buckets.set(key,{n:1,reset:now+windowMs}); return; }
    if (++current.n > max) throw new ApiError(429,'rate_limited','请求过于频繁，请稍后重试');
  };
  const server = http.createServer(async(req,res) => {
    const requestId = randomUUID(); res.setHeader('X-Request-Id',requestId);
    res.setHeader('X-Content-Type-Options','nosniff'); res.setHeader('Referrer-Policy','no-referrer');
    res.setHeader('Content-Security-Policy',"default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self' data:; frame-ancestors 'none'; base-uri 'none'; form-action 'self'");
    let reservation, user, outcome = 200;
    const abort = new AbortController(); res.on('close',() => { if (!res.writableEnded) abort.abort(); });
    try {
      const url = new URL(req.url, 'http://localhost'), path = url.pathname, origin = req.headers.origin;
      const listenPort = server.address()?.port;
      const localHosts = [`127.0.0.1:${listenPort}`, `localhost:${listenPort}`, `[::1]:${listenPort}`];
      if (!localHosts.includes(req.headers.host) && !configuredHosts.has(req.headers.host)) throw new ApiError(403,'host_denied','服务地址未获允许');
      if (++requests % 1000 === 0) for(const [key,bucket] of buckets) if(bucket.reset < Date.now()) buckets.delete(key);
      if(buckets.size > 20000) throw new ApiError(503,'busy','服务繁忙，请稍后重试');
      limiter('ip:' + req.socket.remoteAddress, 200, 60000);
      if (origin) {
        const ownHost = req.headers.host;
        const same = origin === `http://${ownHost}` || origin === `https://${ownHost}`;
        if (!same && !origins.has(origin)) throw new ApiError(403,'origin_denied','此来源未获允许');
        res.setHeader('Access-Control-Allow-Origin',origin);res.setHeader('Vary','Origin');
        res.setHeader('Access-Control-Allow-Headers','Authorization,Content-Type');res.setHeader('Access-Control-Allow-Methods','GET,POST,DELETE,OPTIONS');
      }
      if(req.method === 'OPTIONS') {res.writeHead(204);res.end();return;}
      if(req.method === 'GET' && assets[path]) {
        res.writeHead(200,{'Content-Type':contentTypes[extname(assets[path])],'Cache-Control':'no-cache'});res.end(await readFile(resolve(root,assets[path])));return;
      }
      if(req.method === 'GET' && path === '/health') {json(res,200,{ok:true,service:'modo-api',providerConfigured:provider.ready,version:'0.1.0',demo:options.demo===true});return;}
      if(req.method === 'GET' && path === '/v1/plans') {json(res,200,{price:2000,currency:'CNY',days:30,translations:1000,seconds:7200,paymentAvailable:false});return;}
      if(req.method === 'POST' && ['/v1/auth/register','/v1/auth/login'].includes(path)) {
        limiter('auth:' + req.socket.remoteAddress,10,60000);
        const b=await body(req);json(res,200,path.endsWith('register')?store.register(b):store.login(b));return;
      }
      user = store.authenticate(req.headers.authorization);
      limiter('user:' + user.id,80,60000);
      if(req.method === 'GET' && path === '/v1/me') {json(res,200,{user,usage:store.usage(user)});return;}
      if(req.method === 'POST' && path === '/v1/auth/logout') {store.logout(req.headers.authorization);json(res,200,{ok:true});return;}
      if(req.method === 'DELETE' && path === '/v1/account') {const b=await body(req);store.deleteAccount(user,b.password);json(res,200,{ok:true});return;}
      if(req.method === 'GET' && path === '/v1/sync') {json(res,200,store.sync(user));return;}
      if(req.method === 'POST' && path === '/v1/sync') {json(res,200,store.push(user,await body(req)));return;}
      if(req.method === 'POST' && path === '/v1/translate') {
        const b=await body(req);consent(b.cloudConsent);if(b.privateMode === true)throw new ApiError(403,'private_input','私密输入不允许上传');
        const input=text(b.input,'input',2000),source=language(b.source,'source'),target=language(b.target,'target');
        if(source === target)throw bad('源语言和目标语言不能相同');
        if(options.requireMembership !== false)store.requireMember(user);
        reservation=store.reserve(user,1,0);
        const result=await provider.translate(input,source,target,abort.signal,b.style);json(res,200,result);return;
      }
      if(req.method === 'POST' && path === '/v1/voice') {
        const b=await body(req);consent(b.cloudConsent);if(b.privateMode === true)throw new ApiError(403,'private_input','私密输入不允许上传');
        if(!['native','learning'].includes(b.mode))throw bad('mode 格式不正确');
        const target=language(b.target || 'en');if(b.mode==='learning' && target==='zh')throw bad('学习语言需要为英语或日语');
        const {audio,seconds,duration}=decodeAudio(b.audioBase64);
        if(options.requireMembership !== false)store.requireMember(user);
        reservation=store.reserve(user,0,seconds);
        const transcript=await provider.transcribe(audio,abort.signal);
        const result=b.mode==='native'?{transcript,output:transcript,mode:'native',origin:'ai'}:await provider.coach(transcript,target,abort.signal);
        json(res,200,{...result,mode:b.mode,duration});return;
      }
      throw new ApiError(404,'not_found','接口不存在');
    } catch(error) {
      reservation?.refund();outcome=error instanceof ApiError?error.status:500;
      if(!res.destroyed)json(res,outcome,{error:{code:error instanceof ApiError?error.code:'internal_error',message:error instanceof ApiError?error.message:'服务暂时无法完成请求',requestId}});
    }
    // Request payloads, transcripts, audio, passwords, tokens and API keys are never logged.
    // Optional operational counters can use outcome alone, without user/text dimensions.
  });
  server.requestTimeout = 60000;server.headersTimeout=10000;
  return {server,store,provider};
}
if(process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const host=process.env.HOST || '127.0.0.1', port=Number(process.env.PORT || 8797);
  if(process.env.NODE_ENV === 'production' && !['127.0.0.1','::1'].includes(host)) throw new Error('Production API must listen on loopback behind an HTTPS reverse proxy');
  const {server,store}=createServer();server.listen(port,host,() => console.log(`魔多服务已启动 http://${host.includes(':')?'['+host+']':host}:${port}`));
  for(const signal of ['SIGINT','SIGTERM'])process.on(signal,() => server.close(() => {store.close();process.exit(0);}));
}
