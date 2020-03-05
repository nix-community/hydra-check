# hydra-check

check hydra for the build status of a package in a given channel

# usage

```
$ ./hydra-check --help
usage: hydra-check [options] PACKAGE [CHANNEL]
...

$ ./hydra-check hello
✔ hello-2.10 https://hydra.nixos.org/build/113804835

$ ./hydra-check hello 19.03
✔ hello-2.10 https://hydra.nixos.org/build/103243113

$ ./hydra-check ugarit 19.09 --short
✖ (Dependency failed) chicken-ugarit-2.0 https://hydra.nixos.org/build/108216732

$ ./hydra-check nixos.tests.installer.simpleUefiGrub 19.09 --arch aarch64-linux
✖ (Failed) vm-test-run-installer-simpleUefiGrub https://hydra.nixos.org/build/113892497

Last Builds:
✖ (Failed) vm-test-run-installer-simpleUefiGrub https://hydra.nixos.org/build/113857097
✔ vm-test-run-installer-simpleUefiGrub https://hydra.nixos.org/build/113855182
✔ vm-test-run-installer-simpleUefiGrub https://hydra.nixos.org/build/113849540
✔ vm-test-run-installer-simpleUefiGrub https://hydra.nixos.org/build/113448926
...

```

