# vite-plugin-surrealkit

Run `surrealkit sync` from Vite so you do not need a separate `surrealkit sync --watch` process.

## Install

```sh
npm i -D vite-plugin-surrealkit
```

## Usage

```ts
// vite.config.ts
import { defineConfig } from 'vite';
import { surrealkitPlugin } from 'vite-plugin-surrealkit';

export default defineConfig({
  plugins: [
    surrealkitPlugin({
      // Optional: pass flags to `surrealkit sync`
      syncArgs: ['--allow-shared-prune'],
    }),
  ],
});
```

Default behavior:

- Runs `surrealkit sync` on dev server startup
- Watches `database/schema/**/*.surql`
- Runs `surrealkit sync` again whenever those files are added/changed/removed
- Debounces and queues runs to avoid overlapping processes

## Options

```ts
type RunMode = 'serve' | 'build';
type LogLevel = 'silent' | 'error' | 'info' | 'debug';

interface SurrealkitPluginOptions {
  binary?: string;
  cwd?: string;
  env?: Record<string, string>;
  syncArgs?: string[];
  schemaGlobs?: string[];
  include?: RunMode[];
  runOnStartup?: boolean;
  reloadOnSync?: boolean;
  debounceMs?: number;
  logLevel?: LogLevel;
  failBuildOnError?: boolean;
}
```

## Vite Compatibility

- Vite `^7.0.0`
- Vite `^8.0.0` (peer range included for forward compatibility)
