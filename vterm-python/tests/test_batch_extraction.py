from vterm_python import VTermClient
import time
import json

def test_batch_extraction():
    print("\n--- VTerm Parallel Batch Extraction Demo ---")
    client = VTermClient()
    
    # Define a common pattern for pings
    pattern = r"Reply from (?P<ip>[\d\.]+): bytes=(?P<bytes>\d+) time=(?P<time>\d+)ms TTL=(?P<ttl>\d+)"
    
    # We use the low-level execute/batch API to demonstrate raw power
    # Note: Case matters for the 'type' field (Spawn, not spawn)
    commands = [
        {
            "type": "Spawn",
            "payload": {
                "title": "ping-google",
                "command": "ping google.com ; exit",
                "wait": True,
                "extract_pattern": pattern
            }
        },
        {
            "type": "Spawn",
            "payload": {
                "title": "ping-cloudflare",
                "command": "ping 1.1.1.1 ; exit",
                "wait": True,
                "extract_pattern": pattern
            }
        }
    ]
    
    start = time.perf_counter()
    
    # parallel=True means they run at the same time in separate PTYs
    res = client.batch(commands, parallel=True)
    
    end = time.perf_counter()
    print(f"Total Batch Time: {(end - start)*1000:.2f}ms")
    print(f"Batch Status: {res.get('status')}")
    
    # Each sub-result contains its own extracted data
    for i, sub in enumerate(res.get("sub_results", [])):
        extracted = sub.get("extracted", [])
        print(f"\n[Term {sub.get('id')}] - Extracted {len(extracted)} matches")
        if extracted:
            # Show a sample
            print(f"Sample: {json.dumps(extracted[0])}")

if __name__ == "__main__":
    test_batch_extraction()
