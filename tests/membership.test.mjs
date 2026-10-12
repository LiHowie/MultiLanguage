// 验证正式接口不能由客户端自开会员、周期额度和取消后的收费边界。
import test from 'node:test';
import assert from 'node:assert/strict';
import {once} from 'node:events';
import {Store} from '../services/api/database.mjs';
import {createServer} from '../services/api/server.mjs';

test('Free accounts cannot call paid translation, including forged pro claims',async t=>{
    const store=new Store(':memory:');let calls=0;
    const {server}=createServer({store,provider:{ready:true,translate:async()=>{calls++;return {translation:'hello'};}}});
    server.listen(0,'127.0.0.1');await once(server,'listening');
    t.after(()=>{server.closeAllConnections();server.close();store.close();});
    const base='http://127.0.0.1:'+server.address().port;
    const account=store.register({email:'free@example.com',password:'modo-membership-password',plan:'pro'});
    const call=()=>fetch(base+'/v1/translate',{method:'POST',headers:{'Content-Type':'application/json',Authorization:'Bearer '+account.token},body:JSON.stringify({input:'你好',source:'zh',target:'en',cloudConsent:true,plan:'pro'})});
    let r=await call();assert.equal(r.status,403);assert.equal((await r.json()).error.code,'membership_required');assert.equal(calls,0);
    store.setPlan('free@example.com',Date.now()+30*86400000);r=await call();assert.equal(r.status,200);assert.equal(calls,1);
    const plans=await (await fetch(base+'/v1/plans')).json();assert.equal(plans.price,2000);assert.equal(plans.paymentAvailable,false);
});

test('Membership limits span days and expire without deleting learning records',()=>{
    let now=Date.UTC(2026,9,12,4);const store=new Store(':memory:',{now:()=>now,limits:{free:{translations:0,seconds:0},pro:{translations:2,seconds:10}}});
    try {
        const {user}=store.register({email:'period@example.com',password:'modo-period-password'});
        store.setPlan(user.email,now+30*86400000);
        store.reserve(store.user(user.id),1,4);now+=86400000;
        const member=store.user(user.id);assert.equal(store.usage(member).translations,1);assert.equal(store.usage(member).seconds,4);
        store.reserve(member,1,6);assert.throws(()=>store.reserve(member,1,0),e=>e.code==='quota_exceeded');
        store.push(member,{baseRevision:0,cards:[{id:'lesson',input:'hello',translation:'你好',target:'en',deleted:false}]});
        now+=30*86400000;assert.throws(()=>store.requireMember(member),e=>e.code==='membership_required');
        assert.equal(store.sync(store.user(user.id)).cards.length,1);
    } finally {store.close();}
});
