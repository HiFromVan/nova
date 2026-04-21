#!/usr/bin/env bash
# 生成 Demo 客户端需要的 Python gRPC 代码
# 在项目根目录运行：bash demo/gen_proto.sh

pip install grpcio-tools -q

python -m grpc_tools.protoc \
  -I proto \
  --python_out=demo \
  --grpc_python_out=demo \
  proto/nova_control.proto

echo "生成完成 → demo/nova_control_pb2.py + demo/nova_control_pb2_grpc.py"
