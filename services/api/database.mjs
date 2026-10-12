import { DatabaseSync } from 'node:sqlite';
import { randomBytes, randomUUID, createHash, scryptSync, timingSafeEqual } from 'node:crypto';
import { mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { ApiError, bad, text } from './errors.mjs';

const hash = (value) => createHash('sha256').update(value).digest('hex');
const passwords = (password, salt) => scryptSync(password, salt, 64).toString('hex');
export class Store {
  constructor(path = process.env.DATABASE_PATH || '.runtime/modo.sqlite', options = {}) {
    if (path !== ':memory:') mkdirSync(dirname(resolve(path)), { recursive: true });
    this.db = new DatabaseSync(path); this.now = options.now || Date.now;
    this.limits = options.limits || {free:{translations:0,seconds:0},pro:{translations:1000,seconds:7200}};
    this.db.exec(`PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA secure_delete=ON;
      CREATE TABLE IF NOT EXISTS users(id TEXT PRIMARY KEY,email TEXT UNIQUE NOT NULL,salt TEXT NOT NULL,password TEXT NOT NULL,plan TEXT NOT NULL DEFAULT 'free',plan_until INTEGER NOT NULL DEFAULT 0,plan_start INTEGER NOT NULL DEFAULT 0);
      CREATE TABLE IF NOT EXISTS sessions(token_hash TEXT PRIMARY KEY,user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,expires INTEGER NOT NULL);
      CREATE TABLE IF NOT EXISTS states(user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,revision INTEGER NOT NULL DEFAULT 0,prefs TEXT NOT NULL DEFAULT '{}');
      CREATE TABLE IF NOT EXISTS cards(user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,id TEXT NOT NULL,payload TEXT NOT NULL,deleted INTEGER NOT NULL DEFAULT 0,PRIMARY KEY(user_id,id));
      CREATE TABLE IF NOT EXISTS usage(user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,day TEXT NOT NULL,translations INTEGER NOT NULL DEFAULT 0,seconds INTEGER NOT NULL DEFAULT 0,PRIMARY KEY(user_id,day));`);
  }
  requireMember(user) { if(this.user(user.id).plan !== 'pro')throw new ApiError(403,'membership_required','整句翻译和语音老师需要 20 元/月会员；基础输入免费'); }
  period(user) { const current=this.user(user.id); if(current.plan==='pro'){const row=this.db.prepare('SELECT plan_start FROM users WHERE id=?').get(user.id);return 'member:'+row.plan_start+':'+Math.max(0,Math.floor((this.now()-row.plan_start)/(30*86400000)));} return new Date(this.now()).toISOString().slice(0,10); }
  transaction(fn) { this.db.exec('BEGIN IMMEDIATE'); try { const r = fn(); this.db.exec('COMMIT'); return r; } catch (e) { this.db.exec('ROLLBACK'); throw e; } }
  credentials(body) {
    const email = text(body.email, 'email', 254).toLowerCase();
    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) throw bad('请输入有效邮箱');
    if (typeof body.password !== 'string' || body.password.length < 10 || body.password.length > 128) throw bad('密码需要 10–128 个字符');
    return { email, password: body.password };
  }
  register(body) {
    const { email, password } = this.credentials(body), id = randomUUID(), salt = randomBytes(16).toString('hex');
    if (this.db.prepare('SELECT 1 FROM users WHERE email=?').get(email)) throw new ApiError(409, 'account_exists', '该邮箱已注册');
    this.transaction(() => { this.db.prepare('INSERT INTO users(id,email,salt,password) VALUES(?,?,?,?)').run(id, email, salt, passwords(password, salt)); this.db.prepare('INSERT INTO states(user_id) VALUES(?)').run(id); });
    return this.session(id);
  }
  login(body) {
    const { email, password } = this.credentials(body), user = this.db.prepare('SELECT * FROM users WHERE email=?').get(email);
    // Do the same password work when the account does not exist.
    const actual = passwords(password, user?.salt || 'nonexistent-account-salt');
    if (!user || !timingSafeEqual(Buffer.from(actual, 'hex'), Buffer.from(user.password, 'hex'))) throw new ApiError(401, 'invalid_login', '邮箱或密码不正确');
    return this.session(user.id);
  }
  session(id) {
    const token = randomBytes(32).toString('base64url'), expires = this.now() + 30 * 86400000;
    this.db.prepare('DELETE FROM sessions WHERE expires<?').run(this.now());
    this.db.prepare('INSERT INTO sessions VALUES(?,?,?)').run(hash(token), id, expires);
    return { token, expires, user: this.user(id) };
  }
  authenticate(header) {
    const token = /^Bearer ([A-Za-z0-9_-]{43})$/.exec(header || '')?.[1];
    const session = token && this.db.prepare('SELECT user_id FROM sessions WHERE token_hash=? AND expires>?').get(hash(token), this.now());
    if (!session) throw new ApiError(401, 'authentication_required', '请登录魔多账号');
    return this.user(session.user_id);
  }
  user(id) {
    const row = this.db.prepare('SELECT id,email,plan,plan_until FROM users WHERE id=?').get(id);
    if (!row) throw new ApiError(401, 'authentication_required', '账号已删除');
    return { id: row.id, email: row.email, plan: row.plan === 'pro' && row.plan_until > this.now() ? 'pro' : 'free', planUntil: row.plan_until };
  }
  logout(header) { const token = header.slice(7); this.db.prepare('DELETE FROM sessions WHERE token_hash=?').run(hash(token)); }
  deleteAccount(user, password) {
    const row = this.db.prepare('SELECT salt,password FROM users WHERE id=?').get(user.id);
    if (typeof password !== 'string' || password.length > 128 || !timingSafeEqual(Buffer.from(passwords(password || '', row.salt), 'hex'), Buffer.from(row.password, 'hex'))) throw new ApiError(401, 'invalid_login', '密码不正确');
    this.db.prepare('DELETE FROM users WHERE id=?').run(user.id);
    this.db.exec('PRAGMA wal_checkpoint(TRUNCATE)');
  }
  usage(user) {
    const day = this.period(user);
    const row = this.db.prepare('SELECT translations,seconds FROM usage WHERE user_id=? AND day=?').get(user.id, day) || { translations: 0, seconds: 0 };
    return { day, ...row, limits: this.limits[user.plan] };
  }
  reserve(user, translations = 0, seconds = 0) {
    const day = this.period(user);
    this.transaction(() => {
      const usage = this.usage(user);
      if (usage.translations + translations > usage.limits.translations || usage.seconds + seconds > usage.limits.seconds) throw new ApiError(429, 'quota_exceeded', '本期额度已用完，请在下一会员周期继续使用');
      this.db.prepare('INSERT INTO usage VALUES(?,?,?,?) ON CONFLICT(user_id,day) DO UPDATE SET translations=translations+excluded.translations,seconds=seconds+excluded.seconds').run(user.id, day, translations, seconds);
    });
    let released = false;
    return { refund: () => { if (released) return; released = true; this.db.prepare('UPDATE usage SET translations=MAX(0,translations-?),seconds=MAX(0,seconds-?) WHERE user_id=? AND day=?').run(translations, seconds, user.id, day); } };
  }
  sync(user) {
    const state = this.db.prepare('SELECT revision,prefs FROM states WHERE user_id=?').get(user.id);
    return { revision: state.revision, preferences: JSON.parse(state.prefs), cards: this.db.prepare('SELECT payload,deleted FROM cards WHERE user_id=?').all(user.id).map(row => ({ ...JSON.parse(row.payload), deleted: !!row.deleted })) };
  }
  push(user, body) {
    if (!Number.isSafeInteger(body.baseRevision) || body.baseRevision < 0 || !Array.isArray(body.cards) || body.cards.length > 100) throw bad('同步请求格式不正确');
    const allowedPrefs = ['targetLanguage', 'inputLanguage', 'layout', 'voiceMode'];
    const prefs = {};
    if (body.preferences !== undefined) {
      if (!body.preferences || Array.isArray(body.preferences) || typeof body.preferences !== 'object' || Object.keys(body.preferences).some(k => !allowedPrefs.includes(k))) throw bad('设置格式不正确');
      for (const [k, v] of Object.entries(body.preferences)) {
        const values = k === 'targetLanguage' ? ['en','ja'] : k === 'inputLanguage' ? ['zh','en','ja'] : k === 'layout' ? ['qwerty','t9'] : ['native','learning'];
        if (!values.includes(v)) throw bad('设置值不正确'); prefs[k] = v;
      }
    }
    const cards = body.cards.map(card => {
      if (!card || typeof card.id !== 'string' || !/^[a-zA-Z0-9_-]{1,80}$/.test(card.id) || typeof card.deleted !== 'boolean') throw bad('学习卡片格式不正确');
      if (card.deleted) return { id: card.id, deleted: true };
      if (!['en','ja'].includes(card.target)) throw bad('卡片语言不正确');
      return { id: card.id, input: text(card.input, 'input', 2000), translation: text(card.translation, 'translation', 4000), reading: text(card.reading, 'reading', 4000, true), note: text(card.note, 'note', 2000, true), target: card.target, deleted: false };
    });
    this.transaction(() => {
      const current = this.sync(user);
      if (current.revision !== body.baseRevision) throw new ApiError(409, 'sync_conflict', '其他设备已更新，请拉取后重试');
      const count = this.db.prepare('SELECT COUNT(*) AS n FROM cards WHERE user_id=?').get(user.id).n;
      const newCount = cards.filter(c => !this.db.prepare('SELECT 1 FROM cards WHERE user_id=? AND id=?').get(user.id,c.id)).length;
      if (count + newCount > 10000) throw new ApiError(413, 'too_many_cards', '卡片上限为 10000 张');
      for (const card of cards) this.db.prepare('INSERT INTO cards VALUES(?,?,?,?) ON CONFLICT(user_id,id) DO UPDATE SET payload=excluded.payload,deleted=excluded.deleted').run(user.id, card.id, JSON.stringify(card), Number(card.deleted));
      this.db.prepare('UPDATE states SET revision=revision+1,prefs=? WHERE user_id=?').run(JSON.stringify({ ...current.preferences, ...prefs }), user.id);
    });
    return this.sync(user);
  }
  setPlan(email, until) {
    if (!Number.isSafeInteger(until) || until <= this.now()) throw new Error('Plan expiry must be a future timestamp');
    const r = this.db.prepare("UPDATE users SET plan='pro',plan_until=?,plan_start=? WHERE email=?").run(until, this.now(), email.toLowerCase());
    if (!r.changes) throw new Error('Account not found');
  }
  close() { this.db.close(); }
}
