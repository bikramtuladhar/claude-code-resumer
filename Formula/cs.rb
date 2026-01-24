# typed: false
# frozen_string_literal: true

class Cs < Formula
  desc "Claude Code Session Manager - deterministic sessions based on folder+branch"
  homepage "https://github.com/bikramtuladhar/claude-code-resumer"
  version "0.3.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v#{version}/cs-macos-arm64"
      sha256 "e5ccb601329e2031cf7a4b3d0031b4707debe26076c1aa9000454b71009cb537"

      def install
        bin.install "cs-macos-arm64" => "cs"
      end
    end

    on_intel do
      url "https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v#{version}/cs-macos-intel"
      sha256 "431f423a777fe903522774f64ee430a5e85fe428e136976af91ccc2b95586352"

      def install
        bin.install "cs-macos-intel" => "cs"
      end
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v#{version}/cs-linux-arm64"
      sha256 "413d57bd4289cf01b9e77fcdfc8965d0af3893e0488532104bf7af79e33449c7"

      def install
        bin.install "cs-linux-arm64" => "cs"
      end
    end

    on_intel do
      url "https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v#{version}/cs-linux-x64"
      sha256 "3bc9af80419177307c4ea47af438a7fa2abc8ad34d40248fafab5b3b76016d26"

      def install
        bin.install "cs-linux-x64" => "cs"
      end
    end
  end

  test do
    assert_match "cs #{version}", shell_output("#{bin}/cs --version")
  end
end
