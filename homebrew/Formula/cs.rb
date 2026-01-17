# typed: false
# frozen_string_literal: true

class Cs < Formula
  desc "Claude Code Session Manager - deterministic sessions based on folder+branch"
  homepage "https://github.com/bikramtuladhar/claude-code-resumer"
  # x-release-please-start-version
  version "0.1.0"
  # x-release-please-end
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v#{version}/cs-macos-arm64"
      sha256 "36e7b1abd627572152f47b603f17a8930897518dcd5a29932edc900033369e54"

      def install
        bin.install "cs-macos-arm64" => "cs"
      end
    end

    on_intel do
      url "https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v#{version}/cs-macos-intel"
      sha256 "0953f1fc5eeae008c6bed85c8bdcc55db8262a984e92cef18e8188d545e1abe4"

      def install
        bin.install "cs-macos-intel" => "cs"
      end
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v#{version}/cs-linux-arm64"
      sha256 "13b8bf617984d1fd4713dcc705cf66c50cc9c6a16dd6a831ff752332de1e0bfa"

      def install
        bin.install "cs-linux-arm64" => "cs"
      end
    end

    on_intel do
      url "https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v#{version}/cs-linux-x64"
      sha256 "06a1b87f1951b58d3200b8e4c5fc3e47f76c62edd69a71a3ee877ff49258e4f6"

      def install
        bin.install "cs-linux-x64" => "cs"
      end
    end
  end

  test do
    assert_match "cs #{version}", shell_output("#{bin}/cs --version")
  end
end
