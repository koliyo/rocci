import argparse
import hashlib
import os
import shutil
import subprocess
import sys
import tarfile
import time
from pathlib import Path

from rocci_ops.ghutil import DEFAULT_CHECKS, gh_run, wait_for_check
from rocci_ops.paths import repo_root

RELEASE_BINARIES = (
    "rocci",
    "rocdown",
    "rocci-language-server",
)


def version_from_ref(ref_type: str, ref_name: str, sha: str) -> tuple[str, bool]:
    if ref_type == "tag" and ref_name != "dev":
        return ref_name, False
    return f"dev-{sha[:7]}", True


def release_params(ref_type: str, ref_name: str, sha: str) -> tuple[str, str, bool]:
    if ref_type == "tag" and ref_name != "dev":
        return ref_name, ref_name, False
    short = sha[:7]
    return "dev", f"Development Build ({short})", True


def archive_stem(version: str, target: str) -> str:
    return f"rocci-{version}-{target}"


def write_github_output(pairs: dict[str, str], output_path: str | None) -> None:
    lines = "".join(f"{key}={value}\n" for key, value in pairs.items())
    if output_path:
        with open(output_path, "a", encoding="utf-8") as handle:
            handle.write(lines)
        return
    sys.stdout.write(lines)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def package_archive(root: Path, version: str, target: str) -> Path:
    stem = archive_stem(version, target)
    staging = root / "staging" / stem
    staging.mkdir(parents=True, exist_ok=True)
    release_dir = root / "target" / "release"
    for binary in RELEASE_BINARIES:
        src = release_dir / binary
        if not src.is_file():
            raise SystemExit(f"missing release binary: {src}")
        shutil.copy2(src, staging / binary)
    shutil.copy2(root / "README.md", staging / "README.md")
    archive = root / f"{stem}.tar.gz"
    with tarfile.open(archive, "w:gz") as tar:
        tar.add(staging, arcname=stem)
    checksum = archive.with_name(archive.name + ".sha256")
    checksum.write_text(f"{sha256_file(archive)}  {archive.name}\n", encoding="utf-8")
    return archive


def cmd_version(ns: argparse.Namespace) -> int:
    version, prerelease = version_from_ref(
        os.environ.get("GITHUB_REF_TYPE", ""),
        os.environ.get("GITHUB_REF_NAME", ""),
        os.environ.get("GITHUB_SHA", ""),
    )
    write_github_output(
        {"version": version, "prerelease": "true" if prerelease else "false"},
        os.environ.get("GITHUB_OUTPUT"),
    )
    return 0


def cmd_package(ns: argparse.Namespace) -> int:
    archive = package_archive(repo_root(), ns.version, ns.target)
    write_github_output(
        {"archive": archive.name, "version": ns.version},
        os.environ.get("GITHUB_OUTPUT"),
    )
    return 0


PLATFORM_BUNDLE_NAME = "rocci-platform.tar.zst"


def package_platform_bundle(root: Path, dest_name: str = PLATFORM_BUNDLE_NAME) -> Path:
    crate = root / "crates" / "rocci-platform"
    archives = list(crate.glob("*.tar.zst"))
    if not archives:
        raise SystemExit(
            f"no platform *.tar.zst in {crate} (bundle.sh did not emit an archive)"
        )
    src = max(archives, key=lambda path: path.stat().st_mtime)
    dest = crate / dest_name
    if src.resolve() != dest.resolve():
        shutil.copy2(src, dest)
    if dest.stat().st_size == 0:
        raise SystemExit(f"empty platform bundle: {dest}")
    checksum = dest.with_name(dest.name + ".sha256")
    checksum.write_text(f"{sha256_file(dest)}  {dest.name}\n", encoding="utf-8")
    return dest


def cmd_package_platform(ns: argparse.Namespace) -> int:
    archive = package_platform_bundle(repo_root())
    write_github_output(
        {"archive": archive.name, "path": str(archive)},
        os.environ.get("GITHUB_OUTPUT"),
    )
    return 0


REQUIRED_LIBHOST_TRIPLES = frozenset({"arm64mac", "x64musl"})


def merge_libhost_artifacts(
    download_root: Path,
    dest_targets: Path,
    required: frozenset[str] = REQUIRED_LIBHOST_TRIPLES,
) -> list[str]:
    dest_targets.mkdir(parents=True, exist_ok=True)
    triples: set[str] = set()
    for lib in download_root.rglob("libhost.a"):
        triple = lib.parent.name
        if triple in {"targets", "libhosts", ""}:
            raise SystemExit(f"cannot infer libhost triple from {lib}")
        target_dir = dest_targets / triple
        target_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy2(lib, target_dir / "libhost.a")
        triples.add(triple)
    missing = required - triples
    if missing:
        raise SystemExit(
            f"missing libhost triples {sorted(missing)}; found {sorted(triples)}"
        )
    return sorted(triples)


def cmd_merge_libhosts(ns: argparse.Namespace) -> int:
    triples = merge_libhost_artifacts(Path(ns.source), Path(ns.dest))
    print(" ".join(triples), flush=True)
    return 0


def cmd_params(ns: argparse.Namespace) -> int:
    tag, name, prerelease = release_params(
        os.environ.get("GITHUB_REF_TYPE", ""),
        os.environ.get("GITHUB_REF_NAME", ""),
        os.environ.get("GITHUB_SHA", ""),
    )
    write_github_output(
        {
            "tag": tag,
            "name": name,
            "prerelease": "true" if prerelease else "false",
        },
        os.environ.get("GITHUB_OUTPUT"),
    )
    return 0


def cmd_wait_ci(ns: argparse.Namespace) -> int:
    repo = ns.repo or os.environ["GITHUB_REPOSITORY"]
    sha = ns.sha or os.environ["GITHUB_SHA"]

    def gh(args: list[str]) -> str:
        result = gh_run(args)
        return result.stdout

    for check in DEFAULT_CHECKS:
        wait_for_check(repo=repo, sha=sha, check=check, gh=gh, sleep=time.sleep)
    return 0


def collect_release_artifacts(artifact_dir: Path) -> list[Path]:
    artifacts = (
        sorted(artifact_dir.glob("*.tar.gz"))
        + sorted(artifact_dir.glob("*.tar.zst"))
        + sorted(artifact_dir.glob("*.sha256"))
    )
    if not artifacts:
        raise SystemExit(f"no release artifacts in {artifact_dir}")
    if not any(path.name.endswith(".tar.zst") for path in artifacts):
        raise SystemExit(f"missing platform .tar.zst in {artifact_dir}")
    return artifacts


def cmd_publish(ns: argparse.Namespace) -> int:
    artifacts = collect_release_artifacts(Path(ns.artifact_dir))
    if ns.prerelease:
        subprocess.run(
            ["gh", "release", "delete", ns.tag, "--yes", "--cleanup-tag"],
            check=False,
        )
        argv = [
            "gh",
            "release",
            "create",
            ns.tag,
            "--title",
            ns.title,
            "--prerelease",
            "--generate-notes",
            "--target",
            ns.target_sha,
            *[str(path) for path in artifacts],
        ]
    else:
        argv = [
            "gh",
            "release",
            "create",
            ns.tag,
            "--title",
            ns.title,
            "--generate-notes",
            *[str(path) for path in artifacts],
        ]
    subprocess.run(argv, check=True)
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(prog="rocci-ops archive")
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("version")
    pkg = sub.add_parser("package")
    pkg.add_argument("--version", required=True)
    pkg.add_argument("--target", required=True)
    sub.add_parser("package-platform")
    merge_hosts = sub.add_parser("merge-libhosts")
    merge_hosts.add_argument("--from", dest="source", required=True)
    merge_hosts.add_argument("--dest", required=True)
    sub.add_parser("params")
    wait = sub.add_parser("wait-ci")
    wait.add_argument("--repo")
    wait.add_argument("--sha")
    pub = sub.add_parser("publish")
    pub.add_argument("--tag", required=True)
    pub.add_argument("--title", required=True)
    pub.add_argument("--target-sha", required=True)
    pub.add_argument("--artifact-dir", default="artifacts")
    pub.add_argument("--prerelease", action="store_true")

    ns = parser.parse_args(argv)
    if ns.command == "version":
        return cmd_version(ns)
    if ns.command == "package":
        return cmd_package(ns)
    if ns.command == "package-platform":
        return cmd_package_platform(ns)
    if ns.command == "merge-libhosts":
        return cmd_merge_libhosts(ns)
    if ns.command == "params":
        return cmd_params(ns)
    if ns.command == "wait-ci":
        return cmd_wait_ci(ns)
    if ns.command == "publish":
        return cmd_publish(ns)
    raise SystemExit(2)
