# jenkins-atom-tui

parse/visualize `/rssAll` from Jenkins, and get job logs.

## usage

re-uses `~/.config/jenkins_jobs/jenkins_jobs.ini` (from JJB) for auth/connection
info (`Server List [1]`).

```plain
┌Server List [1]───────────────────┐┌Job List [2]─────────────────────────────────────────────────────────┐
│>> my-jenkins-server              ││>> blah #2                                                           │
│                                  ││   blah #1                                                           │
│                                  ││                                                                     │
│                                  ││                                                                     │
│                                  │└─────────────────────────────────────────────────────────────────────┘
│                                  │┌Job Logs [3]─────────────────────────────────────────────────────────┐
│                                  ││                                                                     │
│                                  ││                                                                     │
│                                  ││                                                                     │
│                                  ││                                                                     │
│                                  ││                                                                     │
│                                  ││                                                                     │
│                                  ││                                                                     │
│                                  ││                                                                     │
│                                  ││                                                                     │
│                                  ││                                                                     │
│                                  ││                                                                     │
│                                  ││                                                                     │
│                                  ││                                                                     │
│                                  ││                                                                     │
│                                  ││                                                                     │
│                                  ││                                                                     │
│                                  ││                                                                     │
│                                  ││                                                                     │
│                                  ││                                                                     │
│                                  ││                                                                     │
└──────────────────────────────────┘└─────────────────────────────────────────────────────────────────────┘
┌Status───────────────────────────────────────────────────────────────────────────────────────────────────┐
│Found 1 servers                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

`Job List [2]` lists jobs from the selected Jenkins instance's `/rssAll`. The
jobs are colored green or red depending on if they're successful or not,
respectively. Refer to `jenkins.rs` for what strings map to which `BuildState`.

`Job Logs [3]` is the job output for the selected job. Use `w` to wrap the logs.

The `Status` pane is read-only and not focusable.

Use `r` to refresh the active pane. Select the active pane with `1`, `2`, or `3`
keys. `hjkl` or arrow keys to navigate inside a pane. Refer to `handler.rs` for
the full set of keybinds.
