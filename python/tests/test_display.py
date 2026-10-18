import pytest

from supremus import Frame, Grid, line

rich = pytest.importorskip("rich")


def test_str_and_repr_detect_a_frame(monkeypatch) -> None:
    monkeypatch.setenv("NO_COLOR", "1")
    monkeypatch.setenv("COLUMNS", "60")
    monkeypatch.setenv("LINES", "30")
    text = str(line([1, 5, 2, 8]))
    assert "\x1b" not in text
    assert "┤" in text
    assert repr(line([1, 5, 2, 8])) == text


def test_mimebundle_carries_text_and_the_card() -> None:
    plot = line([1, 5, 2, 8]).title("card")
    bundle = plot._repr_mimebundle_()
    assert set(bundle) == {"text/plain", "text/html"}
    assert bundle["text/html"].startswith("<pre")
    assert "card" in bundle["text/html"]
    assert "<script" not in bundle["text/html"] and "<style" not in bundle["text/html"]
    assert set(plot._repr_mimebundle_(include=["text/plain"])) == {"text/plain"}
    assert set(plot._repr_mimebundle_(exclude=["text/plain"])) == {"text/html"}
    assert plot._repr_html_() == bundle["text/html"]
    assert plot._mime_() == ("text/html", bundle["text/html"])
    assert plot.to_html(Frame.portable(20, 6)) != plot.to_html()
    assert plot.to_svg().startswith("<svg")


def test_rich_renderable_fills_a_panel() -> None:
    from rich.console import Console
    from rich.panel import Panel

    console = Console(width=50, force_terminal=True, color_system="truecolor", record=True)
    console.print(Panel(line([1, 5, 2, 8]), title="rich"))
    text = console.export_text()
    lines = text.splitlines()
    assert lines[0].startswith("╭") and lines[-1].startswith("╰")
    assert all(len(l) == 50 for l in lines), "every panel row is exactly the console width"
    assert any("┤" in l for l in lines)
    assert any("3" in l for l in lines), "the x axis labels survive"


def test_rich_live_height_is_honored() -> None:
    from rich.console import Console

    console = Console(width=40, force_terminal=True, record=True)
    console.print(line([1, 2, 3]), height=6)
    assert len(console.export_text().splitlines()) == 6


def test_grid_is_a_rich_renderable_too() -> None:
    from rich.console import Console

    console = Console(width=60, force_terminal=True, record=True)
    console.print(Grid(2).with_(line([1, 2])).with_(line([2, 1])))
    assert "┤" in console.export_text()


def test_measure_keeps_a_floor() -> None:
    from rich.console import Console, ConsoleOptions

    console = Console(width=30)
    measurement = line([1, 2]).__rich_measure__(console, console.options)
    assert measurement.minimum == 20 and measurement.maximum == 30
