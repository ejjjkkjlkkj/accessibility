# Local continuous coding loop

`tools/local_agent_loop.py` is a bounded local Ollama coding-loop prototype shared with the IA repository. It accepts a limited unified diff, rejects protected paths, runs detected Rust manifests, Python tests, and native checks for changed supported-language source files; it blocks commits when the required toolchain/build manifest is missing, and creates a local commit only when tests pass. It never pushes, merges, publishes releases, changes firmware, or runs model-generated shell commands.

Run it only in a dedicated, clean feature-branch checkout on the target PC. Requirements: Python 3.11+, Git, Rust/Cargo for Rust tests, and a running local Ollama model. From PowerShell at the repository root:

```powershell
$env:AGENT_WORKSPACE = (Get-Location).Path
$env:OLLAMA_URL = 'http://127.0.0.1:11434'
$env:OLLAMA_MODEL = 'qwen3-coder:30b'
$env:AGENT_MAX_RUNS = '100'
$env:AGENT_SLEEP_SECONDS = '300'
python .\tools\local_agent_loop.py
```

The loop has a finite budget and sleeps between runs; it is not a guaranteed 24/7 Windows service. The PC must stay awake, Ollama must remain available, and a human must configure Task Scheduler/service restart policy. Review local logs, generated commits, and CI runs. No merge or release is automatic. Contract and runner presence do not prove runtime operation, release readiness, or physical UEFI/audio behavior.

## Register automatic start at logon

After checking out this branch in a dedicated clone, ensure the worktree is clean and the configured local model is installed. In PowerShell from the repository root, run `powershell -NoProfile -ExecutionPolicy Bypass -File .\\tools\\install_agent_task.ps1`. This registers a per-user Scheduled Task that starts the supervisor at logon; start it immediately with `Start-ScheduledTask -TaskName 'LocalCodingAgent-accessibility'`. The supervisor restarts the bounded loop after it exits. This has not been installed or started on the target PC by this GitHub change. Review local logs, generated commits, and CI runs. No merge or release is automatic. Keep the machine awake and signed in; the task does not wake a sleeping PC or prove 24/7 uptime.


## Sequential specialist team

Each iteration rotates through five specialist roles: implementer, test engineer, accessibility reviewer, repository-local security red team, and independent reviewer. They share one model and one clean worktree **sequentially**, not as five conflicting writers. A successful iteration must pass the detected test suite before creating a local commit. Logs identify the role, test exit code, changed paths, and commit. After tests pass, the runner creates a local commit only. Pushes are disabled by default; set `AGENT_AUTO_PUSH=1` only when you explicitly authorize publishing to the dedicated feature branch. It never merges or releases.

## Important execution boundary

This runner is not a security sandbox. Although it never runs commands supplied directly by the model, test/build commands execute repository code, including newly generated code. Use a disposable worktree and a low-privilege Windows account without credentials, personal files, or access to sensitive directories. The path checks and prompt are defense-in-depth, not a guarantee against malicious code. Stop the task if test execution or logs are unexpected.

## Start both repository teams with one command

Use the same script from either repository after both feature-branch checkouts exist on the PC. It validates clean feature branches, Git remotes, Python 3.11+, Ollama, and the requested model before registering and starting both per-user scheduled tasks.

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\install_both_agents.ps1 -IaRepoPath 'C:\PATH\TO\ia' -AccessibilityRepoPath 'C:\PATH\TO\accessibility' -Model 'qwen3-coder:30b'
```

Replace the two example paths with the actual checkout directories. The model must already appear in `ollama list`; the script does not download it. Both agents create test-gated local commits by default and do not push. Add `-EnableAutoPush` only when you explicitly authorize publishing to the existing feature branches; `-DisableAutoPush` always takes precedence. Neither agent merges or publishes a release. After starting, verify both Task Scheduler states and each repository's supervisor/JSONL logs; script success alone is not proof of continuous operation.
