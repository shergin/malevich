from supremus import Frame, Theme
from supremus.graphics import detect_graphics


def test_plain_and_portable_are_deterministic() -> None:
    assert Frame.plain(40, 10).to_json() == {
        "width": 40,
        "height": 10,
        "charset": "Braille",
        "color": "Plain",
        "theme": {"palette": ["Cyan", "Yellow", "Green", "Magenta", "Blue", "Red"]},
    }
    assert Frame.portable(40, 10).charset == "Quadrants"
    assert Frame.plain(40, 10).with_(height=8).height == 8
    assert Frame.plain(40, 10).with_(theme=Theme.LIGHT).theme.is_light


def test_detect_on_a_pipe_is_plain_and_sized_by_env(pipe) -> None:
    frame = Frame.detect(pipe, env={"COLUMNS": "100", "LINES": "45", "LANG": "en_US.UTF-8"})
    assert (frame.width, frame.height) == (100, 15)
    assert frame.color == "Plain"
    assert frame.charset == "Quadrants"
    assert Frame.detect(pipe, env={}).width == 80
    assert Frame.detect(pipe, env={}).height == 16


def test_detect_color_ladder(tty, pipe) -> None:
    assert Frame.detect(tty, env={"NO_COLOR": "", "COLORTERM": "truecolor"}).color == "Plain"
    assert Frame.detect(tty, env={"COLORTERM": "truecolor"}).color == "TrueColor"
    assert Frame.detect(tty, env={"TERM": "xterm-direct"}).color == "TrueColor"
    assert Frame.detect(tty, env={"TERM": "xterm-256color"}).color == "Ansi256"
    assert Frame.detect(tty, env={"TERM": "xterm"}).color == "Ansi16"
    assert Frame.detect(tty, env={"TERM": "screen-256color", "COLORTERM": "truecolor"}).color == "Ansi256"
    assert Frame.detect(tty, env={"TERM": "screen"}).color == "Ansi16"
    assert Frame.detect(tty, env={"TERM": "dumb"}).color == "Plain"
    assert Frame.detect(pipe, env={"FORCE_COLOR": "1", "TERM": "xterm"}).color == "Ansi16"
    assert Frame.detect(pipe, env={"CLICOLOR_FORCE": "0", "TERM": "xterm"}).color == "Plain"


def test_detect_charset_ladder(tty) -> None:
    assert Frame.detect(tty, env={"MALEVICH_CHARSET": "braille"}).charset == "Braille"
    assert Frame.detect(tty, env={"SUPREMUS_CHARSET": "octants"}).charset == "Octants"
    assert Frame.detect(tty, env={"TERM": "dumb"}).charset == "Ascii"
    assert Frame.detect(tty, env={"LC_ALL": "C", "LANG": "en_US.UTF-8"}).charset == "Ascii"
    assert Frame.detect(tty, env={"LANG": "en_US.UTF-8"}).charset == "Quadrants"
    assert Frame.detect(tty, env={}).charset == "Quadrants"


def test_theme_detect() -> None:
    assert Theme.detect({"COLORFGBG": "0;15"}) == Theme.LIGHT
    assert Theme.detect({"COLORFGBG": "15;0"}) == Theme.DARK
    assert Theme.detect({}) == Theme.DARK


def test_graphics_detect() -> None:
    assert detect_graphics({}) is None
    assert detect_graphics({"KITTY_WINDOW_ID": "1"}).protocol == "kitty"
    assert detect_graphics({"TERM_PROGRAM": "iTerm.app"}).protocol == "iterm2"
    assert detect_graphics({"TERM": "xterm-kitty", "TMUX": "/tmp/x"}) is None
    assert detect_graphics({"MALEVICH_GRAPHICS": "none", "KITTY_WINDOW_ID": "1"}) is None
    assert detect_graphics({"MALEVICH_GRAPHICS": "sixel"}).protocol == "sixel"
    assert detect_graphics({"WT_SESSION": "x"}).protocol == "sixel"
