from rocci_ops.paths import repo_root
from rocci_ops.playground import build_playground
from rocci_ops.util import run

BUILD_USAGE = "usage: rocci-ops build [playground | lsp [--release]]"

RELEASE_CRATES = (
    "rocci-cli",
    "rocci-rocdown-cli",
    "rocci-rocdown-lsp",
)


def build_release() -> int:
    root = repo_root()
    argv = ["cargo", "build", "--release"]
    for crate in RELEASE_CRATES:
        argv.extend(["-p", crate])
    run(argv, cwd=root)
    return 0


def build_lsp(*, release: bool = False) -> int:
    argv = ["cargo", "build", "-p", "rocci-rocdown-lsp"]
    if release:
        argv.insert(2, "--release")
    run(argv, cwd=repo_root())
    return 0


def build_command(argv: list[str]) -> int:
    if argv and argv[0] in ("-h", "--help"):
        raise SystemExit(BUILD_USAGE)
    if not argv:
        return build_release()
    if argv == ["playground"]:
        return build_playground()
    if argv and argv[0] == "lsp":
        rest = argv[1:]
        if rest == ["--release"]:
            return build_lsp(release=True)
        if rest:
            raise SystemExit(BUILD_USAGE)
        return build_lsp()
    raise SystemExit(BUILD_USAGE)
