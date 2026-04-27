import vterm_python
import json

def fluent_fleet_demo():
    client = vterm_python.VTermClient()
    
    print("Executing atomic batch for fleet orchestration...")
    # 1. Spawn the entire fleet in one atomic operation
    # Using the new _op methods for a cleaner, typed syntax
    spawn_ops = [
        client.spawn_op("Terminal A", visible=True),
        client.spawn_op("Terminal B", visible=True),
    ]
    
    spawn_result = client.batch(spawn_ops)
    ids = [r["id"] for r in spawn_result["sub_results"]]
    print(f"Spawned fleet: {ids}")
    
    # 2. Append and Check in a single atomic call
    # No more manual dictionary construction!
    fleet_actions = [
        client.write_op(ids[0], "echo 'I am Alpha'<Enter>"),
        client.write_op(ids[1], "echo 'I am Beta'<Enter>"),
        {"type": "Wait", "payload": {"ms": 1000}}, 
        client.read_op(ids[0]),
        client.read_op(ids[1]),
    ]
    
    print("Executing fleet actions batch...")
    final_result = client.batch(fleet_actions)
    
    # Extract the screen contents (the last two results in the sub_results list)
    screen_a = final_result["sub_results"][3]["content"]
    screen_b = final_result["sub_results"][4]["content"]
    
    print("\n--- FLEET REASONING ---")
    print(f"Terminal A: {screen_a.strip()}")
    print(f"Terminal B: {screen_b.strip()}")
    
    # Clean up
    for tid in ids:
        client.close(tid)

if __name__ == "__main__":
    fluent_fleet_demo()
