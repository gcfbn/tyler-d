import os
from grpc_tools import protoc

def generate_proto():
    print("Generating gRPC code from protos...")
    protoc.main((
        '',
        '-I../../protos',
        '--python_out=.',
        '--grpc_python_out=.',
        '../../protos/ocr.proto',
    ))

if __name__ == "__main__":
    generate_proto()
