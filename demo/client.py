"""
Nova Demo 客户端
模拟 Becoming 发送指令，验证完整仿真闭环。

用法:
  python demo/client.py                  # 交互模式
  python demo/client.py --auto           # 自动跑预设序列
  python demo/client.py --host 192.168.x.x  # 连远程 Nova
"""

import grpc
import time
import argparse
import sys
import os

sys.path.insert(0, os.path.dirname(__file__))

# 生成 proto 代码：
#   python -m grpc_tools.protoc -I proto --python_out=demo --grpc_python_out=demo proto/nova_control.proto
import nova_control_pb2 as pb
import nova_control_pb2_grpc as pb_grpc


def connect(host: str, port: int = 50052):
    channel = grpc.insecure_channel(f"{host}:{port}")
    stub = pb_grpc.NovaControlStub(channel)
    print(f"[demo] 连接 Nova @ {host}:{port}")
    return stub


def print_state(state, label=""):
    pos = state.position
    print(
        f"[demo] {label} "
        f"pos=({pos.x:.2f},{pos.y:.2f},{pos.z:.2f}) "
        f"stable={state.is_stable} "
        f"joints={list(round(q, 2) for q in state.qpos[:4])}..."
    )


def cmd_move(stub, vx=0.0, vy=0.0, wz=0.0):
    resp = stub.Command(pb.SemanticCommand(
        move=pb.MoveCommand(vx=vx, vy=vy, wz=wz),
        timestamp=time.time(),
    ))
    print_state(resp, f"move(vx={vx})")
    return resp


def cmd_stop(stub):
    resp = stub.Command(pb.SemanticCommand(
        stop=pb.StopCommand(),
        timestamp=time.time(),
    ))
    print_state(resp, "stop")
    return resp


def run_auto(stub):
    """预设测试序列：前进 → 转向 → 停止"""
    print("\n[demo] === 自动测试序列 ===")

    print("\n[demo] 1. 前进 0.5 m/s，持续 5s")
    for _ in range(10):
        cmd_move(stub, vx=0.5)
        time.sleep(0.5)

    print("\n[demo] 2. 左转，持续 3s")
    for _ in range(6):
        cmd_move(stub, vx=0.3, wz=0.5)
        time.sleep(0.5)

    print("\n[demo] 3. 停止")
    cmd_stop(stub)

    print("\n[demo] 4. 订阅状态流（3s）")
    deadline = time.time() + 3
    for state in stub.Subscribe(pb.SubscribeRequest()):
        print_state(state, "stream")
        if time.time() > deadline:
            break

    print("\n[demo] 测试完成 ✓")


def run_interactive(stub):
    """交互模式"""
    print("\n[demo] 交互模式")
    print("  w/s  前进/后退    a/d  左转/右转    空格  停止    q  退出\n")

    import tty, termios, select

    fd = sys.stdin.fileno()
    old = termios.tcgetattr(fd)
    try:
        tty.setraw(fd)
        while True:
            if select.select([sys.stdin], [], [], 0.1)[0]:
                ch = sys.stdin.read(1)
                if ch == 'q':
                    break
                elif ch == 'w':
                    cmd_move(stub, vx=0.5)
                elif ch == 's':
                    cmd_move(stub, vx=-0.3)
                elif ch == 'a':
                    cmd_move(stub, wz=0.5)
                elif ch == 'd':
                    cmd_move(stub, wz=-0.5)
                elif ch == ' ':
                    cmd_stop(stub)
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old)
        cmd_stop(stub)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=50052)
    parser.add_argument("--auto", action="store_true", help="运行自动测试序列")
    args = parser.parse_args()

    stub = connect(args.host, args.port)

    if args.auto:
        run_auto(stub)
    else:
        run_interactive(stub)
