import { defineConfig } from "vitest/config";

// npm test（src/**/*.test.ts）とは別の設定。ホストの WinAppDriver を叩くので
// 直列に動かし、GUI の起動やキー送信のぶんタイムアウトを長めにする。
export default defineConfig({
  test: {
    include: ["e2e/**/*.test.ts"],
    fileParallelism: false,
    testTimeout: 60_000,
    hookTimeout: 60_000,
    // compose の TTY では回転表示だけ見えて止まるように見える。
    reporters: ["verbose"],
  },
});
