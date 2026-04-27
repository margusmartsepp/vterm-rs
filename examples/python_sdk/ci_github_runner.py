from fastmcp import FastMCP
import vterm_python
import time

mcp = FastMCP("ci_github_runner")
client = vterm_python.VTermClient()

@mcp.tool()
def trigger_and_monitor_build(repo_url: str) -> str:
    """Simulates cloning a repo and running a CI build, monitoring the terminal until completion."""
    tid = client.spawn("ci_runner", None, 10000, 1000)
    
    # 1. Setup workspace
    client.write(tid, "mkdir -Force C:\\temp_ci; cd C:\\temp_ci\n")
    client.wait_until(tid, r"PS C:\\temp_ci>", 5000)
    
    # 2. Simulate git clone and build
    client.write(tid, f"echo 'Cloning {repo_url}...' ; Start-Sleep -s 2 ; echo 'Building...' ; Start-Sleep -s 3 ; echo 'BUILD SUCCESSFUL'\n")
    
    # 3. Wait specifically for the 'BUILD SUCCESSFUL' output
    try:
        res = client.wait_until(tid, r"BUILD SUCCESSFUL", 15000)
        client.close(tid)
        return f"Build completed successfully! Log snippet:\n{res}"
    except Exception as e:
        client.close(tid)
        return f"Build failed or timed out: {e}"

if __name__ == "__main__":
    mcp.run()
