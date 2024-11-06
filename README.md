# hydra-check

![Test](https://github.com/nix-community/hydra-check/workflows/Test/badge.svg)

Check hydra for the build status of a package in a given channel.

# Disclaimer
Keep in mind that hydra is the NixOS build-farm orchestrator and has more important tasks to do than answering your puny requests. Response time may be in the seconds for each request.

# Usage

```console
$ nix-shell

$ hydra-check --help
usage: hydra-check [options] PACKAGES...
...

$ hydra-check
Evaluations of jobset nixpkgs/trunk @ https://hydra.nixos.org/jobset/nixpkgs/trunk/evals
⧖ nixpkgs → df76cd6  5h ago      ✔ 194306  ✖ 3554   ⧖ 55364  Δ ?       https://hydra.nixos.org/eval/1809865
⧖ nixpkgs → 7701a9e  14h ago     ✔ 196874  ✖ 5123   ⧖ 51253  Δ ?       https://hydra.nixos.org/eval/1809859
⧖ nixpkgs → 7150b43  23h ago     ✔ 200232  ✖ 48388  ⧖ 4629   Δ ?       https://hydra.nixos.org/eval/1809854
⧖ nixpkgs → ed1b999  1d ago      ✔ 200402  ✖ 48482  ⧖ 4319   Δ ?       https://hydra.nixos.org/eval/1809850
⧖ nixpkgs → b651050  1d ago      ✔ 244082  ✖ 6187   ⧖ 2813   Δ ?       https://hydra.nixos.org/eval/1809840
✔ nixpkgs → 85f7e66  2d ago      ✔ 246713  ✖ 5963   ⧖ 0      Δ +67     https://hydra.nixos.org/eval/1809829
✔ nixpkgs → fe348e1  2d ago      ✔ 246646  ✖ 6015   ⧖ 0      Δ +276    https://hydra.nixos.org/eval/1809822
...

$ hydra-check hello
Build Status for nixpkgs.hello.x86_64-linux on jobset nixos/trunk-combined
✔ hello-2.10 from 2020-03-14 - https://hydra.nixos.org/build/114752982

$ hydra-check hello --arch x86_64-darwin
Build Status for hello.x86_64-darwin on jobset nixpkgs/trunk
✔ hello-2.12.1 from 2023-09-28 - https://hydra.nixos.org/build/236635446

$ hydra-check hello python --channel 19.03
Build Status for nixpkgs.hello.x86_64-linux on jobset nixos/19.03
✔ hello-2.10 from 2019-10-14 - https://hydra.nixos.org/build/103243113
Build Status for nixpkgs.python.x86_64-linux on jobset nixos/19.03
✔ python-2.7.17 from 2020-01-14 - https://hydra.nixos.org/build/110523905

$ hydra-check nixos.tests.installer.simpleUefiGrub --channel 19.09 --arch aarch64-linux
Build Status for nixos.tests.installer.simpleUefiGrub.aarch64-linux on jobset nixos/19.09
✖ (Dependency failed) vm-test-run-installer-simpleUefiGrub from 2020-03-19 - https://hydra.nixos.org/build/115139363

Last Builds:
✖ (Dependency failed) vm-test-run-installer-simpleUefiGrub from 2020-03-18 - https://hydra.nixos.org/build/115135183
✖ (Dependency failed) vm-test-run-installer-simpleUefiGrub from 2020-03-18 - https://hydra.nixos.org/build/115093440
✔ vm-test-run-installer-simpleUefiGrub from 2020-03-18 - https://hydra.nixos.org/build/115073926
✔ vm-test-run-installer-simpleUefiGrub from 2020-03-17 - https://hydra.nixos.org/build/115013869
✖ (Cancelled) vm-test-run-installer-simpleUefiGrub from 2020-03-17 - https://hydra.nixos.org/build/114921818
✔ vm-test-run-installer-simpleUefiGrub from 2020-03-17 - https://hydra.nixos.org/build/114887664
✖ (Timed out) vm-test-run-installer-simpleUefiGrub from 2020-03-16 - https://hydra.nixos.org/build/114881668
...


$ hydra-check ugarit --channel 19.09 --short
Build Status for nixpkgs.ugarit.x86_64-linux on jobset nixos/19.09
✖ (Dependency failed) chicken-ugarit-2.0 from 2020-02-23 - https://hydra.nixos.org/build/108216732


$ hydra-check nixos.containerTarball hello --channel 19.09 --arch i686-linux --json | jq .
{
  "nixos.containerTarball": [
    {
      "icon": "✖",
      "success": false,
      "status": "Failed",
      "timestamp": "2020-03-18T22:02:59Z",
      "build_id": "115099119",
      "build_url": "https://hydra.nixos.org/build/115099119",
      "name": "tarball",
      "arch": "i686-linux"
    },
    {
      "icon": "✖",
      "success": false,
      "status": "Failed",
      "timestamp": "2020-03-17T18:10:09Z",
      "build_id": "115073178",
      "build_url": "https://hydra.nixos.org/build/115073178",
      "name": "tarball",
      "arch": "i686-linux"
    },
    ...
  ],
    "hello": [
    {
      "icon": "✔",
      "success": true,
      "status": "Succeeded",
      "timestamp": "2017-07-31T13:28:03Z",
      "build_id": "57619684",
      "build_url": "https://hydra.nixos.org/build/57619684",
      "name": "hello-2.10",
      "arch": "i686-linux"
    },
    {
      "icon": "✔",
      "success": true,
      "status": "Succeeded",
      "timestamp": "2017-07-25T03:36:27Z",
      "build_id": "56997384",
      "build_url": "https://hydra.nixos.org/build/56997384",
      "name": "hello-2.10",
      "arch": "i686-linux"
    },
    ...
  ]
}

$ hydra-check --channel=staging-next --eval
info: querying the latest evaluation of --jobset 'nixpkgs/staging-next'

Evaluations of jobset nixpkgs/staging-next @ https://hydra.nixos.org/jobset/nixpkgs/staging-next/evals
⧖ nixpkgs → b2a0e31  1d ago  ✔ 42294  ✖ 2447  ⧖ 208458  Δ +18066  https://hydra.nixos.org/eval/1809845

info: no package filter has been specified, so the default filter '/nixVersions.stable' is used for better performance
info: specify another filter with --eval '1809845/<filter>', or force an empty filter with a trailing slash: '1809845/'

Evaluation 1809845 filtered by 'nixVersions.stable' @ https://hydra.nixos.org/eval/1809845?filter=nixVersions.stable

input: nixpkgs
type: Git checkout
value: https://github.com/NixOS/nixpkgs.git
revision: b2a0e3125e8b373ee2d6480ebd3b8f5c20080796
store_path: /nix/store/54adh6vxi8zf1vpxj2gagwajk3hcrd0x-source

input: officialRelease
type: Boolean
value: false

input: supportedSystems
type: Nix expression
value: [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ]

changed_input: nixpkgs
changes: 20741966793c to b2a0e3125e8b
url: https://hydra.nixos.org/api/scmdiff?type=git&rev2=b2a0e3125e8b373ee2d6480ebd3b8f5c20080796&branch=&rev1=20741966793c81f2322645d30bce95d37f63f545&uri=https%3A%2F%2Fgithub.com%2FNixOS%2Fnixpkgs.git
revs: 20741966793c81f2322645d30bce95d37f63f545 -> b2a0e3125e8b373ee2d6480ebd3b8f5c20080796

Queued Jobs:
⧖ (Queued)  nixVersions.stable.aarch64-darwin  nix-2.24.10              https://hydra.nixos.org/build/277949183
⧖ (Queued)  nixVersions.stable.aarch64-linux   nix-2.24.10  2024-11-07  https://hydra.nixos.org/build/277629888
⧖ (Queued)  nixVersions.stable.x86_64-darwin   nix-2.24.10              https://hydra.nixos.org/build/277948312
⧖ (Queued)  nixVersions.stable.x86_64-linux    nix-2.24.10  2024-11-07  https://hydra.nixos.org/build/277560645

```

# Changelog

## 2.0.0 Breaking changes
- Rewritten in Rust
- Always prints long outputs with all recent builds unless `--short` is explicitly specified
- `--arch` defaults to the target architecture (instead of `x86_64-linux` all the time)
- `--jobset` explicitly conflicts with `--channel` to avoid confusion, as channels are just aliases for jobsets
- The `staging` channel / alias is removed as `nixos/staging` is no longer active; instead we add `staging-next` as an alias for `nixpkgs/staging-next`
- The default `unstable` channel points to `nixpkgs/trunk` on non-NixOS systems

### Features
- Print recent evaluations of the jobset if no package is specified
- Add an `--eval` flag for information about a specific evaluation
- Infer the current stable Nixpkgs release (e.g. `24.05`) with a hack
- Support standard channel names (e.g. `nixos-unstable`)
- Generate shell completions with `--shell-completion SHELL`
- Print nicely formatted, colored and aligned tables
