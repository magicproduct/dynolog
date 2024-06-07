// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use std::net::TcpStream;
use std::net::ToSocketAddrs;

use anyhow::Result;
use clap::Parser;

// Make all the command modules accessible to this file.
mod commands;
use commands::gputrace::GpuTraceConfig;
use commands::gputrace::GpuTraceOptions;
use commands::gputrace::GpuTraceTriggerConfig;
use commands::*;

/// Instructions on adding a new Dyno CLI command:
///
/// 1. Add a new variant to the `Command` enum.
///    Please include a description of the command and, if applicable, its flags/subcommands.
///
/// 2. Create a new file for the command's implementation in the commands/ directory (ie
///    commands/status.rs). This new file is where the command should be implemented.
///    Make the new command's module accessible from this file by adding
///    a new line with `pub mod <newfile>;` to commands/mod.rs.
///
///
/// 3. Add a branch to the match statement in main() to handle the new enum variant (from step 1).
///    From here, invoke the handling logic defined in the new file (from step 2). In an effort to keep
///    the command dispatching logic clear and concise, please keep the code in the match branch to a minimum.

const DYNO_PORT: u16 = 1778;

#[derive(Debug, Parser)]
struct Opts {
    #[clap(long, default_value = "localhost")]
    hostname: String,
    #[clap(long, default_value_t = DYNO_PORT)]
    port: u16,
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Parser)]
enum Command {
    /// Check the status of a dynolog process
    Status,
    /// Check the version of a dynolog process
    Version,
    /// Capture gputrace
    Gputrace(gputrace::Options),
    /// Pause dcgm profiling. This enables running tools like Nsight compute and avoids conflicts.
    DcgmPause {
        /// Duration to pause dcgm profiling in seconds
        #[clap(long, default_value_t = 300)]
        duration_s: i32,
    },
    /// Resume dcgm profiling
    DcgmResume,

    /// Run a single command on multiple hosts
    Batch (batch::Options),
}



/// Create a socket connection to dynolog
fn create_dyno_client(host: &str, port: u16) -> Result<TcpStream> {
    let addr = (host, port)
        .to_socket_addrs()?
        .next()
        .expect("Failed to connect to the server");

    TcpStream::connect(addr).map_err(|err| err.into())
}

fn main() -> Result<()> {
    let Opts {
        hostname,
        port,
        cmd,
    } = Opts::parse();

    let dyno_client =
        create_dyno_client(&hostname, port).expect("Couldn't connect to the server...");

    match cmd {
        Command::Status => status::run_status(dyno_client),
        Command::Version => version::run_version(dyno_client),
        Command::Gputrace (opts) => gputrace::run_gputrace_from_opts(dyno_client, opts),
        Command::DcgmPause { duration_s } => dcgm::run_dcgm_pause(dyno_client, duration_s),
        Command::DcgmResume => dcgm::run_dcgm_resume(dyno_client),
        Command::Batch (batch::Options{ hosts, cmd }) => batch::run_batch(hosts, cmd),
        // ... add new commands here
    }
}
