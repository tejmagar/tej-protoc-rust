import socket
from typing import List

from tej_protoc import protocol
from tej_protoc.client import TPClient
from tej_protoc.file import File
from tej_protoc.server import TPServer
from tej_protoc.callbacks import ResponseCallback


class TestCallback(ResponseCallback):
    def connected(self, client: socket.socket):
        print("Connected")
        self.client.send(protocol.BytesBuilder(0)
                         .add_file('a.png', open('bmc.png', 'rb').read()).set_message(b'').bytes())

    def received(self, files: List[File], message_data: bytes):
        if message_data:
            print(message_data.decode())

        if files:
            file = open('oo.png', 'wb')
            file.write(files[0].data)
            file.close()


def test_server():
    server = TPServer('0.0.0.0', 1234, TestCallback)
    server.listen()


def test_client():
    client = TPClient('127.0.0.1', 1234, TestCallback)
    client.listen()


if __name__ == '__main__':
    test_client()
