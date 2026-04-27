from fastmcp import FastMCP
import vterm_python

mcp = FastMCP("docker_debugger")
client = vterm_python.VTermClient()

@mcp.tool()
def inspect_docker_container(container_name: str) -> str:
    """Spawns a terminal, execs into a running Docker container, and fetches the top processes."""
    tid = client.spawn(f"docker_exec_{container_name}", None, 5000, 500)
    
    # Exec into the container
    client.write(tid, f"docker exec -it {container_name} /bin/sh\n")
    
    # Wait for the container shell prompt (usually '#' or '$')
    client.wait_until(tid, r"(\#|\$) $", 5000)
    
    # Run top
    client.write(tid, "top -b -n 1\n")
    
    # Wait for prompt again
    res = client.wait_until(tid, r"(\#|\$) $", 5000)
    
    # Exit container
    client.write(tid, "exit\n")
    client.close(tid)
    
    return f"Container Top Output:\n{res}"

if __name__ == "__main__":
    mcp.run()
