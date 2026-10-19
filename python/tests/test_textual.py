import asyncio

import pytest

textual = pytest.importorskip("textual")

from textual.app import App, ComposeResult  # noqa: E402

from supremus import line  # noqa: E402
from supremus.textual import PlotState, PlotWidget  # noqa: E402


class Static(App[None]):
    def compose(self) -> ComposeResult:
        yield PlotWidget(line([1, 5, 2, 8]).title("hello"), id="plot")


class Interactive(App[None]):
    def __init__(self) -> None:
        super().__init__()
        self.state = PlotState()

    def compose(self) -> ComposeResult:
        yield PlotWidget(line([1, 5, 2, 8, 3, 9, 4, 10]), state=self.state, id="plot")


def text_of(strip) -> str:
    return "".join(segment.text for segment in strip)


def test_widget_paints_the_plot_at_its_size() -> None:
    async def scenario() -> None:
        app = Static()
        async with app.run_test(size=(60, 16)) as pilot:
            await pilot.pause()
            widget = app.query_one("#plot", PlotWidget)
            assert widget.content_size.width == 60
            assert "hello" in text_of(widget.render_line(0))
            axis = text_of(widget.render_line(1))
            assert "┤" in axis or "│" in axis
            widget.plot = line([2, 2, 2]).title("swapped")
            await pilot.pause()
            assert "swapped" in text_of(widget.render_line(0))

    asyncio.run(scenario())


def test_widget_feeds_the_controller() -> None:
    async def scenario() -> None:
        app = Interactive()
        async with app.run_test(size=(60, 16)) as pilot:
            await pilot.pause()
            widget = app.query_one("#plot", PlotWidget)
            widget.focus()
            await pilot.pause()
            assert app.state.mapping() is not None
            await pilot.hover("#plot", offset=(30, 8))
            await pilot.pause()
            assert app.state.cursor() == (30, 8)
            await pilot.press("plus")
            await pilot.pause()
            assert "x" in app.state.viewport()
            await pilot.press("r")
            await pilot.pause()
            assert app.state.viewport() == {}

    asyncio.run(scenario())
