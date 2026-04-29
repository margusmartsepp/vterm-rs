import os
import json
import time
import subprocess
import psutil
from pathlib import Path
from typing import Dict, Optional
import win32pipe, win32file, pywintypes

# Configuration
SESSION_REGISTRY = Path.home() / ".vterm" / "sessions.json"
PIPE_NAME = r"\\.\pipe\vterm-rs-skill"
VTERM_EXE = Path("target/debug/vterm.exe").absolute()

class Supervisor:
    def __init__(self):
        self.sessions = self._load_sessions()
        self.orchestrator_proc = None
        
    def _load_sessions(self):
        if SESSION_REGISTRY.exists():
            return json.loads(SESSION_REGISTRY.read_text())
        return {}

    def _save_sessions(self):
        SESSION_REGISTRY.parent.mkdir(parents=True, exist_ok=True)
        SESSION_REGISTRY.write_text(json.dumps(self.sessions, indent=2))

    def ensure_orchestrator(self):
        """Self-healing orchestrator check."""
        if self.orchestrator_proc is None or self.orchestrator_proc.poll() is not None:
            print("[Supervisor] Starting vterm orchestrator...")
            self.orchestrator_proc = subprocess.Popen(
                [str(VTERM_EXE), "--headless"],
                creationflags=subprocess.CREATE_NEW_CONSOLE
            )
            # Give it time to bind
            time.sleep(1)
            
    def get_system_telemetry(self):
        """Expose 'it's not us' metrics."""
        cpu = psutil.cpu_percent()
        mem = psutil.virtual_memory().percent
        
        # Monitor all tracked terminals
        terminal_metrics = {}
        for tid, session in list(self.sessions.items()):
            try:
                p = psutil.Process(session["pid"])
                terminal_metrics[tid] = {
                    "cpu": p.cpu_percent(),
                    "mem_mb": p.memory_info().rss / 1024 / 1024,
                    "status": "running"
                }
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                print(f"[Supervisor] Reaping dead session {tid} (PID {session['pid']})")
                del self.sessions[tid]
                self._save_sessions()

        return {
            "system_cpu": cpu,
            "system_mem": mem,
            "terminals": terminal_metrics,
            "assurance_score": 1.0 - (mem / 100.0)
        }

    def register_session(self, tid, pid, title):
        """Deterministic resource tracking."""
        self.sessions[str(tid)] = {
            "pid": pid,
            "title": title,
            "created_at": time.time()
        }
        self._save_sessions()

    def run(self):
        print("[Supervisor] VTerm Supervisor v1.0 active.")
        self.ensure_orchestrator()
        
        while True:
            self.ensure_orchestrator()
            metrics = self.get_system_telemetry()
            # In a real impl, we'd serve these over a control port or SHM
            time.sleep(5)

if __name__ == "__main__":
    sup = Supervisor()
    sup.run()
