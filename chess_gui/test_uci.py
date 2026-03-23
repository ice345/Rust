import subprocess
import time
import sys

engine = sys.argv[1]

p = subprocess.Popen([engine], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)

def send(cmd):
    print(">", cmd)
    p.stdin.write(cmd + "\n")
    p.stdin.flush()

send("uci")
time.sleep(0.5)
send("isready")
time.sleep(0.5)
send("setoption name MultiPV value 3")
send("position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
send("go depth 10")

while True:
    line = p.stdout.readline()
    if not line:
        break
    print(line.strip())
    if "bestmove" in line:
        break

p.terminate()
