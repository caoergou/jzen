class Jed < Formula
  desc "A dual-interface JSON editor for humans and AI agents"
  homepage "https://github.com/caoergou/jed"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/caoergou/jed/releases/download/v0.1.0/jed-macos-aarch64"
      sha256 "please_run_brew_style"
    else
      url "https://github.com/caoergou/jed/releases/download/v0.1.0/jed-macos-x86_64"
      sha256 "please_run_brew_style"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/caoergou/jed/releases/download/v0.1.0/jed-linux-x86_64"
      sha256 "please_run_brew_style"
    else
      url "https://github.com/caoergou/jed/releases/download/v0.1.0/jed-linux-aarch64"
      sha256 "please_run_brew_style"
    end
  end

  def install
    bin.install Dir["*"].find { |f| f =~ /jed-/ && !f.end_with?(".sha256") } => "jed"
  end

  test do
    system "#{bin}/jed", "--version"
  end
end
