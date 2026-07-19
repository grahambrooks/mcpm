class . < Formula
  desc "A grahambrooks command-line tool"
  homepage "https://github.com/grahambrooks/mcpm"
  version "2026.7.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/grahambrooks/mcpm/releases/download/v2026.7.0/mcpm-v2026.7.0-aarch64-apple-darwin.tar.gz"
      sha256 "b494463c88415ba46033e08c6c6f84214ab6c22acd795772ce771109d7c26a8f"
    end
    on_intel do
      odie "Intel Mac binaries are not provided. Run `cargo install --git https://github.com/grahambrooks/mcpm --locked` to build from source."
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/grahambrooks/mcpm/releases/download/v2026.7.0/mcpm-v2026.7.0-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "872ced15caaa8352e840c9546a686b55e97ad779128be0a3e8a2773c94afafcf"
    end
    on_intel do
      url "https://github.com/grahambrooks/mcpm/releases/download/v2026.7.0/mcpm-v2026.7.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "f4cb1108dafda74690b704a654d0e70bba5125f594488ee32015f807d32c4c4c"
    end
  end

  def install
    bin.install "mcpm"
  end

  test do
    assert_path_exists bin/"mcpm"
  end
end
