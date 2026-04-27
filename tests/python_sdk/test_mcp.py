import asyncio
import time
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client
import os
import sys
import warnings

# Silence harmless Windows asyncio subprocess teardown warnings
warnings.filterwarnings("ignore", message="unclosed transport", category=ResourceWarning)

async def main():
    start_time = time.time()
    
    script_dir = os.path.dirname(os.path.abspath(__file__))
    server_script = os.path.join(script_dir, "test_fastmcp.py")
    
    server_params = StdioServerParameters(
        command=sys.executable,
        args=[server_script],
    )

    print("Connecting to vterm-mcp...")
    async with stdio_client(server_params) as (read, write):
        async with ClientSession(read, write) as session:
            await session.initialize()
            print("Connected and initialized.")

            print("\nSpawning terminal...")
            spawn_res = await session.call_tool("terminal_spawn", {
                "title": "mcp-test",
                "max_lines": 500
            })
            spawn_text = spawn_res.content[0].text
            print(spawn_text)
            
            term_id = int(spawn_text.strip())

            print(f"\nSending 'ping google.com -t' to Terminal {term_id}...")
            await session.call_tool("terminal_write", {
                "id": term_id,
                "text": "ping google.com -t<Enter>"
            })

            print("Waiting 6 seconds...")
            await asyncio.sleep(6)

            print("\nSending Ctrl+C...")
            await session.call_tool("terminal_write", {
                "id": term_id,
                "text": "<C-c>"
            })
            
            print("Waiting 0.5s for command to stop...")
            await asyncio.sleep(0.5)

            print("\nReading screen after ping:")
            read_res = await session.call_tool("terminal_read", {"id": term_id})
            print("-" * 40)
            print(read_res.content[0].text)
            print("-" * 40)

            print("\nSending 'Invoke-WebRequest https://github.com/margusmartsepp/vterm-rs'...")
            await session.call_tool("terminal_write", {
                "id": term_id,
                "text": "Invoke-WebRequest -Uri https://github.com/margusmartsepp/vterm-rs -UseBasicParsing | Select-Object StatusCode<Enter>"
            })

            print("\nWaiting for Invoke-WebRequest to finish...")
            wait_res = await session.call_tool("terminal_wait_until", {
                "id": term_id,
                "pattern": r"PS [A-Z]:\\.*>",
                "timeout_ms": 30000
            })
            
            print("\nResult of wait_until:")
            print("-" * 40)
            print(wait_res.content[0].text)
            print("-" * 40)

            # Let's read the final screen state to see what actually happened!
            read_res = await session.call_tool("terminal_read", {"id": term_id})
            print("\nFinal Screen Content:")
            print("-" * 40)
            print(read_res.content[0].text)
            print("-" * 40)

            print("\nClosing terminal...")
            await session.call_tool("terminal_close", {"id": term_id})

            print(f"\nTotal execution time: {time.time() - start_time:.2f}s")

    # Give the ProactorEventLoop time to flush and cleanly garbage-collect the subprocess pipes 
    # after the context managers close them, preventing the messy "I/O operation on closed pipe" traceback.
    await asyncio.sleep(0.5)

if __name__ == "__main__":
    asyncio.run(main())
