import re

def find_key_lines(filepath):
    with open(filepath, 'r') as f:
        lines = f.readlines()
    for i, line in enumerate(lines, 1):
        if "'desk'" in line or "'mobile'" in line:
            print("  %s:%d: %s" % (filepath, i, line.rstrip()))

print("Key desk/mobile locations:")
find_key_lines('frontend/src/i18n/zh.ts')
find_key_lines('frontend/src/i18n/en.ts')

print("\nUsage of 'desk' and 'mobile' in frontend/src:")
import subprocess
r = subprocess.run(['rg', '-n', 'desk|mobile', 'frontend/src/'], capture_output=True, text=True)
for line in r.stdout.strip().split('\n'):
    if 'desk' in line or 'mobile' in line:
        print("  %s" % line)
