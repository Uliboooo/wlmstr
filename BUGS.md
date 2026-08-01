# Bug List (found 2026-07-20, not yet fixed)

Ordered roughly by severity. Clippy is clean — these are all logic/design bugs.

## Crashes / panics

1. **`todo!()` in error display — `src/main.rs:52`**
   `Error::NoWallpaperFound => todo!()`. Any time this error surfaces (empty wallpaper dir),
   `main`'s `eprint!("{}", e)` hits the `todo!()` and the program **panics** instead of
   printing an error message.

2. **`.unwrap()` on a mapped error — `src/status.rs:86`**
   In `get_first_paper_in_dir`, the entry error is mapped with `.map_err(Error::Io)` and then
   immediately `.unwrap()`ed, so an I/O error on the dir entry panics instead of returning
   the error.

## Logic bugs

3. **Video → video leaks mpvpaper processes — `src/status.rs:159-191`**
   `apply()` only kills the existing `mpvpaper` in the *image* branch. When switching from one
   video to another, a new `mpvpaper --fork` is spawned while the old one keeps running —
   they stack up.

4. **`"git"` typo in the video extensions list — `src/status.rs:17`**
   `SUPPURTED_VIDEO_TYPES` contains `"git"` instead of `"gif"`. Also `"webp"` appears **twice**
   in that list *and* in the images list (the image list is checked first, so webp-as-video is
   unreachable dead code). And `"jpegxl"` — the real-world extension is `jxl`.

5. **Exit status of `awww`/`mpvpaper` ignored — `src/status.rs:193-194`**
   `Ok(_)` on `.output()` only means the process *spawned*. If `awww` exits non-zero (daemon
   not running, bad file), `apply()` reports success and the new state is saved even though
   the wallpaper never changed. The `FailedProcess` variant exists exactly for this but is
   never fed the exit status/stderr.

6. **Broken `--mpv-args` argument — `src/status.rs:178`**
   `format!("--mpv-args=\"{}\"", ...)` embeds *literal* quote characters into the argv entry
   (no shell is involved with `Command`), so mpv receives `"--hwdec=auto-safe …"` with quotes
   included. Also `-o` is passed twice with conflicting values (`"no-audio loop"` and
   `"no-audio loop --panscan=1.0"`), and `--mpv-args` isn't mpvpaper's flag name (it's
   `-o`/`--mpv-options`) — combined with bug 5, this fails silently.

7. **`Next` iterates unfiltered directory entries — `src/main.rs:146-150`**
   The file list from `read_dir` includes subdirectories, dotfiles, and unsupported files.
   When rotation lands on one, `apply()` returns `NotSupportFile` *before* the state is
   saved — so the next run computes the exact same "next" entry and fails again: **rotation
   gets permanently stuck** on the first non-wallpaper entry in sort order.

8. **`get_first_paper_in_dir` returns an arbitrary entry — `src/status.rs:77-88`**
   It takes the *first* `read_dir` entry — filesystem order, unsorted, unfiltered.
   `wlmstr set -d <dir>` can pick a subdirectory or a random non-image as the starting
   wallpaper, inconsistent with the sorted order `update()` uses.

9. **Any load error is treated as "file not found" — `src/main.rs:169`**
   `Err(_)` on `Status::load_by_extension` conflates corrupted JSON / permission errors with
   a missing file. Consequences: `status`/`next` report the misleading "value not set, run
   `set`" message on a parse error, and `set` will silently **overwrite** a config it merely
   failed to parse.

10. **`find_mpvpaper` matches too broadly, kills only first match — `src/status.rs:225-232`**
    Substring match on the whole cmdline means `vim mpvpaper.log` or `grep mpvpaper` would
    get SIGTERM'd. And only the first match is killed — multiple mpvpaper instances
    (multi-monitor) leave survivors.

## Smaller issues

11. **`XDG_DATA_HOME` handling — `src/main.rs:131-137`**
    `to_string_lossy().to_string()` mangles non-UTF-8 paths (use `PathBuf::from(os_string)`
    directly). Also, per the XDG spec, an unset `XDG_DATA_HOME` should fall back to
    `~/.local/share` rather than hard-error.

12. **Path-equality mismatch — `src/status.rs:100`**
    `position(|x| **x == self.paper_path)` compares full `PathBuf`s. If the stored path came
    from `set -p ./wallpapers/a.png` while `read_dir` yields `wallpapers/a.png` (or relative
    vs. absolute), it never matches and silently falls into the "file was deleted" recovery
    branch.

13. **Kill/spawn race — `src/status.rs:162-167`**
    SIGTERM is sent to mpvpaper and `awww` is spawned immediately without waiting for the
    process to actually exit and release the wayland layer surface.

14. **`eprint!` without newline — `src/main.rs:201`**
    Error output leaves the shell prompt glued to the message.

15. **Typos throughout** (cosmetic but user-facing)
    `Derection` (Direction), `SUPPURTED`, `"mpbpaper"` in the error message at
    `src/main.rs:47`, `"iamge"` in the `-p` help text, and the garbled duplicated text in the
    `NotFoundXDGDATAPATH` message ("Not dound…").

16. **`test.json` committed at repo root**
    Leftover personal test data (contains real home paths) with an outdated schema
    (has a `mode` field that no longer exists).

## Notes

The nastiest interactions: **7 + 5** (a stuck rotation that also can't tell you the spawn
succeeded but the tool failed) and **3** (process leak on video wallpapers).
