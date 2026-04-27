from fastmcp import FastMCP
import vterm_python

mcp = FastMCP("devops_health")
client = vterm_python.VTermClient()

@mcp.tool()
def check_disk_space() -> str:
    """Spawns a terminal, checks system disk space, and returns the result."""
    ops = [
        client.spawn_op("disk_check", max_lines=500),
        client.write_op(1, "Get-PSDrive -PSProvider FileSystem | Select-Object Name, Used, Free | Format-Table -AutoSize<Enter>"),
        client.wait_until_op(1, "PS ", 10000),
        client.read_op(1)
    ]
    
    res = client.batch(ops)
    screen = res["sub_results"][3]["content"]
    client.close(1)
    return screen

@mcp.tool()
def ping_internal_services() -> str:
    """Pings an internal service to verify network connectivity."""
    ops = [
        client.spawn_op("ping_test", max_lines=500),
        client.write_op(1, "ping -n 4 8.8.8.8<Enter>"),
        client.wait_until_op(1, "PS ", 15000),
        client.read_op(1)
    ]
    
    res = client.batch(ops)
    screen = res["sub_results"][3]["content"]
    client.close(1)
    return screen

if __name__ == "__main__":
    mcp.run()
