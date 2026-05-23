import json
import time
import subprocess
import signal
import os
import socket
import requests
from typing import List, Optional
from threading import Thread
from http.server import HTTPServer, BaseHTTPRequestHandler


class NetworkSimulator:

    def __init__(self, api_base_url="http://localhost:3000"):
        self.api_base_url = api_base_url
        self.nodes = {}
        self.peer_connections = {}
        self.partitions = set()
        self.mock_servers = {}

    def start_node(self, node_id, config_path="", port=None):
        if node_id in self.nodes:
            raise AssertionError(f"Node {node_id} is already running")
        if not config_path:
            config_path = f"/tmp/poi-node-{node_id}.toml"
            self._write_default_config(node_id, config_path, port)
        proc = subprocess.Popen(
            ["poi-node", "--config", config_path],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            preexec_fn=os.setsid,
        )
        self.nodes[node_id] = {
            "process": proc,
            "config_path": config_path,
            "pid": proc.pid,
            "started_at": time.time(),
        }
        time.sleep(2)
        return {"node_id": node_id, "pid": proc.pid, "status": "running"}

    def stop_node(self, node_id):
        if node_id not in self.nodes:
            raise AssertionError(f"Node {node_id} is not running")
        info = self.nodes[node_id]
        try:
            os.killpg(os.getpgid(info["pid"]), signal.SIGTERM)
        except ProcessLookupError:
            pass
        info["process"].wait(timeout=10)
        del self.nodes[node_id]
        return {"node_id": node_id, "status": "stopped"}

    def node_is_running(self, node_id):
        if node_id not in self.nodes:
            return False
        info = self.nodes[node_id]
        return info["process"].poll() is None

    def stop_all_nodes(self):
        results = []
        for node_id in list(self.nodes.keys()):
            results.append(self.stop_node(node_id))
        return results

    def connect_nodes(self, node1_id, node2_id, address=None):
        if not address:
            address = f"/ip4/127.0.0.1/tcp/9091"
        key = frozenset([node1_id, node2_id])
        self.peer_connections[key] = {
            "node1": node1_id,
            "node2": node2_id,
            "address": address,
            "connected_at": time.time(),
            "status": "connected",
        }
        return {"status": "connected", "peers": [node1_id, node2_id]}

    def disconnect_nodes(self, node1_id, node2_id):
        key = frozenset([node1_id, node2_id])
        if key in self.peer_connections:
            self.peer_connections[key]["status"] = "disconnected"
            del self.peer_connections[key]
        return {"status": "disconnected", "peers": [node1_id, node2_id]}

    def get_connected_peers(self, node_id=None):
        connected = []
        for key, conn in self.peer_connections.items():
            if node_id is None or node_id in key:
                connected.append(conn)
        return connected

    def simulate_network_partition(self, node_ids):
        partition_id = f"partition-{int(time.time())}"
        self.partitions.add(partition_id)
        for nid in node_ids:
            if nid in self.nodes:
                info = self.nodes[nid]
                try:
                    os.kill(info["pid"], signal.SIGSTOP)
                except ProcessLookupError:
                    pass
        return {"partition_id": partition_id, "nodes": node_ids, "status": "partitioned"}

    def heal_network_partition(self, partition_id):
        if partition_id not in self.partitions:
            raise AssertionError(f"Partition {partition_id} not found")
        for key, conn in list(self.peer_connections.items()):
            for node_id in [conn["node1"], conn["node2"]]:
                if node_id in self.nodes:
                    info = self.nodes[node_id]
                    try:
                        os.kill(info["pid"], signal.SIGCONT)
                    except ProcessLookupError:
                        pass
        self.partitions.discard(partition_id)
        return {"partition_id": partition_id, "status": "healed"}

    def simulate_network_delay(self, node_id, delay_ms=500):
        return {"node_id": node_id, "delay_ms": delay_ms, "status": "simulated"}

    def simulate_packet_loss(self, node_id, loss_percent=10):
        return {"node_id": node_id, "loss_percent": loss_percent, "status": "simulated"}

    def get_node_peer_count(self, node_id):
        try:
            resp = requests.get(
                f"{self.api_base_url}/api/v1/node/peers",
                timeout=5,
            )
            if resp.status_code == 200:
                data = resp.json()
                return len(data.get("peers", []))
        except (requests.ConnectionError, requests.Timeout):
            pass
        return 0

    def get_node_info(self, node_id):
        node_info = self.nodes.get(node_id, {})
        return {
            "node_id": node_id,
            "running": node_id in self.nodes and node_info.get("process", None) and node_info["process"].poll() is None,
            "pid": node_info.get("pid"),
            "uptime": time.time() - node_info.get("started_at", time.time()) if node_id in self.nodes else 0,
            "peer_count": self.get_node_peer_count(node_id),
        }

    def start_mock_peer(self, peer_id, port):
        class MockHandler(BaseHTTPRequestHandler):
            def do_POST(self):
                content_length = int(self.headers.get("Content-Length", 0))
                body = self.rfile.read(content_length)
                self.send_response(200)
                self.end_headers()
                self.wfile.write(json.dumps({"status": "ok", "peer_id": peer_id}).encode())
            def log_message(self, format, *args):
                pass
        server = HTTPServer(("127.0.0.1", port), MockHandler)
        thread = Thread(target=server.serve_forever, daemon=True)
        thread.start()
        self.mock_servers[peer_id] = {"server": server, "thread": thread, "port": port}
        return {"peer_id": peer_id, "port": port, "status": "listening"}

    def stop_mock_peer(self, peer_id):
        if peer_id in self.mock_servers:
            self.mock_servers[peer_id]["server"].shutdown()
            del self.mock_servers[peer_id]
        return {"peer_id": peer_id, "status": "stopped"}

    def _write_default_config(self, node_id, config_path, port=None):
        config = {
            "node": {
                "id": node_id,
                "data_dir": f"./data/{node_id}",
                "log_level": "debug",
            },
            "network": {
                "listen_addr": f"0.0.0.0:{port or 9090}",
                "external_addr": f"127.0.0.1:{port or 9090}",
                "bootstrap_peers": [],
            },
            "storage": {
                "provider": "duckdb",
                "duckdb_path": f"./data/{node_id}/proofs.duckdb",
            },
            "api": {
                "bind_addr": f"0.0.0.0:{3000 if port is None else port + 1}",
                "allowed_origins": ["*"],
            },
            "ai": {
                "provider": "ollama",
                "model": "gemma3:latest",
                "embedding_model": "nomic-embed-text:latest",
                "endpoint": "http://localhost:11434",
            },
            "vector_store": {
                "provider": "chroma",
                "host": "localhost",
                "port": 8000,
                "collection": f"poi-proofs-{node_id}",
            },
            "analytics": {
                "duckdb_path": f"./data/{node_id}/analytics.duckdb",
            },
            "observability": {
                "log_level": "debug",
            },
        }
        os.makedirs(os.path.dirname(config_path) or ".", exist_ok=True)
        with open(config_path, "w") as f:
            import toml
            toml.dump(config, f)
