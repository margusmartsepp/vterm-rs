from vterm_python import VTermClient
import time
import json

def test_inspect():
    print("\n--- VTerm Orchestrator Inspection Demo ---")
    client = VTermClient()
    
    # Get initial state
    res = client.execute({"type": "Inspect", "payload": {"assurance": True}})
    
    print(f"Version: {res.get('version')}")
    print(f"Active Terminals: {res.get('active_terminals')}")
    print(f"Pool Size: {res.get('pool_size')}")
    print(f"Max Terminals Limit: {res.get('max_terminals')}")
    print(f"Max Memory Limit: {res.get('max_mem_mb')} MB")
    print(f"Current Memory Usage: {res.get('mem_usage_mb')} MB")
    
    # Spawn a few terminals to see the counters move
    print("\nSpawning 3 terminals...")
    ids = []
    for i in range(3):
        ids.append(client.spawn(f"load-test-{i}", command="echo hello", wait=False))
    
    # Check again
    res = client.execute({"type": "Inspect", "payload": {"assurance": False}})
    print(f"Active Terminals (after spawn): {res.get('active_terminals')}")
    print(f"Pool Size (after spawn): {res.get('pool_size')}")
    
    # Cleanup
    for id in ids:
        client.close(id)
    
    res = client.execute({"type": "Inspect", "payload": {"assurance": False}})
    print(f"Active Terminals (after cleanup): {res.get('active_terminals')}")
    print(f"Pool Size (after cleanup): {res.get('pool_size')}")

if __name__ == "__main__":
    test_inspect()
