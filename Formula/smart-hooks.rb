class SmartHooks < Formula
  desc "Intelligent git hooks that understand your code changes and automatically select the right tests to run"
  homepage "https://github.com/CaptainEmpower/smart-hooks"
  url "https://github.com/CaptainEmpower/smart-hooks/archive/refs/tags/v0.2.0.tar.gz"
  sha256 "TO_BE_UPDATED_AFTER_RELEASE"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/smart-hooks", "--help"
    system "#{bin}/smart-hooks", "test", "--help"
    system "#{bin}/smart-hooks", "analyze", "--help"
    system "#{bin}/smart-hooks", "check", "--help"
  end
end