# hydra-check

check hydra for the build status of a package in a given channel.

# Disclaimer
Keep in mind that hydra is the NixOS build-farm orchestrator and has more important tasks to do than answering your puny requests. Response time may be in the seconds for each request.

# Usage

```console
$ nix-shell

$ hydra-check --help
usage: hydra-check [options] PACKAGES...
...

$ hydra-check hello
Build Status for nixpkgs.hello.x86_64-linux on unstable
✔ hello-2.10 from 2020-03-14 - https://hydra.nixos.org/build/114752982

$ hydra-check hello python --channel 19.03
Build Status for nixpkgs.hello.x86_64-linux on 19.03
✔ hello-2.10 from 2019-10-14 - https://hydra.nixos.org/build/103243113
Build Status for nixpkgs.python.x86_64-linux on 19.03
✔ python-2.7.17 from 2020-01-14 - https://hydra.nixos.org/build/110523905


$ hydra-check nixos.tests.installer.simpleUefiGrub --channel 19.09 --arch aarch64-linux
Build Status for nixos.tests.installer.simpleUefiGrub.aarch64-linux on 19.09
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
Build Status for nixpkgs.ugarit.x86_64-linux on 19.09
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

```
