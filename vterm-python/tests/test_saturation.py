from vterm_python import VTermClient
import time

def test_saturation():
    client = VTermClient()
    ids = []
    print("\n--- VTerm System Saturation Test ---")
    print("Limit is set to 10 terminals.")
    
    try:
        # We try to spawn 15 terminals
        for i in range(15):
            print(f"Attempting spawn {i+1}...", end=" ", flush=True)
            id = client.spawn(f"saturation-{i}", command="echo saturate", wait=False)
            ids.append(id)
            print(f"Success (id={id})")
    except Exception as e:
        print(f"\nCaught expected saturation error: {e}")
    
    print(f"Active terminals before cleanup: {len(ids)}")
    
    # Cleanup
    for id in ids:
        client.close(id)
    print("Cleanup finished.")

if __name__ == "__main__":
    test_saturation()
