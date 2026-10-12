// 原创魔多 M 标记；生成 PNG/ICO，不复用青简名称或图形商标。
import fs from 'node:fs';import path from 'node:path';import zlib from 'node:zlib';
const root=path.resolve(import.meta.dirname,'..'),size=128;
const pixels=Buffer.alloc((size*4+1)*size);
for(let y=0;y<size;y++)for(let x=0;x<size;x++){
    const p=y*(size*4+1)+1+x*4,corner=(x<18&&y<18)||(x>109&&y<18)||(x<18&&y>109)||(x>109&&y>109);
    const letter=(x>=29&&x<=41&&y>=29&&y<=99)||(x>=87&&x<=99&&y>=29&&y<=99)||
        (y>=29&&y<=68&&Math.abs(x-(35+(y-29)*.74))<7)||(y>=29&&y<=68&&Math.abs(x-(93-(y-29)*.74))<7);
    pixels[p]=letter?255:100;pixels[p+1]=letter?255:87;pixels[p+2]=letter?255:202;pixels[p+3]=corner?0:255;
}
function crc(b){let c=0xffffffff;for(const x of b){c^=x;for(let i=0;i<8;i++)c=(c>>>1)^(c&1?0xedb88320:0);}return (c^0xffffffff)>>>0;}
function chunk(type,data){const name=Buffer.from(type),b=Buffer.alloc(data.length+12);b.writeUInt32BE(data.length);name.copy(b,4);data.copy(b,8);b.writeUInt32BE(crc(Buffer.concat([name,data])),b.length-4);return b;}
const header=Buffer.alloc(13);header.writeUInt32BE(size,0);header.writeUInt32BE(size,4);header[8]=8;header[9]=6;
const png=Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]),chunk('IHDR',header),chunk('IDAT',zlib.deflateSync(pixels)),chunk('IEND',Buffer.alloc(0))]);
const ico=Buffer.alloc(22);ico.writeUInt16LE(1,2);ico.writeUInt16LE(1,4);ico[6]=size;ico[7]=size;ico.writeUInt16LE(1,10);ico.writeUInt16LE(32,12);ico.writeUInt32LE(png.length,14);ico.writeUInt32LE(22,18);
fs.writeFileSync(path.join(root,'apps/windows/tsf/resources/modo.ico'),Buffer.concat([ico,png]));fs.writeFileSync(path.join(root,'assets/icon/modo.png'),png);
fs.writeFileSync(path.join(root,'assets/icon/modo.svg'),'<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" viewBox="0 0 128 128"><rect width="128" height="128" rx="22" fill="#6457ca"/><path d="M35 99V35L64 65L93 35V99" fill="none" stroke="white" stroke-width="13" stroke-linejoin="round"/></svg>');
// 上游打包路径继续有效，但所有随包品牌图形均替换为魔多标记。
fs.copyFileSync(path.join(root,'assets/icon/modo.png'),path.join(root,'assets/icon/logo.png'));
fs.copyFileSync(path.join(root,'assets/icon/modo.svg'),path.join(root,'assets/icon/qingjian-mark.svg'));
fs.copyFileSync(path.join(root,'apps/windows/tsf/resources/modo.ico'),path.join(root,'apps/windows/tsf/resources/qingjian.ico'));
fs.writeFileSync(path.join(root,'assets/icon/menu.svg'),'<svg xmlns="http://www.w3.org/2000/svg" width="22" height="16" viewBox="0 0 22 16"><path d="M3 14V2L11 10L19 2V14" fill="none" stroke="black" stroke-width="2.5" stroke-linejoin="round"/></svg>');
const stream='0 G 2.5 w 1 j 3 2 m 3 14 l 11 6 l 19 14 l 19 2 l S\n';
const objects=['<< /Type /Catalog /Pages 2 0 R >>','<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 22 16] /Resources << >> /Contents 4 0 R >>',
    `<< /Length ${Buffer.byteLength(stream)} >>\nstream\n${stream}endstream`];
let pdf='%PDF-1.4\n',offsets=[0];
for(let i=0;i<objects.length;i++){offsets.push(Buffer.byteLength(pdf));pdf+=`${i+1} 0 obj\n${objects[i]}\nendobj\n`;}
const xref=Buffer.byteLength(pdf);pdf+='xref\n0 5\n0000000000 65535 f \n'+offsets.slice(1).map(o=>String(o).padStart(10,'0')+' 00000 n \n').join('');
pdf+=`trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n${xref}\n%%EOF\n`;
fs.writeFileSync(path.join(root,'assets/icon/menu.pdf'),pdf);
