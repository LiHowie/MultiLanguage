// A trusted server operator may grant a subscription after verifying payment.
// This command is deliberately absent from the public HTTP API.
import { Store } from '../services/api/database.mjs';
if(process.env.MODO_TEST_MODE !== '1')throw new Error('测试开通未启用：仅在受信任的本机服务端设置 MODO_TEST_MODE=1');
const [email,days]=process.argv.slice(2);
if(!email || !Number.isInteger(Number(days)) || Number(days)<1 || Number(days)>366)throw new Error('Usage: node --env-file-if-exists=.env scripts/plan.mjs email days');
const store=new Store();try{store.setPlan(email,Date.now()+Number(days)*86400000);console.log('订阅有效期已更新');}finally{store.close();}
