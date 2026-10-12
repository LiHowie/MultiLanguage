// 本机测试开通，凭证只写入被忽略的运行目录，不进入仓库或客户端。
import {Store} from '../services/api/database.mjs';
import {randomBytes} from 'node:crypto';
import {mkdirSync,writeFileSync} from 'node:fs';
if(process.env.MODO_TEST_MODE!=='1')throw new Error('请先在本机服务端开启 MODO_TEST_MODE=1');
const email='learner@modo.local',store=new Store();
try {
    if(store.db.prepare('SELECT 1 FROM users WHERE email=?').get(email)){console.log('本机魔多测试账号已存在');process.exitCode=0;}
    else {
        const password=randomBytes(18).toString('base64url');store.register({email,password});store.setPlan(email,Date.now()+30*86400000);
        mkdirSync('.runtime',{recursive:true});writeFileSync('.runtime/test-account.txt',`魔多本机测试账号\n邮箱：${email}\n密码：${password}\n服务地址：http://127.0.0.1:8797/v1\n会员：30天测试会员\n仅用于本机开发验证，不能用于公开部署。\n`,{mode:0o600});
        console.log('测试会员已准备，凭证见 .runtime/test-account.txt');
    }
} finally {store.close();}
