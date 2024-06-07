// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use std::net::TcpStream;
use anyhow::Result;
use clap::Parser;
use crate::gputrace;


#[derive(Debug, Parser)]
pub enum Command {
    /// Capture gputrace
    Gputrace(gputrace::Options),
}

#[derive(Debug, Parser)]
pub struct Options {
    /// Hosts to un the command on
    #[clap(long, required = true)]
    pub hosts: Vec<String>,

    /// Command to run on multiple hosts
    #[clap(subcommand)]
    pub cmd: Command,
}

pub fn run_batch(hosts: Vec<String>, cmd: Command) -> Result<()> {

    match cmd {
        Command::Gputrace(opts) => {
            let mut handles = vec![];
            for host in hosts {
                let mut host = host.clone();
                if !host.contains(":") {
                    host.push_str(format!(":{}", crate::DYNO_PORT).as_str());
                }
                let opts = opts.clone();
                let handle = std::thread::spawn(move || {
                    let client = TcpStream::connect(host).unwrap();
                    gputrace::run_gputrace_from_opts(client, opts)
                });
                handles.push(handle);
            }
            for handle in handles {
                handle.join().unwrap()?;
            }
        }
    }
    Ok(())
}