# typed: false
# frozen_string_literal: true

class Cs < Formula
  desc "Claude Code Session Manager - deterministic sessions based on folder+branch"
  homepage "https://github.com/bikramtuladhar/claude-code-resumer"
  version "0.1.4"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v#{version}/cs-macos-arm64"
      sha256 "bcbf0d5d4968c4df33adf21817cbb2e84c8c5f264ab750c2086b19733b609ffb"

      def install
        bin.install "cs-macos-arm64" => "cs"
      end
    end

    on_intel do
      url "https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v#{version}/cs-macos-intel"
      sha256 "0bfa7829a037aab9e5f7a038019f47dab4ab3e91e1222cb907ed1fd7b9d7efb0"

      def install
        bin.install "cs-macos-intel" => "cs"
      end
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v#{version}/cs-linux-arm64"
      sha256 "66f6a45561b4ce62fcf0f45d052f1a9a5b5674570e57fef9c6562f15afa5044b"

      def install
        bin.install "cs-linux-arm64" => "cs"
      end
    end

    on_intel do
      url "https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v#{version}/cs-linux-x64"
      sha256 "a988ac84ab3caccea920c618151978af5238efe014cfd472b47525e937749b03"

      def install
        bin.install "cs-linux-x64" => "cs"
      end
    end
  end

  test do
    assert_match "cs #{version}", shell_output("#{bin}/cs --version")
  end
end
