import sys

def generate_docker_compose(num_nodes=20):
    compose = [
        "version: '3.8'",
        "",
        "services:",
        "  # Bootnode (Genesis Node)",
        "  bootnode:",
        "    build: .",
        "    image: thunder-node:latest",
        "    container_name: thunder_bootnode",
        "    ports:",
        "      - \"9000:9000\"  # P2P",
        "      - \"8080:8080\"  # RPC API",
        "    volumes:",
        "      - bootnode_data:/app/node_data",
        "    command: [\"--port\", \"9000\"]",
        "    restart: always"
    ]

    for i in range(2, num_nodes + 1):
        p2p_port = 9000 + (i - 1)
        rpc_port = 8080 + (i - 1)
        node_name = f"validator{i}"
        
        compose.extend([
            "",
            f"  # Validator {i}",
            f"  {node_name}:",
            "    build: .",
            "    image: thunder-node:latest",
            f"    container_name: thunder_{node_name}",
            "    ports:",
            f"      - \"{p2p_port}:9000\"",
            f"      - \"{rpc_port}:8080\"",
            "    volumes:",
            f"      - {node_name}_data:/app/node_data",
            "    depends_on:",
            "      - bootnode",
            f"    command: [\"--port\", \"9000\", \"--bootnode\", \"http://bootnode:9000\"]",
            "    restart: always"
        ])

    compose.extend(["", "volumes:", "  bootnode_data:"])

    for i in range(2, num_nodes + 1):
        compose.append(f"  validator{i}_data:")

    with open("docker-compose.yml", "w") as f:
        f.write("\n".join(compose))
        f.write("\n")

if __name__ == "__main__":
    count = int(sys.argv[1]) if len(sys.argv) > 1 else 20
    generate_docker_compose(count)
    print(f"✅ Successfully generated docker-compose.yml for {count} Nodes!")
