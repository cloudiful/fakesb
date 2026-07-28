#!/usr/bin/env nu

const SCRIPT_DIR = (path self | path dirname)
const ROOT_DIR = ($SCRIPT_DIR | path dirname)

def print-startup-summary [mode: string] {
  print $"Starting fakESB local dev \(( $mode ))"
  print "  server   http://127.0.0.1:3000"
  print "  health   http://127.0.0.1:3000/healthz"
  if $mode == "full" {
    print "  web      http://127.0.0.1:5173"
    print "  db       postgresql://127.0.0.1:5434/postgres"
  }
  print "Stop with Ctrl+C"
}

def repo-root [] {
  $ROOT_DIR
}

def env-with-defaults [] {
  {
    DATABASE_URL: ($env.DATABASE_URL? | default ""),
    RUST_LOG: ($env.RUST_LOG? | default "info"),
    FAKESB_SERVER__HOST: ($env.FAKESB_SERVER__HOST? | default "127.0.0.1"),
    FAKESB_SERVER__PORT: ($env.FAKESB_SERVER__PORT? | default "3000"),
  }
}

def ensure-database-url [] {
  let values = (env-with-defaults)
  if ($values.DATABASE_URL | is-empty) {
    error make {
      msg: "DATABASE_URL is empty"
      help: "Set DATABASE_URL in the shell environment."
    }
  }
}

def endpoint-ready [url: string] {
  try {
    http get --allow-errors --max-time 500ms $url | ignore
    true
  } catch {
    false
  }
}

def wait-for-server [] {
  print "Waiting for fakESB server..."
  for _ in 1..120 {
    if (endpoint-ready "http://127.0.0.1:3000/healthz") {
      return true
    }
    sleep 250ms
  }
  print "Warning: timed out waiting for http://127.0.0.1:3000/healthz"
  false
}

def run-server [] {
  let root = (repo-root)
  with-env (env-with-defaults) {
    cd ($root | path join "server")
    ^cargo run
  }
}

def run-web [] {
  let root = (repo-root)
  let cache_dir = ($root | path join ".cache" "bun")
  mkdir $cache_dir
  with-env {
    TMPDIR: "/private/tmp",
    BUN_INSTALL_CACHE_DIR: $cache_dir,
  } {
    cd ($root | path join "web")
    if not ("node_modules" | path exists) {
      ^bun install --minimum-release-age 604800
    }
    with-env { NUXT_PUBLIC_API_BASE: "http://127.0.0.1:3000" } {
      ^bun run dev --host 127.0.0.1 --port 5173
    }
  }
}

def run-backend [] {
  ensure-database-url
  print-startup-summary "backend"
  run-server
}

def run-full [] {
  ensure-database-url
  print-startup-summary "full"

  let server_job = (job spawn { run-server })
  wait-for-server | ignore
  let web_job = (job spawn { run-web })

  try {
    loop {
      let active = (job list | get id)
      if not ($server_job.id in $active) {
        break
      }
      if not ($web_job.id in $active) {
        break
      }
      sleep 500ms
    }
  } finally {
    job kill $server_job.id
    job kill $web_job.id
  }
}

def main [command?: string] {
  let action = ($command | default "help")

  match $action {
    "backend" => { run-backend }
    "full" => { run-full }
    "help" => {
      print "Usage: nu scripts/dev.nu <backend|full>"
      print "Commands:"
      print "  backend   Start only the Rust API on http://127.0.0.1:3000"
      print "  full      Start Rust API and Nuxt web dev server"
      print "Config:"
      print "  DATABASE_URL is read from the shell environment"
      print "  FAKESB_SERVER__HOST defaults to 127.0.0.1"
      print "  FAKESB_SERVER__PORT defaults to 3000"
    }
    _ => {
      error make {
        msg: $"unknown command: ($action)"
        help: "Run `nu scripts/dev.nu help` for usage."
      }
    }
  }
}
