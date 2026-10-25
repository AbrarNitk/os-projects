# Linux Signals: Zero-to-Advanced Learning Plan

## Destination

This plan develops the signal knowledge needed to build three increasingly difficult Linux programs:

1. an interactive shell with pipelines and job control;
2. a reliable process supervisor;
3. a small service manager plus a `systemctl`-like client.

The goal is not merely to memorize signal names. By the end, you should be able to reason about delivery races, signal masks across threads and `fork`/`exec`, terminal-generated signals, child-state transitions, safe shutdown, PID reuse, and event-loop integration.

Use Rust for the main implementations. Write a few tiny C programs where noted, because the Linux/POSIX API and its documentation are expressed in C and understanding that boundary makes Rust wrappers much easier to evaluate.

## Working method

For every topic, use this loop:

1. predict the observable behavior;
2. write the smallest experiment that can disprove the prediction;
3. observe it with `/proc`, `ps`, `strace`, and a second terminal;
4. explain the result in your own words;
5. turn the discovery into an automated test.

Keep a lab journal in `notes/`. For each experiment record the hypothesis, command, result, and explanation. Do not advance when you can only reproduce an example but cannot explain it.

Suggested pace: 12–16 weeks at 10–12 hours per week. The exit criteria matter more than the calendar.

## Phase 0 — Prerequisites and observability (2–3 days)

Learn:

- processes, threads, PIDs, PPIDs, credentials, and the `fork` → `execve` model;
- file descriptors, pipes, blocking syscalls, and terminal basics;
- the difference between libc, a syscall, POSIX behavior, and Linux-specific behavior;
- enough unsafe Rust/FFI to read `libc` and `nix` calls critically.

Tools to practice:

```sh
ps -o pid,ppid,pgid,sid,tpgid,stat,cmd
cat /proc/PID/status
strace -f -e trace=signal,process,wait4 PROGRAM
kill -l
```

Deliverable: a small program that prints its PID, PPID, process group, session, and terminal foreground process group, then waits so those values can be inspected externally.

Exit criterion: explain PID vs process-group ID vs session ID without referring to notes.

## Phase 1 — Signal fundamentals (week 1)

Learn:

- why signals are asynchronous notifications rather than ordinary function calls;
- generation, pending state, delivery, disposition, and handling;
- default actions: terminate, core dump, stop, continue, and ignore;
- `signal(7)`, the conventional signals, and why signal numbers are not portable;
- `kill(2)`, `raise(3)`, `killpg(3)`, and permission checks;
- process-directed vs thread-directed signals;
- `SIGKILL` and `SIGSTOP` cannot be caught, blocked, or ignored;
- standard signals coalesce; they are not a reliable event counter.

Labs:

- reproduce default, ignored, and caught dispositions for `SIGINT`, `SIGTERM`, `SIGHUP`, and `SIGQUIT`;
- send a signal to one PID, process group `0`, and a negative process-group ID;
- block a standard signal, send it repeatedly, inspect `SigPnd`/`ShdPnd` in `/proc/PID/status`, then unblock it;
- distinguish normal termination from signal termination using the shell's exit status and `wait` status.

Deliverable: `signal-lab`, a CLI that installs a selected disposition, displays its mask and pending set, and logs safe observations.

Exit criterion: draw the complete path from signal generation to default action/handler execution.

## Phase 2 — Correct handlers and race-free waiting (week 2)

Learn:

- `sigaction(2)` and why it replaces historical `signal(2)` semantics;
- `sa_mask`, `SA_RESTART`, `SA_RESETHAND`, `SA_NODEFER`, and `SA_SIGINFO`;
- `siginfo_t`: sender PID/UID, queued value, child status, and fault address;
- async-signal-safety and why allocation, formatting, most I/O, mutexes, and Rust runtime operations are forbidden in a handler;
- `errno` preservation and reentrancy;
- interruption of blocking syscalls and the limits of `SA_RESTART`;
- the lost-wakeup race in “check flag, then sleep”;
- `sigprocmask`/`pthread_sigmask`, `sigsuspend`, `sigwaitinfo`, and `sigtimedwait`.

Repository correction: `src/main.rs` currently calls `println!` in `handle_sigint`. Remove that call. A minimal handler should only update an appropriate atomic/`sig_atomic_t`, or write a byte to a pre-created self-pipe. Treat even apparently harmless library calls as unsafe unless `signal-safety(7)` permits them.

Labs:

- demonstrate the lost-wakeup race, then fix it with “block → inspect state → `sigsuspend`”;
- compare a syscall with and without `SA_RESTART`;
- use `SA_SIGINFO` to report the sender outside the handler;
- implement a self-pipe handler and drain it from the normal event loop;
- implement a handler-free version using `sigwaitinfo` in a dedicated signal thread.

Deliverable: two safe shutdown loops—self-pipe and synchronous signal waiting—with stress tests.

Exit criterion: review a handler and identify every operation that is not async-signal-safe.

## Phase 3 — Linux event-loop signal handling (week 3)

Learn:

- `signalfd(2)`: block signals first, then consume structured events through a file descriptor;
- `epoll(7)`, readiness, edge- vs level-triggering, and draining queues;
- tradeoffs among handlers, self-pipe, `sigwaitinfo`, and `signalfd`;
- inheritance of dispositions and masks across `fork`, and reset/preservation rules across `execve`;
- real-time signals: ordering, queuing, payloads, limits, and why application protocols should not overuse them;
- timers via `timerfd` versus signal-generating POSIX timers.

Labs:

- combine stdin, a timerfd, and signalfd in one epoll loop;
- prove that standard signals coalesce while real-time signals queue;
- intentionally inherit a blocked signal across `exec`, observe the bug, and fix the child setup;
- inspect `/proc/PID/status` fields `SigBlk`, `SigIgn`, and `SigCgt`.

Deliverable: an event-loop skeleton reusable by the later supervisor.

Exit criterion: justify which signal-consumption model you would choose for a single-threaded supervisor and for a multithreaded program.

## Phase 4 — Children, zombies, and `SIGCHLD` (weeks 4–5)

Learn:

- zombie and orphan states;
- `wait`, `waitpid`, and `waitid`; `WNOHANG`, `WUNTRACED`, and `WCONTINUED`;
- decoding exited, signaled, core-dumped, stopped, and continued child states;
- why one `SIGCHLD` may represent many child transitions;
- the mandatory “wait until no more state changes” loop;
- `SA_NOCLDSTOP`, `SA_NOCLDWAIT`, ignored `SIGCHLD`, and their consequences;
- fork/reap races: block `SIGCHLD` around fork and bookkeeping;
- Linux subreapers with `prctl(PR_SET_CHILD_SUBREAPER)`;
- `PR_SET_PDEATHSIG` and its race/credential caveats;
- PID reuse and Linux pidfds: `pidfd_open`, `pidfd_send_signal`, polling, and waiting where supported.

Labs:

- create and inspect a zombie, then reap it;
- fork 100 short-lived children and prove every child is reaped;
- track stop/continue/exit transitions using `waitid`;
- demonstrate the “child exits before parent records PID” race and eliminate it;
- adopt and reap a double-forked descendant as a subreaper;
- compare PID-based and pidfd-based signaling.

Deliverable: a race-tested child registry keyed by stable identity where possible, with no zombies under stress.

Exit criterion: explain why a `SIGCHLD` handler must not perform exactly one `waitpid` call.

## Phase 5 — Sessions, process groups, terminals, and job control (weeks 6–7)

This phase is the core of the shell project.

Learn:

- sessions, controlling terminals, foreground/background process groups;
- `setsid`, `setpgid`, `tcgetpgrp`, and `tcsetpgrp`;
- terminal-generated `SIGINT`, `SIGQUIT`, and `SIGTSTP` target the foreground process group;
- `SIGTTIN` and `SIGTTOU` for background terminal access;
- `SIGCONT`, stopped jobs, and orphaned process-group behavior;
- terminal modes with `termios`, saving/restoring them around stopped jobs;
- the parent/child `setpgid` race and why both sides commonly call it;
- shell signal dispositions and resetting them in children before `exec`;
- non-interactive versus interactive shell behavior;
- `SIGHUP` propagation and shell exit policy.

Labs:

- create a pipeline whose processes share one process group;
- transfer terminal foreground ownership to that group and take it back safely;
- stop a foreground pipeline with Ctrl-Z, list it, resume it in background, then foreground it;
- verify Ctrl-C kills the foreground job but not the shell;
- demonstrate background-read suspension through `SIGTTIN`;
- restore terminal settings after a foreground program changes them or stops.

Deliverable: a job-control harness before writing a parser.

Exit criterion: draw all PIDs/PGIDs/SIDs and terminal ownership changes for `cmd1 | cmd2`, Ctrl-Z, `bg`, then `fg`.

## Project 1 — Interactive shell (weeks 8–9)

Scope:

- commands, redirection, pipelines, and background `&` jobs;
- one process group per job;
- built-ins `jobs`, `fg`, `bg`, `kill`, and `exit`;
- correct foreground terminal handoff and restoration;
- asynchronous child-state collection without prompt corruption;
- clean handling of Ctrl-C, Ctrl-Z, stopped jobs, and shell exit;
- reset masks and dispositions in children before `exec`;
- no zombies and no lost child transitions.

Do not begin with a complex parser. Validate process and signal semantics using a tiny argv representation first.

Acceptance tests:

- foreground and background pipelines;
- rapid children that exit immediately;
- Ctrl-C/Ctrl-Z at awkward times, including during pipeline creation;
- repeated `fg`/`bg` cycles;
- commands that read from the terminal in the background;
- shell survives 1,000 short jobs with no zombies;
- PTY-driven integration tests, because ordinary pipes do not reproduce terminal job control.

## Phase 6 — Supervisor semantics (weeks 10–11)

Learn:

- supervision trees and why daemon double-forking conflicts with simple supervision;
- foreground services and `Type=simple`-like contracts;
- startup, running, stopping, failed, and backoff states;
- forwarding signals to a process versus its process group;
- graceful shutdown: `SIGTERM`, deadline, then `SIGKILL` escalation;
- restart policies: never, on failure, always; exit-code and signal classification;
- exponential backoff, jitter, start-rate limiting, and crash-loop prevention;
- readiness and watchdog protocols should use IPC, not unreliable signal counting;
- log pipes, pipe backpressure, descriptor ownership, and avoiding deadlocks;
- configuration reload through `SIGHUP` as an application convention, not a kernel guarantee;
- shutdown ordering and dependency graphs;
- subreaper limitations and why cgroups provide stronger descendant tracking.

Deliverable: a deterministic service state machine whose transition function can be unit-tested without spawning processes.

Exit criterion: specify the result of every signal/exit event in every service state, including events arriving during shutdown.

## Project 2 — Process supervisor (week 12)

Build a Linux supervisor that:

- starts foreground child services in separate process groups;
- uses signalfd + epoll or an equally race-safe architecture;
- reaps all child transitions and optionally acts as a subreaper;
- supports restart policy, bounded backoff, and start limits;
- forwards termination to the whole service process group;
- escalates after a monotonic-clock timeout;
- captures stdout/stderr without blocking supervision;
- reloads validated configuration without losing ownership of existing children;
- exposes status over a Unix-domain control socket;
- prefers pidfds where available and documents its PID-reuse fallback.

Acceptance tests should inject rapid exit, signal death, forked grandchildren, ignored `SIGTERM`, log flooding, partial startup, reload during shutdown, and supervisor restart.

## Phase 7 — Toward a systemd-style service manager (weeks 13–16+)

First clarify the architecture: `systemctl` is a client; the service manager is the long-running daemon. Build both, but keep their responsibilities separate.

Learn:

- PID 1's special signal/default-action and orphan-reaping responsibilities;
- why experimenting as real PID 1 belongs in a disposable PID namespace/container;
- Unix sockets, authentication with peer credentials, request IDs, and idempotent commands;
- declarative units, dependency ordering, requirements versus ordering, and cycle detection;
- service state persistence and daemon/client reconnect behavior;
- socket activation and careful file-descriptor inheritance;
- notification-based readiness over a Unix datagram socket;
- cgroup v2 service membership, lifecycle tracking, resource control, and kill semantics;
- privilege dropping, capabilities, `no_new_privs`, namespaces, and seccomp as later hardening topics;
- boot/shutdown transactions are graph problems, not signal-broadcast problems.

### Project 3A — Service manager daemon

Build:

- a unit loader and dependency DAG;
- start/stop/restart/reload/status operations;
- a supervision state machine per service;
- explicit readiness states;
- process-group support first, then optional cgroup v2 tracking;
- journal-style structured event records;
- transactional ordering with failure propagation;
- a Unix-domain control protocol with versioning.

### Project 3B — `systemctl`-like client

Build a separate CLI that sends requests to the daemon and renders stable human and machine-readable status. The client must never infer service truth merely by checking whether a PID exists.

### Safe PID 1 extension

Only after the ordinary manager is reliable, run it as PID 1 inside `unshare --pid --fork` or a container. Verify orphan adoption, complete reaping, namespace shutdown behavior, and explicit handling of signals whose default behavior differs for PID 1. Do not test an unfinished manager as the host's real init system.

## Advanced kernel-depth track (parallel with phases 4–7)

Read selectively after the user-space behavior is clear:

- Linux signal generation and delivery paths at a conceptual level;
- per-thread and shared pending queues;
- `task_struct`, signal structures, and architecture signal frames;
- return from handlers through `rt_sigreturn` and the vDSO/restorer;
- alternate stacks with `sigaltstack`, `SA_ONSTACK`, stack overflow, and guard pages;
- synchronous fault signals (`SIGSEGV`, `SIGBUS`, `SIGILL`, `SIGFPE`) and why recovery is generally unsafe;
- core dumps, `RLIMIT_CORE`, `/proc/sys/kernel/core_pattern`, and postmortem debugging;
- memory-model questions around handlers, atomics, and lock-free guarantees;
- ptrace/seccomp interactions with signal delivery;
- namespaces, cgroups, PID translation, and signal permission boundaries.

Labs:

- inspect a signal frame and backtrace in GDB;
- use an alternate stack for diagnostic handling of stack exhaustion, documenting its limitations;
- trace signal syscalls and delivery with `strace`; optionally use `perf`/bpftrace when permitted;
- read the relevant Linux kernel source only after predicting the observed behavior.

## Reading order

Use installed Linux man pages as the primary specification for this Linux-only track:

1. `signal(7)`, `signal-safety(7)`, `sigaction(2)`, `sigprocmask(2)`, `sigsuspend(2)`;
2. `kill(2)`, `sigqueue(3)`, `signalfd(2)`, `epoll(7)`;
3. `fork(2)`, `execve(2)`, `waitpid(2)`, `waitid(2)`;
4. `credentials(7)`, `pid_namespaces(7)`, `proc_pid_status(5)`;
5. `setpgid(2)`, `setsid(2)`, `credentials(7)`, `termios(3)`, `tty(4)`;
6. `prctl(2)`, `PR_SET_CHILD_SUBREAPER(2const)`, `PR_SET_PDEATHSIG(2const)`;
7. `pidfd_open(2)`, `pidfd_send_signal(2)`, `cgroups(7)`;
8. systemd's public documentation for unit, service, kill, and control-protocol concepts—study semantics, then design a smaller system deliberately.

Supplement with *The Linux Programming Interface* for process, signal, terminal, and job-control chapters, and read the Rust `std`, `libc`, and `nix` documentation for the exact safety contracts used by the implementation.

## Mastery checklist

You are ready to build these projects when you can answer and demonstrate all of the following:

- What is pending when a blocked standard signal is generated ten times?
- Which thread receives a process-directed signal, and how can the design avoid depending on that choice?
- Which dispositions and masks survive `fork` and `exec`?
- Why is an atomic flag plus ordinary sleep racy, and how does `sigsuspend` fix it?
- Why are `println!`, heap allocation, and mutex locking forbidden in a handler?
- How do `SA_RESTART` and `EINTR` affect each blocking operation you use?
- Why must child reaping drain all available state changes?
- How does a shell prevent Ctrl-C from terminating itself while terminating the foreground pipeline?
- What races exist around `setpgid`, terminal handoff, fork bookkeeping, and immediate child exit?
- Why can a PID refer to the wrong process later, and how do pidfds help?
- Why must a supervisor signal a service group or cgroup rather than only its leader?
- What happens when a service ignores `SIGTERM`?
- Why are readiness, status, and dependency semantics IPC/state-machine problems rather than signal problems?
- What is special about running as PID 1?

## Definition of completion

The track is complete when all three projects pass repeatable stress and PTY integration tests; `strace` and `/proc` observations agree with your written state diagrams; sanitizer/debug runs reveal no unsafe handler behavior; and you can explain each race that the design prevents. A working happy-path demo is not completion—correct behavior under adversarial timing is.
