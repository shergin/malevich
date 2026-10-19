import io

import pytest

from supremus._cli import main, parse_bars, parse_numbers


def run(argv: list[str], stdin: str, monkeypatch, capsys) -> tuple[int, str, str]:
    monkeypatch.setattr("sys.stdin", io.StringIO(stdin))
    monkeypatch.setenv("COLUMNS", "40")
    monkeypatch.setenv("LINES", "30")
    code = main(argv)
    out, err = capsys.readouterr()
    return code, out, err


def test_line_from_stdin(monkeypatch, capsys) -> None:
    code, out, err = run(["line"], "1 5 2 8", monkeypatch, capsys)
    assert code == 0 and err == ""
    assert "┤" in out and "\x1b" not in out


def test_hist_and_bar(monkeypatch, capsys) -> None:
    code, out, _ = run(["hist"], " ".join(str(i % 7) for i in range(50)), monkeypatch, capsys)
    assert code == 0 and "┤" in out
    code, out, _ = run(["bar"], "mon\t3\ntue\t7\n", monkeypatch, capsys)
    assert code == 0 and "mon" in out and "tue" in out


def test_help_and_errors(monkeypatch, capsys) -> None:
    code, out, _ = run(["--help"], "", monkeypatch, capsys)
    assert code == 0 and "Usage" in out
    code, _, err = run(["line"], "", monkeypatch, capsys)
    assert code == 1 and "expected numbers" in err
    code, _, err = run(["pie"], "", monkeypatch, capsys)
    assert code == 1 and "unknown command" in err


def test_showcase_runs(monkeypatch, capsys) -> None:
    code, out, _ = run([], "", monkeypatch, capsys)
    assert code == 0
    assert "training" in out and "hist(samples)" in out


def test_parsers() -> None:
    assert parse_numbers("1, 2 nan 3 x") == [1.0, 2.0, None, 3.0]
    assert parse_bars("a\t1\nb 2\n\nc\tx\n") == (["a", "b"], [1.0, 2.0])


@pytest.mark.parametrize("argv", [["help"], ["-h"]])
def test_help_aliases(argv, monkeypatch, capsys) -> None:
    assert run(argv, "", monkeypatch, capsys)[0] == 0
