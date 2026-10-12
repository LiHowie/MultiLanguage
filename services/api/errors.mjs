export class ApiError extends Error {
  constructor(status, code, message) { super(message); this.status = status; this.code = code; }
}
export const bad = (message) => new ApiError(400, 'invalid_request', message);
export function text(value, name, max = 2000, optional = false) {
  if (optional && (value === undefined || value === '')) return '';
  if (typeof value !== 'string' || !value.trim() || value.length > max || /\u0000/.test(value)) throw bad(`${name} 格式不正确`);
  return value.normalize('NFKC').trim();
}
export function language(value, name = 'language') {
  if (!['zh', 'en', 'ja'].includes(value)) throw bad(`${name} 仅支持 zh、en、ja`);
  return value;
}
export function consent(value) {
  if (value !== true) throw new ApiError(403, 'consent_required', '请先同意将本次输入发送到硅基流动进行处理');
}
