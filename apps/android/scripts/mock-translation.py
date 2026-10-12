"""仅用于模拟器验证的本机翻译服务；只接受测试密钥，不调用外部 AI。"""
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        if self.headers.get("Authorization") != "Bearer qingjian-test-key-not-a-real-secret":
            self.send_error(401)
            return
        body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        messages = body.get("messages", [])
        try:
            request = json.loads(messages[-1]["content"])
        except (ValueError, KeyError):
            request = {}
        language = request.get("target_language", "en")
        formal = "书面" in messages[0]["content"]
        text = {
            "en": "I would like to study a foreign language." if formal else "I'd like to learn a foreign language.",
            "ja": "外国語を学びたいです。",
            "es": "Me gustaría aprender un idioma extranjero.",
        }[language]
        content = json.dumps({"translation": text} if "text" in request else {"ok": True}, ensure_ascii=False)
        reply = json.dumps({
            "id": "qingjian-local-test", "object": "chat.completion", "created": 1,
            "model": "test-translator", "choices": [{"index": 0, "message": {"role": "assistant", "content": content}, "finish_reason": "stop"}],
        }, ensure_ascii=False).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(reply)))
        self.end_headers()
        self.wfile.write(reply)
        print(json.dumps({"language": language, "formal": formal, "source_only": set(request) <= {"text", "target_language"}}, ensure_ascii=False), flush=True)

    def log_message(self, *_args):
        pass


if __name__ == "__main__":
    print("Qingjian test service on 127.0.0.1:18766", flush=True)
    ThreadingHTTPServer(("127.0.0.1", 18766), Handler).serve_forever()
