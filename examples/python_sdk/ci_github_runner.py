from fastmcp import FastMCP
import vterm_python
import time

mcp = FastMCP("ci_github_runner")
client = vterm_python.VTermClient()

@mcp.tool()
def trigger_and_monitor_build(repo_url: str) -> str:
    """Simulates cloning a repo and running a CI build, monitoring the terminal until completion."""
    # Atomic Pipeline: Setup -> Action -> Monitor
    ops = [
        client.spawn_op("ci_runner", max_lines=1000),
        client.write_op(1, f"mkdir -Force C:\\temp_ci; cd C:\\temp_ci; echo 'Cloning {repo_url}...'; Start-Sleep -s 2; echo 'Building...'; Start-Sleep -s 3; echo 'BUILD SUCCESSFUL'<Enter>"),
        client.wait_until_op(1, "BUILD SUCCESSFUL", 20000),
        client.read_op(1)
    ]
    
    try:
        res = client.batch(ops)
        screen = res["sub_results"][3]["content"]
        client.close(1)
        return f"Build completed successfully! Log snippet:\n{screen}"
    except Exception as e:
        client.close(1)
        return f"Build failed or timed out: {e}"

if __name__ == "__main__":
    mcp.run()
