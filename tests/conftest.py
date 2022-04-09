# pylint: disable=invalid-name
#from dataclasses import dataclass
from pathlib import Path
from typing import Type

import pytest

TEST_ROOT = Path(__file__).parent.resolve()


class Helpers:
    @staticmethod
    def root() -> Path:
        return TEST_ROOT

    @staticmethod
    def read_asset(asset: str) -> str:
        return str(Path(TEST_ROOT.joinpath("assets", asset)).read_text("utf-8"))


@pytest.fixture
def helpers() -> Type[Helpers]:
    return Helpers
