# CI on our own Windows runner (decision 0012): this repository's pinned
# toolchain, installed once into the runner's tool cache, and its paths handed
# to this job alone (GITHUB_PATH, GITHUB_ENV) - nothing machine-wide, nothing
# in the runner account's profile, PATH untouched outside the job.
#
# The pins are the workflow's env, and Rust's own is rust-toolchain.toml:
#   RUSTUP_VERSION, RUSTUP_INIT_SHA256   rustup, x86_64-pc-windows-gnu host
#   LLVM_MINGW_RELEASE, LLVM_MINGW_SHA256 llvm-mingw, the ucrt-x86_64 zip (0011)
#   CARGO_DENY_VERSION, CARGO_DENY_SHA256 cargo-deny, its Windows build
# Every download is checked against its pinned SHA-256 before it is used.
#
# Layout, under <tool cache>\brush: rustup\, cargo\ (RUSTUP_HOME, CARGO_HOME:
# the toolchains, the crates, the advisory database), llvm-mingw-<release>-
# ucrt-x86_64\, cargo-deny-<version>-x86_64-pc-windows-msvc\. The build's
# target folder sits beside the checkout (it survives the checkout's clean, so
# builds are incremental) and is emptied when it passes -TargetGB.
#
# Run from the checkout's root.

param([int]$TargetGB = 20)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

function Need([string]$name) {
    $value = [Environment]::GetEnvironmentVariable($name)
    if (-not $value) { throw "$name isn't set (the workflow's env pins it)" }
    return $value
}

function Run([string]$what) {
    # A native command, its stderr as text; its output shown only when it fails.
    $ErrorActionPreference = 'Continue'
    $out = & $args[0] @($args | Select-Object -Skip 1) 2>&1 | ForEach-Object { "$_" }
    if ($LASTEXITCODE) { $out | Write-Host; throw "$what failed ($LASTEXITCODE)" }
    return $out
}

function Fetch([string]$url, [string]$out, [string]$sha) {
    Run "download $url" curl.exe -sSfL --retry 3 -o $out $url | Out-Null
    $got = (Get-FileHash $out -Algorithm SHA256).Hash.ToLower()
    if ($got -ne $sha.ToLower()) {
        Remove-Item $out
        throw "$url has SHA-256 $got, not the pinned $sha"
    }
}

function Unpack([string]$archive, [string]$folder) {
    # Into a scratch folder first, so a half-unpacked tool is never taken for a whole one.
    $scratch = Join-Path $root '_unpack'
    if (Test-Path $scratch) { Remove-Item -Recurse -Force $scratch }
    New-Item -ItemType Directory $scratch | Out-Null
    Run "unpack $archive" "$env:SystemRoot\System32\tar.exe" -xf $archive -C $scratch | Out-Null
    Move-Item (Join-Path $scratch $folder) (Join-Path $root $folder)
    Remove-Item -Recurse -Force $scratch
    Remove-Item $archive
}

function Append([string]$file, [string]$line) {
    # UTF-8 without a byte-order mark: Windows PowerShell's own UTF-8 writes
    # one, and the runner would read it as part of the first name.
    [IO.File]::AppendAllText($file, "$line`n", (New-Object Text.UTF8Encoding $false))
}

$cache = if ($env:RUNNER_TOOL_CACHE) { $env:RUNNER_TOOL_CACHE } else { Join-Path $env:LOCALAPPDATA 'polyglotic-brush-ci' }
$root = Join-Path $cache 'brush'
New-Item -ItemType Directory -Force $root | Out-Null
$env:RUSTUP_HOME = Join-Path $root 'rustup'
$env:CARGO_HOME = Join-Path $root 'cargo'
$rustup = Join-Path $env:CARGO_HOME 'bin\rustup.exe'

# --- rustup ------------------------------------------------------------------
$want = Need 'RUSTUP_VERSION'
$have = if (Test-Path $rustup) { (Run 'rustup --version' $rustup --version | Where-Object { $_ -like 'rustup *' } | Select-Object -First 1) } else { '' }
if ($have -notlike "rustup $want *") {
    $init = Join-Path $root 'rustup-init.exe'
    Fetch "https://static.rust-lang.org/rustup/archive/$want/x86_64-pc-windows-gnu/rustup-init.exe" $init (Need 'RUSTUP_INIT_SHA256')
    Run 'rustup-init' $init -y --default-host x86_64-pc-windows-gnu --default-toolchain none --no-modify-path --profile minimal | Out-Null
    Remove-Item $init
    $have = Run 'rustup --version' $rustup --version | Where-Object { $_ -like 'rustup *' } | Select-Object -First 1
}
Write-Host $have

# --- Rust, as rust-toolchain.toml pins it, and the gnullvm target (0011) --------
Run 'rustup toolchain install' $rustup toolchain install | Out-Null
Run 'rustup target add' $rustup target add x86_64-pc-windows-gnullvm | Out-Null
Write-Host (Run 'rustc --version' (Join-Path $env:CARGO_HOME 'bin\rustc.exe') --version)

# --- llvm-mingw ------------------------------------------------------------------
$release = Need 'LLVM_MINGW_RELEASE'
$llvmName = "llvm-mingw-$release-ucrt-x86_64"
$llvm = Join-Path $root $llvmName
if (-not (Test-Path (Join-Path $llvm 'bin\llvm-dlltool.exe'))) {
    $zip = Join-Path $root "$llvmName.zip"
    Fetch "https://github.com/mstorsjo/llvm-mingw/releases/download/$release/$llvmName.zip" $zip (Need 'LLVM_MINGW_SHA256')
    Unpack $zip $llvmName
}
Write-Host "llvm-mingw $release"

# --- cargo-deny --------------------------------------------------------------------
$denyVersion = Need 'CARGO_DENY_VERSION'
$denyName = "cargo-deny-$denyVersion-x86_64-pc-windows-msvc"
$deny = Join-Path $root $denyName
if (-not (Test-Path (Join-Path $deny 'cargo-deny.exe'))) {
    $tgz = Join-Path $root "$denyName.tar.gz"
    Fetch "https://github.com/EmbarkStudios/cargo-deny/releases/download/$denyVersion/$denyName.tar.gz" $tgz (Need 'CARGO_DENY_SHA256')
    Unpack $tgz $denyName
}
Write-Host (Run 'cargo-deny --version' (Join-Path $deny 'cargo-deny.exe') --version)

# --- the target folder ---------------------------------------------------------------
$workspace = if ($env:GITHUB_WORKSPACE) { $env:GITHUB_WORKSPACE } else { (Get-Location).Path }
$target = Join-Path (Split-Path $workspace -Parent) 'target'
if (Test-Path $target) {
    $gb = (Get-ChildItem $target -Recurse -File -Force -ErrorAction SilentlyContinue | Measure-Object Length -Sum).Sum / 1GB
    if ($gb -gt $TargetGB) {
        Remove-Item -Recurse -Force $target
        Write-Host ("target was {0:N1} GB, over {1} GB: emptied" -f $gb, $TargetGB)
    } else {
        Write-Host ("target: {0:N1} GB" -f $gb)
    }
}

# --- for this job only -------------------------------------------------------------
if ($env:GITHUB_PATH) {
    Append $env:GITHUB_PATH (Join-Path $env:CARGO_HOME 'bin')
    Append $env:GITHUB_PATH $deny
}
if ($env:GITHUB_ENV) {
    Append $env:GITHUB_ENV "RUSTUP_HOME=$env:RUSTUP_HOME"
    Append $env:GITHUB_ENV "CARGO_HOME=$env:CARGO_HOME"
    Append $env:GITHUB_ENV "CARGO_TARGET_DIR=$target"
    # tools/env-windows.sh takes it in Git Bash's form or Windows' with forward slashes
    Append $env:GITHUB_ENV ("LLVM_MINGW=" + ($llvm -replace '\\', '/'))
}
