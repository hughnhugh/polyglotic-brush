# Source this in Git Bash before cargo, on a Windows PC without Microsoft's
# build tools (docs/decisions/0011):
#
#   export LLVM_MINGW=/c/path/to/llvm-mingw-20261117-ucrt-x86_64
#   . tools/env-windows.sh
#   cargo build --release -p brush-cli
#
# It builds for x86_64-pc-windows-gnullvm (rustup target add
# x86_64-pc-windows-gnullvm), linked by llvm-mingw's clang and lld, with
# llvm-mingw's dlltool for the Windows imports. Output is under
# target/x86_64-pc-windows-gnullvm/.

if [ -z "$LLVM_MINGW" ] || [ ! -x "$LLVM_MINGW/bin/llvm-dlltool.exe" ]; then
  echo "env-windows.sh: set LLVM_MINGW to an unpacked llvm-mingw (github.com/mstorsjo/llvm-mingw, the ucrt-x86_64 zip)" >&2
  return 1 2>/dev/null || exit 1
fi
_llvm="$(cygpath -w "$LLVM_MINGW")\\bin"
export CARGO_BUILD_TARGET=x86_64-pc-windows-gnullvm
export CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_LINKER="$_llvm\\x86_64-w64-mingw32-clang.exe"
export CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_RUSTFLAGS="-Cdlltool=$_llvm\\llvm-dlltool.exe"
unset _llvm
