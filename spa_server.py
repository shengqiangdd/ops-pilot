"""Simple HTTP server with SPA fallback"""
import http.server
import os
import sys

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 5173
DIR = sys.argv[2] if len(sys.argv) > 2 else '.'

class SPAHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIR, **kwargs)
    def send_head(self):
        path = self.translate_path(self.path)
        if os.path.exists(path) and not os.path.isdir(path):
            return super().send_head()
        self.path = '/index.html'
        return super().send_head()

if __name__ == '__main__':
    http.server.HTTPServer(('0.0.0.0', PORT), SPAHandler).serve_forever()
