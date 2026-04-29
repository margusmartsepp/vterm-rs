import json
from collections import defaultdict
import sys

def main():
    log_file = "vterm_metrics.jsonl"
    try:
        with open(log_file, "r") as f:
            lines = f.readlines()
    except FileNotFoundError:
        print(f"Error: {log_file} not found.")
        return

    stats = defaultdict(list)
    for line in lines:
        if not line.strip(): continue
        data = json.loads(line)
        stats[data["type"]].append(data["duration_ms"])

    print(f"{'Command Type':<20} | {'Count':<5} | {'Avg (ms)':<10} | {'Max (ms)':<10}")
    print("-" * 55)
    for cmd, durations in sorted(stats.items()):
        avg = sum(durations) / len(durations)
        mx = max(durations)
        print(f"{cmd:<20} | {len(durations):<5} | {avg:<10.2f} | {mx:<10}")

if __name__ == "__main__":
    main()
