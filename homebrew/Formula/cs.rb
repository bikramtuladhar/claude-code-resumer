# typed: false
# frozen_string_literal: true

class Cs < Formula
  desc "Claude Code Session Manager - deterministic sessions based on folder+branch"
  homepage "https://github.com/bikramtuladhar/claude-code-resumer"
  version "1.0.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v#{version}/cs-macos-arm64"
      sha256 "SHA256_MACOS_ARM64"

      def install
        bin.install "cs-macos-arm64" => "cs"
      end
    end

    on_intel do
      url "https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v#{version}/cs-macos-intel"
      sha256 "SHA256_MACOS_INTEL"

      def install
        bin.install "cs-macos-intel" => "cs"
      end
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v#{version}/cs-linux-arm64"
      sha256 "SHA256_LINUX_ARM64"

      def install
        bin.install "cs-linux-arm64" => "cs"
      end
    end

    on_intel do
      url "https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v#{version}/cs-linux-x64"
      sha256 "SHA256_LINUX_X64"

      def install
        bin.install "cs-linux-x64" => "cs"
      end
    end
  end

  test do
    assert_match "cs #{version}", shell_output("#{bin}/cs --version")
  end
end
