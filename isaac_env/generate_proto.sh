#!/bin/bash
# 生成 Python gRPC 代码

echo "生成 Python gRPC 代码..."

# 从项目根目录的 proto 文件生成
python -m grpc_tools.protoc \
    -I../proto \
    --python_out=./proto \
    --grpc_python_out=./proto \
    ../proto/simulator.proto

echo "完成！生成的文件："
ls -lh proto/simulator_pb2*.py

echo ""
echo "现在可以运行服务器："
echo "  python server.py"
