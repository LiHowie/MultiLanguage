// 本地验收专用：真实账号/会员/额度接口与固定模型回复，不调用收费服务。
import {Store} from '../services/api/database.mjs';
import {createServer} from '../services/api/server.mjs';
const store=new Store(':memory:');
store.register({email:'member@modo.example',password:'modo-smoke-password'});
store.setPlan('member@modo.example',Date.now()+30*86400000);
const provider={ready:true,translate:async(input,source,target)=>({input,source,target,unit:'sentence',translation:target==='ja'?'外国語を勉強したいです。':'I want to learn a foreign language.',reading:'',note:'',origin:'ai'}),
    transcribe:async()=> 'I go to school yesterday.',coach:async(transcript,target)=>({transcript,target,motherMeaning:'我昨天去了学校。',correctedText:'I went to school yesterday.',explanation:'过去的动作使用过去时。',reading:'',uncertain:false,origin:'ai'})};
const {server}=createServer({store,provider,demo:true});server.listen(8789,'127.0.0.1',()=>console.log('魔多本地验收服务：http://127.0.0.1:8789（固定模型回复）'));
for(const signal of ['SIGINT','SIGTERM'])process.on(signal,()=>server.close(()=>{store.close();process.exit(0);}));
