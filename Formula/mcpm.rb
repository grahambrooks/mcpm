class Mcpm < Formula
  desc "Model Context Protocol Server Manager"
  homepage "https://github.com/grahambrooks/mcpm"
  version "2026.7.1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/grahambrooks/mcpm/releases/download/v2026.7.1/mcpm-v2026.7.1-aarch64-apple-darwin.tar.gz"
      sha256 "c0366d0c94f1edd2b3fa98adfcfda66e5351365eb029f19a4cc7a849eea49890"
    end
    on_intel do
      odie "Intel Mac binaries are not provided. Run `cargo install --git https://github.com/grahambrooks/mcpm --locked` to build from source."
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/grahambrooks/mcpm/releases/download/v2026.7.1/mcpm-v2026.7.1-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "2516345ac7e94b485cbe2ef90c36497b4c78db25329439d8a567156be5b7c730"
    end
    on_intel do
      url "https://github.com/grahambrooks/mcpm/releases/download/v2026.7.1/mcpm-v2026.7.1-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "8630472be3e8bf3b6ef24270aaa3d7dd56638473b9fcbced669625ced77faf95"
    end
  end

  def install
    bin.install "mcpm"
  end

  test do
    assert_path_exists bin/"mcpm"
  end
end
