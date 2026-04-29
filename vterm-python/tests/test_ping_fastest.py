import time
from contextlib import contextmanager
from vterm_python import VTermClient

@contextmanager
def profile(step: str):
    """Abstracts telemetry out of the business logic."""
    start = time.perf_counter()
    yield
    print(f"[{step}] Elapsed: {(time.perf_counter() - start) * 1000:.2f}ms")

def test_ping_fastest():
    client = VTermClient()
    pattern = r"Reply from (?P<ip>[\d\.]+): bytes=(?P<bytes>\d+) time=(?P<time>\d+)ms TTL=(?P<ttl>\d+)"

    with profile("Total Pipeline Execution"):
        
        with profile("Pool Hijack + Command Injection"):
            term_id = client.spawn("fast-ping", command="ping google.com", wait=False)

        with profile("Bundled Extraction + Callback"):
            # Inline lambda mimics LINQ projection. Defensive .get() prevents FFI boundary crashes.
            filtered_json = client.spawn(
                "fluent-ping", 
                command="ping google.com ; exit", 
                wait=True, 
                extract_pattern=pattern, 
                callback=lambda res: [m for m in res.get("extracted", []) if int(m.get("time", 0)) > 1]
            )

    print(f"\n{filtered_json}\n")
    
    # Resource teardown
    client.close(term_id)

if __name__ == "__main__":
    test_ping_fastest()