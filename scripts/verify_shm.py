import mmap
import ctypes
import time
import sys

def main(id):
    name = f"vterm-rs-shm-{id}"
    print(f"Opening shared memory: {name}")
    
    # On Windows, mmap.mmap takes a tagname
    try:
        # We don't know the exact size, but we know it's 4096 from our Rust code
        shm = mmap.mmap(-1, 4096, tagname=name, access=mmap.ACCESS_READ)
    except FileNotFoundError:
        print(f"Error: Shared memory {name} not found. Is the terminal running?")
        return

    print("Connected to SHM. Reading screen (Press Ctrl+C to stop)...")
    try:
        last_content = ""
        while True:
            # Read the first 4096 bytes
            shm.seek(0)
            # The buffer might contain nulls or old data if not fully filled
            # Our Rust code writes the screen contents at offset 0
            # For now, we'll just read everything and strip nulls
            content = shm.read(4096).decode('utf-8', errors='ignore').split('\x00')[0]
            
            if content != last_content:
                print("--- Screen Updated ---")
                print(content)
                last_content = content
            
            time.sleep(0.1)
    except KeyboardInterrupt:
        print("\nExiting...")
    finally:
        shm.close()

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python verify_shm.py <terminal_id>")
    else:
        main(sys.argv[1])
