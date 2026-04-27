from fastmcp import FastMCP
import vterm_python

mcp = FastMCP("devops_health")
client = vterm_python.VTermClient()

@mcp.tool()
def check_disk_space() -> str:
    """Spawns a terminal, checks system disk space, and returns the result."""
    tid = client.spawn("disk_check", None, 5000, 500)
    
    # Send a cross-platform friendly command or Windows specific (since orchestrator defaults to PowerShell)
    client.write(tid, "Get-PSDrive -PSProvider FileSystem | Select-Object Name, @{Name='Used(GB)';Expression={[math]::Round($_.Used/1GB,2)}}, @{Name='Free(GB)';Expression={[math]::Round($_.Free/1GB,2)}} | Format-Table -AutoSize\n")
    
    # Wait until PowerShell prompt returns
    res = client.wait_until(tid, r"PS [A-Z]:\\.*>", 10000)
    client.close(tid)
    
    return res

@mcp.tool()
def ping_internal_services() -> str:
    """Pings an internal service to verify network connectivity."""
    tid = client.spawn("ping_test", None, 5000, 500)
    
    client.write(tid, "ping -n 4 8.8.8.8\n")
    # Wait until ping finishes and the prompt returns
    res = client.wait_until(tid, r"PS [A-Z]:\\.*>", 15000)
    client.close(tid)
    
    return res

if __name__ == "__main__":
    mcp.run()
