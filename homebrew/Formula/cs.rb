# typed: false
# frozen_string_literal: true

class Cs < Formula
  desc "Claude Code Session Manager - deterministic sessions based on folder+branch"
  homepage "https://github.com/bikramtuladhar/claude-code-resumer"
  # x-release-please-start-version
  version "0.1.1"
  # x-release-please-end
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v#{version}/cs-macos-arm64"
      sha256 "a9a58da2e48c1ae43059d5fab714f1cc4a2012fd46cfd32bc308772f5a8ca02d"

      def install
        bin.install "cs-macos-arm64" => "cs"
      end
    end

    on_intel do
      url "https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v#{version}/cs-macos-intel"
      sha256 "f9ba34e8cae43402eb68a8a3732ab76a295b2a061a976786ed1a735260954276"

      def install
        bin.install "cs-macos-intel" => "cs"
      end
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v#{version}/cs-linux-arm64"
      sha256 "e3ee57579ba726bb71341e4b50e1d2193ad531f91d08dca260a645c45025e077"

      def install
        bin.install "cs-linux-arm64" => "cs"
      end
    end

    on_intel do
      url "https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v#{version}/cs-linux-x64"
      sha256 "6cabefcf4d28a40e8ce91bfb1bb68290554752591a04d1991d43da4d14a3fefb"

      def install
        bin.install "cs-linux-x64" => "cs"
      end
    end
  end

  test do
    assert_match "cs #{version}", shell_output("#{bin}/cs --version")
  end
end
