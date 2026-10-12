// 服务端与学习页语法检查。
import { spawnSync } from 'node:child_process';
import { readdirSync } from 'node:fs';
for (const directory of ['services/api','apps/learning','scripts','tests']) {
  for (const file of readdirSync(directory).filter(f=>/\.(mjs|js)$/.test(f))) {
    const r=spawnSync(process.execPath,['--check',directory+'/'+file],{stdio:'inherit'});
    if(r.status!==0)process.exit(r.status||1);
  }
}
