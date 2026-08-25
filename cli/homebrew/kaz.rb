# Homebrew formula for kaz (the malevich CLI). Builds from source with cargo and
# installs the binary, shell completions, and the man page.
#
# Canonical copy; the live formula is Formula/kaz.rb in shergin/homebrew-tap.
# To bump: retag `cli-vX.Y.Z` on shergin/malevich, then set url + sha256 with
#   curl -sL <url> | shasum -a 256
class Kaz < Formula
  desc "Pipe data to an honest terminal plot"
  homepage "https://github.com/shergin/malevich"
  url "https://github.com/shergin/malevich/archive/refs/tags/cli-v0.2.1.tar.gz"
  sha256 "49ebd46de3c0a85f001afc7f8795cb68891b923ccd4bf57f8686bf8ae9f2a88a"
  license any_of: ["MIT", "Apache-2.0"]
  head "https://github.com/shergin/malevich.git", branch: "main"

  depends_on "rust" => :build

  def install
    # `kaz` is the `malevich-cli` workspace member; --path builds just it.
    system "cargo", "install", *std_cargo_args(path: "cli")

    # Hand-written completions and man page ship with the crate.
    bash_completion.install "cli/completions/kaz.bash" => "kaz"
    zsh_completion.install "cli/completions/kaz.zsh" => "_kaz"
    fish_completion.install "cli/completions/kaz.fish"
    man1.install "cli/man/kaz.1"
  end

  test do
    # The version prints.
    assert_match "kaz", shell_output("#{bin}/kaz --version")

    # A plain plot draws: ascii marks come out as '*'.
    output = pipe_output(
      "#{bin}/kaz line -o - --color never --charset ascii -w 20 -h 6",
      "1\n4\n2\n8\n5\n",
    )
    assert_match "*", output
  end
end
