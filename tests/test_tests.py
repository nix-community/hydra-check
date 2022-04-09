import pytest

from hydra_check import cli


def test_get_url() -> None:
    assert cli.get_url("unstable") == "https://hydra.nixos.org/job/unstable"
