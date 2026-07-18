import pty, os, select, time, re, sys

def run_mimo(prompt, timeout=120):
    master, slave = pty.openpty()
    pid = os.fork()
    if pid == 0:
        os.close(master)
        os.dup2(slave, 0)
        os.dup2(slave, 1)
        os.dup2(slave, 2)
        os.close(slave)
        os.environ['PATH'] = os.path.expanduser('~/.mimocode/bin') + ':' + os.environ.get('PATH', '')
        os.chdir('/app/working/workspaces/default/ops-pilot')
        os.execvp('mimo', ['mimo', 'run', '--dangerously-skip-permissions'] + [prompt])
    else:
        os.close(slave)
        output = b''
        start = time.time()
        while time.time() - start < timeout:
            r, _, _ = select.select([master], [], [], 1)
            if r:
                try:
                    data = os.read(master, 8192)
                    if not data:
                        break
                    output += data
                except:
                    break
        try:
            os.kill(pid, 9)
            os.waitpid(pid, 0)
        except:
            pass
        os.close(master)
        text = output.decode('utf-8', errors='replace')
        text = re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', text)
        text = re.sub(r'\x1b\].*?\x07', '', text)
        return text

if __name__ == '__main__':
    prompt = sys.argv[1] if len(sys.argv) > 1 else "say hello"
    timeout = int(sys.argv[2]) if len(sys.argv) > 2 else 120
    result = run_mimo(prompt, timeout)
    print(result)
