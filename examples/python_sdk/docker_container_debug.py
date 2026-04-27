from fastmcp import FastMCP
import vterm_python

mcp = FastMCP("docker_debugger")
client = vterm_python.VTermClient()

@mcp.tool()
def inspect_docker_container(container_name: str) -> str:
    """Spawns a terminal, execs into a running Docker container, and fetches the top processes."""
    ops = [
        client.spawn_op(f"docker_exec_{container_name}", max_lines=500),
        client.write_op(1, f"docker exec -it {container_name} /bin/sh<Enter>"),
        client.wait_until_op(1, "$ ", 5000),
        client.write_op(1, "top -b -n 1<Enter>"),
        client.wait_until_op(1, "$ ", 5000),
        client.read_op(1)
    ]
    
    res = client.batch(ops)
    screen = res["sub_results"][5]["content"]
    
    # Clean exit
    client.batch([client.write_op(1, "exit<Enter>"), client.close_op(1)])
    
    return f"Container Top Output:\n{screen}"

if __name__ == "__main__":
    mcp.run()
