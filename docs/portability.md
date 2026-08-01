# wlmstr の他言語移植性の比較

現行実装 (Rust, `src/main.rs` 218行 + `src/status.rs` 261行 = 479行) を
Zig / Gleam / Python / Go / その他へ移植した場合の難易度・得失をまとめる。

---

## 0. 前提: このツールは既に Linux 専用である

移植性の議論に入る前に一点。`flake.nix:7-12` は `aarch64-darwin` / `x86_64-darwin`
を systems に含めているが、実装は以下の理由で **Linux (Wayland) 専用**である。

- `procfs` による `/proc` 走査 (`src/status.rs:254-261`) — macOS に `/proc` はない
- `libc::kill` + `SIGTERM` (`src/status.rs:191`) — POSIX なので macOS でも動くが上記とセット
- 外部コマンド `awww` / `mpvpaper` (`src/status.rs:196`, `:208`) — Wayland 専用
- `XDG_DATA_HOME` 必須 (`src/main.rs:132`)

つまりここでいう「移植性」は **OS 間の可搬性ではなく、言語を差し替えたときの実装コスト**
の話に閉じる。darwin ビルドは通るが実行時に必ず失敗するので、flake の systems は
Linux 2つに絞るのが正しい。

---

## 1. 言語に要求される機能の棚卸し

| # | 要求 | 該当箇所 | 移植難度 |
|---|------|----------|----------|
| R1 | サブコマンド付き CLI (derive/宣言的) | `main.rs:61-129` (clap) | 中 |
| R2 | 構造体 ⇄ JSON の永続化 | `status.rs:20,49` (serde + easy_storage) | 低〜中 |
| R3 | ディレクトリ列挙・パス操作・拡張子判定・ソート | `main.rs:144-150`, `status.rs:106-123,233-252` | 低 |
| R4 | 非 UTF-8 パスの扱い (`OsString`) | `to_string_lossy` が随所 | 中 |
| R5 | 環境変数の取得 | `main.rs:132` | 低 |
| R6 | 外部プロセス起動と出力・終了コードの取得 | `status.rs:196-220` | 低 |
| R7 | `/proc` 走査による PID 発見 | `status.rs:254-261` | **中〜高** |
| R8 | `kill(2)` で SIGTERM 送信 | `status.rs:189-195` | **中〜高** |
| R9 | 乱数による選択 | `status.rs:140-142` | 低 |
| R10 | 単一バイナリ配布 + Nix flake パッケージング | `flake.nix:30-40` | 言語依存 |
| R11 | 起動速度 (キーバインド/タイマーから毎回叩かれる) | — | 言語依存 |

**移植の成否を分けるのは R7 / R8 / R10 / R11 の 4 つ**で、R1〜R6, R9 はどの言語でも書ける。
特に R8 (シグナル送信) はランタイムが OS プロセスを抽象化している言語ほど苦しくなる。

なお `easy_storage` は作者自身のクレートなので、どの言語に移しても
「ファイルを読む → JSON パース」を自前で書き直すことになる (実質 10-20 行)。

---

## 2. Zig

**総評: 技術的な摩擦が最も小さい。ただし言語自体の非安定性が最大のリスク。**

| 要求 | 対応 |
|------|------|
| R1 | ✗ 標準に arg parser なし。`std.process.argsAlloc` から手書き (80-120行)、または zig-clap / flags |
| R2 | ○ `std.json` で struct ⇄ JSON。ただし allocator 引き回しが必要 |
| R3 | ○ `std.fs.Dir.iterate()`, `std.fs.path.extension()`, `std.sort.pdq` + 比較関数 |
| R4 | **◎ Rust より素直**。Zig のパスは `[]const u8` (バイト列) で Linux の実態と一致し、`to_string_lossy` が全部消える |
| R5 | ○ `std.process.getEnvVarOwned` |
| R6 | ◎ `std.process.Child.run()` が stdout/stderr/term をまとめて返す。終了コード検査 (BUGS #5) が自然に書ける |
| R7 | △ procfs 相当のライブラリはないが、`/proc/*/cmdline` を読むだけなので 30行程度で自作可能 |
| R8 | **◎ `std.posix.kill(pid, SIG.TERM)`。FFI も `unsafe` ブロックも libc リンクも不要** |
| R9 | ○ `std.Random.DefaultPrng` (seed は `std.crypto.random` から) |
| R10 | ○ 単一静的バイナリ。Nix は `stdenv.mkDerivation` + `zig build`。依存ゼロにすれば `zon2nix` すら不要 |
| R11 | ◎ ~1ms、Rust と同等 |

- 予想行数: **700-900行** (Rust比 +50〜90%)。増分の大半は CLI 手書きと allocator/`errdefer` の定型。
- 得るもの: `unsafe` の消滅、非 UTF-8 パス問題の消滅、ゼロ依存 (現行は 6 クレート + 推移的依存)。
- 失うもの: `serde` の derive、`clap` の derive、`Result<T, E>` の `?` に相当する糖衣は `try` があるが
  エラー型は error set なので `Error::FailedProcess(bool, String, String)` のような
  **値を持つエラーが表現できない** — エラー詳細は out-param か別チャネルで運ぶ設計変更が要る。
  現行の `impl Display for Error` (`main.rs:27-58`) は素直には移らない。
- **最大のリスクは言語のバージョン破壊**。0.13 → 0.14 → 0.15 で `std.io` / `std.json` /
  ビルドシステムに毎回破壊的変更が入っている。「1年放置したらビルドが通らない」が現実に起きる。
  wlmstr のような小さな個人ツールでは、これは Rust の後方互換性を捨てる対価として重い。

---

## 3. Gleam

**総評: 型の表現力は Rust と最もよく噛み合うが、実行モデルがこのアプリと致命的にミスマッチ。**

Gleam は BEAM (Erlang VM) と JS の 2 ターゲットを持つ。どちらを選んでも問題が出る。

| 要求 | 対応 (BEAM ターゲット) |
|------|------|
| R1 | △ `argv` + `glint` / `clip`。derive 相当はなくデコーダ手書き |
| R2 | △ `gleam/json` + `gleam/dynamic/decode`。エンコーダもデコーダも手書き (Status は2フィールドなので実害小) |
| R3 | ○ `simplifile.read_directory`, `filepath` パッケージ |
| R4 | ✗ Gleam の `String` は UTF-8 必須。**非 UTF-8 のファイル名を型として扱えない**。`BitArray` に落とせば可能だが、その瞬間に文字列 API が全部使えなくなる |
| R5 | ○ `envoy` |
| R6 | △ `shellout` (Erlang/JS 両対応)。終了コードは取れる |
| R7 | △ `/proc` 走査自体は `simplifile` で可能だが、`cmdline` は NUL 区切りのバイナリなので `BitArray` + Erlang FFI で分割する必要がある |
| R8 | **✗ 最大の壁。BEAM に `kill(2)` はない**。選択肢は (a) `kill -TERM <pid>` を `shellout` で起動 = プロセス起動が1回増える、(b) C で NIF を書く = Gleam の外に出て Nix ビルドも複雑化、(c) `os:cmd/1`。現実解は (a) の一択 |
| R9 | ○ `prng` パッケージ |
| R10 | **✗ 単一バイナリ配布が事実上不可**。escript は Erlang ランタイム必須、リリース同梱なら数十MB。`burrito` 等は Elixir 寄りで Nix と相性が悪い |
| R11 | **✗ BEAM 起動が数十ms〜100ms 程度**。壁紙切り替えをキーバインドやタイマーから叩く用途では体感に出る |

JS ターゲットに逃げると R10/R11 は多少マシ (Node 起動 ~40ms、`deno compile` で単一バイナリ化は可能) だが、
その場合 `simplifile` / `shellout` の BEAM 実装が使えず **FFI を書き直す**ことになる。

**言語としての相性は良い**: `Error` enum (`main.rs:14-25`) と `Derection` enum は
Gleam のカスタム型にそのまま移り、パターンマッチの網羅性検査も効く。
`Result` も標準。BUGS.md #1 (`todo!()` による panic) のような穴は原理的に減る。

しかし **BEAM は「長時間走る並行システム」のための VM であり、
「毎回起動して即終了する CLI」は最も不得意な形**。この用途では最悪の選択肢。

---

## 4. Python

**総評: 移植コストは最小 (半日)。失うのは型と単一バイナリ配布。**

| 要求 | 対応 |
|------|------|
| R1 | ◎ `argparse` がサブコマンドを標準サポート。`typer` を使えば derive 風の書き味 |
| R2 | ○ `json` + `dataclasses`。型検証は手書きだが 2 フィールド |
| R3 | **◎ 最短**。`pathlib.Path.iterdir()`, `.suffix.lower()`, `sorted(key=...)` |
| R4 | ○ `os.fsencode` / `fsdecode` + surrogateescape で往復可能。ただし surrogate を含む文字列は `json.dump` が例外を投げるので、Rust と同程度の注意は要る |
| R5 | ◎ `os.environ.get` |
| R6 | ◎ `subprocess.run(capture_output=True)` — `returncode` が自然に取れる |
| R7 | ◎ `/proc` を `pathlib` で 5行。psutil すら不要 |
| R8 | **◎ `os.kill(pid, signal.SIGTERM)` — 標準ライブラリ、FFI 不要** |
| R9 | ◎ `random.choice` |
| R10 | △ 単一バイナリではない。ただし **依存ゼロなら Nix パッケージ化は最も簡単** (`mkDerivation` + `makeWrapper`)。代償は closure size (python3 ~50MB vs 静的 Rust バイナリ ~2MB) |
| R11 | ○ 起動 ~20-40ms。`awww` の spawn 自体が数ms〜数十ms かかるので、体感差はほぼ出ない |

- 予想行数: **150-200行** (Rust の 1/3 以下)。標準ライブラリだけで全要求を満たせる唯一の候補。
- 失うもの: `Error` enum の網羅的パターンマッチ。例外に置き換わるので
  BUGS.md #1/#2 のような panic 系は自然に解消する一方、
  **#7 (未フィルタのディレクトリエントリ) や #12 (パス比較のミスマッチ) のような論理バグは
  型では防げない** — テストで担保するしかない。
- NixOS ユーザ向けツールとしては closure size の増加が唯一の実害。
  ただし python3 は大抵のシステムに既にあるので実質ゼロとも言える。

---

## 5. その他の候補

### Go — Rust の代替として総合的に最良

`os/exec`, `syscall.Kill`, `encoding/json`, `os.ReadDir` がすべて標準。
`/proc` 走査も数十行で自作。パスは `string` (バイト列相当) なので R4 も無害。
起動 ~2ms、単一静的バイナリ、Nix は `buildGoModule` で最短。
**失うのは sum type と網羅的パターンマッチ**で、`Error` enum は
`errors.Is/As` + センチネルに崩れる。行数は Rust とほぼ同等 (500-600行)。
CLI は標準 `flag` だとサブコマンド手書き、`cobra` を入れれば clap 相当。

### OCaml — 隠れた良候補

`cmdliner` は clap に最も近い宣言的 CLI、`Unix.kill` / `Unix.readdir` が標準、
`yojson` + `ppx_deriving_yojson` は serde derive に相当、
バリアント型で `Error` enum もそのまま移る。単一バイナリ、起動速い、Nix サポートあり。
Rust から失うものが最も少ない。デメリットは書き手人口とエコシステムの薄さ。

### Nim

C にトランスパイルして単一バイナリ。`std/osproc`, `posix.kill`, `std/json`, `parseopt`
が標準で揃い、記述量は Python 並みで性能は Rust 級。Nix サポートとエコシステムの薄さが難点。

### TypeScript (Deno / Bun)

`Deno.Command`, `Deno.kill`, JSON ネイティブ、`deno compile` で単一バイナリ化可能。
移植は速いが生成バイナリが数十MB になり、Nix closure の観点では Python より不利。

### C

全要求を満たせるが、JSON パーサを外部から持ち込む必要があり、
Rust から得るものが何もない。

---

## 6. 総合比較

◎=優 ○=可 △=要工夫 ✗=困難

| | Rust (現行) | Zig | Gleam | Python | Go | OCaml |
|---|---|---|---|---|---|---|
| CLI (R1) | ◎ clap derive | ✗ 手書き | △ | ◎ argparse | ○ | ◎ cmdliner |
| JSON (R2) | ◎ serde | ○ | △ | ○ | ◎ | ◎ |
| FS/パス (R3) | ◎ | ○ | ○ | ◎ | ◎ | ○ |
| 非UTF-8パス (R4) | △ lossy | ◎ | ✗ | ○ | ◎ | ○ |
| プロセス起動 (R6) | ○ | ◎ | △ | ◎ | ◎ | ○ |
| /proc 走査 (R7) | ◎ procfs | △ 自作 | △ FFI | ◎ | △ 自作 | ○ |
| **シグナル (R8)** | △ unsafe FFI | **◎ 標準** | **✗** | **◎ 標準** | ◎ 標準 | ◎ 標準 |
| 単一バイナリ (R10) | ◎ | ◎ | ✗ | ✗ | ◎ | ◎ |
| 起動速度 (R11) | ◎ ~1ms | ◎ ~1ms | ✗ 数十-100ms | ○ ~30ms | ◎ ~2ms | ◎ |
| 型安全性 | ◎ | ○ | ◎ | ✗ | ○ | ◎ |
| 長期ビルド安定性 | ◎ | **✗** | ○ | ○ | ◎ | ○ |
| 予想行数 | 479 | 700-900 | 400-500 | 150-200 | 500-600 | 350-450 |
| 移植工数 | — | 3-5日 | 4-7日 | 0.5-1日 | 1-2日 | 2-3日 |

---

## 7. 結論

1. **移植しないのが最善。** wlmstr の要求 (単一バイナリ / 高速起動 / シグナル送信 /
   非 UTF-8 パス / Nix パッケージング) に対して Rust は既に妥当な選択で、
   唯一の弱点である `libc::kill` の `unsafe` は 5 行に閉じている (`status.rs:191`)。
   BUGS.md に並ぶ 16 件はいずれも**言語のせいではなく実装の問題**で、
   移植しても自動的には消えない (むしろ #7 や #12 は Python に移すと検出しにくくなる)。

2. **どうしても移すなら Go か Python。**
   - Go: 配布形態と性能を維持したまま `unsafe` を消せる。失うのは sum type。
   - Python: 半日で 1/3 の行数になる。失うのは型と単一バイナリ。
     「自分の NixOS 設定でしか使わない」なら実質デメリットなし。

3. **Zig は技術的には最も気持ちよく収まる** (`std.posix.kill` と バイト列パスにより、
   現行実装で一番汚い 2 箇所が同時に解決する) **が、言語のバージョン破壊が
   個人用ツールには重すぎる**。定期的にメンテする覚悟があるなら好適。

4. **Gleam はこのアプリには選ぶべきでない。** 型システムの相性は良いが、
   BEAM の起動時間と単一バイナリ配布の不可能性、そして何より
   `kill(2)` が使えず外部コマンド起動で代替せざるを得ない点が、
   「毎回起動して即終了し、他プロセスにシグナルを送る CLI」という
   このツールの性質と正面から衝突する。

### 補足: 移植の前にやるべきこと

現行の Rust 実装には移植可能なテストが 1 件も存在しない。
`Status::next()` (`status.rs:119-180`) の回転ロジックと `is_image()` の
拡張子判定はどの言語に移しても同じ振る舞いが要求される中核なので、
**移植を検討する前にこの 2 つのテーブル駆動テストを書いておく**と、
移植先での等価性検証がそのまま流用できる。
