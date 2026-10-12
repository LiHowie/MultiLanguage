import { bad } from './errors.mjs';

// Accept only bounded, uncompressed WAV. Duration is derived on the server,
// never taken from a client-supplied duration used for membership billing.
export function validateWav(audio) {
  if (!Buffer.isBuffer(audio) || audio.length < 44 || audio.length > 4 * 1024 * 1024) throw bad('录音大小不正确（最多 30 秒）');
  if (audio.toString('ascii', 0, 4) !== 'RIFF' || audio.toString('ascii', 8, 12) !== 'WAVE' || audio.readUInt32LE(4) + 8 !== audio.length) throw bad('请上传 PCM WAV 录音');
  let format, dataBytes = 0, offset = 12;
  while (offset + 8 <= audio.length) {
    const type = audio.toString('ascii', offset, offset + 4), length = audio.readUInt32LE(offset + 4);
    const start = offset + 8;
    if (start + length > audio.length) throw bad('录音文件不完整');
    if (type === 'fmt ') {
      if (format || length < 16) throw bad('录音格式不正确');
      format = { encoding: audio.readUInt16LE(start), channels: audio.readUInt16LE(start + 2), sampleRate: audio.readUInt32LE(start + 4), byteRate: audio.readUInt32LE(start + 8), align: audio.readUInt16LE(start + 12), bits: audio.readUInt16LE(start + 14) };
    }
    if (type === 'data') dataBytes += length;
    offset = start + length + (length % 2);
  }
  if (!format || !dataBytes || format.encoding !== 1 || format.channels !== 1 || format.bits !== 16 || ![16000, 22050, 24000, 44100, 48000].includes(format.sampleRate) || format.align !== 2 || format.byteRate !== format.sampleRate * 2 || dataBytes % 2) throw bad('仅支持单声道 16 位 PCM WAV');
  const duration = dataBytes / format.byteRate;
  if (duration < 0.2 || duration > 30.1) throw bad('录音需要在 0.2–30 秒之间');
  return { duration, seconds: Math.ceil(duration), mime: 'audio/wav', filename: 'speech.wav' };
}

export function decodeAudio(encoded) {
  if (typeof encoded !== 'string' || encoded.length > 5_600_000 || !/^[A-Za-z0-9+/]+={0,2}$/.test(encoded) || encoded.length % 4) throw bad('audioBase64 格式不正确');
  const audio = Buffer.from(encoded, 'base64');
  if (audio.toString('base64') !== encoded) throw bad('audioBase64 编码不正确');
  return { audio, ...validateWav(audio) };
}
