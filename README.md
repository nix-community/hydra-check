# hydra-check

check hydra for the build status of a package in a given channel.

# Disclaimer
Keep in mind that hydra is the NixOS build-farm orchestrator and has more important tasks to do than answering your puny requests. Response time may be in the seconds for each request.

# Usage

```console
$ nix-shell

$ hydra-check --help
usage: hydra-check [options] PACKAGE [CHANNEL]
...

$ hydra-check hello
✔ hello-2.10 https://hydra.nixos.org/build/113804835

$ hydra-check hello 19.03
✔ hello-2.10 https://hydra.nixos.org/build/103243113

$ hydra-check nixos.tests.installer.simpleUefiGrub 19.09 --arch aarch64-linux
✖ (Failed) vm-test-run-installer-simpleUefiGrub https://hydra.nixos.org/build/113892497

Last Builds:
✖ (Failed) vm-test-run-installer-simpleUefiGrub https://hydra.nixos.org/build/113857097
✔ vm-test-run-installer-simpleUefiGrub https://hydra.nixos.org/build/113855182
✔ vm-test-run-installer-simpleUefiGrub https://hydra.nixos.org/build/113849540
✔ vm-test-run-installer-simpleUefiGrub https://hydra.nixos.org/build/113448926
...

$ hydra-check ugarit 19.09 --short
✖ (Dependency failed) chicken-ugarit-2.0 https://hydra.nixos.org/build/108216732

$ hydra-check nixos.containerTarball 19.09 --arch i686-linux --json | jq .
[
  {
    "icon": "✖",
    "success": false,
    "status": "Failed",
    "timestamp": "2020-03-05T15:03:15Z",
    "build_id": "113892448",
    "build_url": "https://hydra.nixos.org/build/113892448",
    "name": "tarball",
    "arch": "i686-linux"
  },
  {
    "icon": "✔",
    "success": true,
    "status": "Succeeded",
    "timestamp": "2020-03-04T14:56:01Z",
    "build_id": "113857110",
    "build_url": "https://hydra.nixos.org/build/113857110",
    "name": "tarball",
    "arch": "i686-linux"
  },
  {
    "icon": "✖",
    "success": false,
    "status": "Failed",
    "timestamp": "2020-03-04T02:52:56Z",
    "build_id": "113855194",
    "build_url": "https://hydra.nixos.org/build/113855194",
    "name": "tarball",
    "arch": "i686-linux"
  },
  ...
]

```
