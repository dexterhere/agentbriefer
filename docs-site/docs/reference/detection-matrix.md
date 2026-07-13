---
title: Detection matrix
description: Languages, manifest files, and package managers Agentbriefer can detect.
---

# Detection matrix

Agentbriefer inspects common project manifests during `init`. Detection fills in an editable draft;
it never replaces your judgment about the project's intended architecture or policy.

| Ecosystem | Signals | Typical package manager |
| --- | --- | --- |
| Rust | `Cargo.toml` | Cargo |
| Node.js / TypeScript | `package.json` and lockfiles | npm, pnpm, Yarn, or Bun |
| Go | `go.mod` | Go modules |
| Python | `pyproject.toml`, `requirements.txt` | Project-dependent |
| PHP | `composer.json` | Composer |
| Java / Maven | `pom.xml` | Maven |
| C# | `*.csproj` | NuGet / dotnet |
| F# | `*.fsproj` | NuGet / dotnet |
| Dart | `pubspec.yaml` | pub |
| Julia | `Project.toml` | Pkg |
| Crystal | `shard.yml` | Shards |
| Haskell | `package.yaml`, `*.cabal`, `stack.yaml` | Cabal or Stack |
| R | `DESCRIPTION`, `renv.lock` | renv when present |
| Ruby | `Gemfile`, `Gemfile.lock` | Bundler |
| Swift | `Package.swift` | Swift Package Manager |
| Scala | `build.sbt` | sbt |
| JVM / Gradle | `build.gradle`, `build.gradle.kts` | Gradle |
| C / C++ | `vcpkg.json`, `conanfile.txt`, `CMakeLists.txt` | vcpkg, Conan, or CMake |
| Elixir | `mix.exs` | Mix |
| Clojure | `deps.edn`, `project.clj` | Clojure CLI or Leiningen |
| Erlang | `rebar.config` | Rebar3 |
| Objective-C | `Podfile` | CocoaPods |
| Lua | `*.rockspec` | LuaRocks |
| Perl | `cpanfile` | cpanm / Carton |
| Visual Basic .NET | `*.vbproj` | NuGet / dotnet |
| Nim | `*.nimble` | Nimble |
| PowerShell | `*.psd1` | PowerShellGet / PSResourceGet |
| Zig | `build.zig.zon` | Zig package manager |

## Monorepos and mixed stacks

If several signals exist, Agentbriefer combines the evidence it understands and presents a proposal.
Run `init` from the directory whose scope should receive the generated instructions:

```bash
cd ./services/payments
agentbriefer init
```

For a repository-wide policy plus package-specific policy, maintain an Agentbriefer configuration at
each intentional instruction boundary.
