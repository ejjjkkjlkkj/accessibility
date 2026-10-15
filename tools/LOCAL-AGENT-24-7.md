# Local continuous coding loop

`tools/local_agent_loop.py` is a bounded local Ollama coding-loop prototype shared with the IA repository. It accepts a limited unified diff, rejects protected paths, runs detected Rust manifests and/or Python tests, and creates a local commit only when tests pass. It never pushes, merges, publishes releases, changes firmware, or runs model-generated shell commands.

Run it only in a dedicated, clean feature-branch checkout on the target PC. Requirements: Python 3.11+, Git, Rust/Cargo for Rust tests, and a running local Ollama model. From PowerShell at the repository root:

```powershell
$env:AGENT_WORKSPACE = (Get-Location).Path
$env:OLLAMA_URL = 'http://127.0.0.1:11434'
$env:OLLAMA_MODEL = 'qwen3-coder:30b'
$env:AGENT_MAX_RUNS = '100'
$env:AGENT_SLEEP_SECONDS = '300'
python .\tools\local_agent_loop.py
```

The loop has a finite budget and sleeps between runs; it is not a guaranteed 24/7 Windows service. The PC must stay awake, Ollama must remain available, and a human must configure Task Scheduler/service restart policy. Review local commits and test logs before pushing. Contract and runner presence do not prove runtime operation, release readiness, or physical UEFI/audio behavior.
