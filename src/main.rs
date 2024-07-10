use anyhow::{anyhow, Result};
use clap::Args;
use clap::Parser;
use clap::Subcommand;
use itertools::Itertools;
use midir::{MidiIO, MidiInput, MidiOutput};
use std::io::{BufRead, Write};
use std::time::Duration;

#[derive(Args, Clone)]
struct Options {
    /// Prefix hexadecimal output with timestamp provided by midir
    #[arg(long)]
    show_ts: bool,

    /// Prefix hexadecimal output with size of message printed
    #[arg(long)]
    show_size: bool,
}

#[derive(Clone, Subcommand)]
enum Command {
    /// List available MIDI devices
    Ls {},

    /// Decode Korg SysEx data
    /// Write to MIDI device
    W { out_port_name: String },

    /// Read from a MIDI device and print to stdio
    R {
        in_port_name: String,
        #[command(flatten)]
        options: Options,
    },

    /// Read & write from a pair of MIDI device ports
    Rw {
        in_port_name: String,
        out_port_name: String,
        #[command(flatten)]
        options: Options,
    },
}

#[derive(clap::Parser, Clone)]
struct Arguments {
    #[command(subcommand)]
    command: Command,
}

fn print(t: u64, data: &[u8], options: &Options) -> Result<()> {
    let mut out = std::io::stdout();
    if options.show_ts {
        out.write_all(t.to_string().as_bytes())?;
        out.write(&[b' '])?;
    }
    if options.show_size {
        out.write_all(data.len().to_string().as_bytes())?;
        out.write(&[b' '])?;
    }
    let s = data
        .into_iter()
        .map(|x| format!("{:02x}", x).to_string())
        .collect::<Vec<_>>()
        .join(" ");
    out.write(s.as_bytes())?;
    out.write(&[b'\n'])?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Arguments::parse();
    match args.command {
        Command::Ls {} => {
            let midi_in = MidiInput::new("rust client")?;
            for (i, port) in midi_in.ports().into_iter().enumerate() {
                let name = midi_in.port_name(&port)?;
                println!("input #{}: \"{}\"", (i + 1), name);
            }
            let midi_out = MidiOutput::new("cust client")?;
            for (i, port) in midi_out.ports().into_iter().enumerate() {
                let name = midi_out.port_name(&port)?;
                println!("output #{}: \"{}\"", (i + 1), name);
            }
        }
        Command::R {
            in_port_name,
            options,
        } => {
            let midi_in = MidiInput::new("rust client")?;
            let port = midi_in
                .ports()
                .into_iter()
                .find(|port| midi_in.port_name(port).ok().iter().contains(&in_port_name))
                .ok_or_else(|| anyhow!("port not found"))?;
            eprintln!("found port");
            let _conn = midi_in.connect(
                &port,
                "midir-read-input",
                move |t: u64, data: &[u8], _| {
                    print(t, data, &options).unwrap();
                },
                (),
            )?;
            loop {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
        Command::Rw {
            in_port_name,
            out_port_name,
            options,
        } => {
            let midi_in = MidiInput::new("cust client")?;
            let midi_out = MidiOutput::new("cust client")?;
            let in_port = midi_in
                .ports()
                .into_iter()
                .find(|port| midi_in.port_name(port).ok().iter().contains(&in_port_name))
                .ok_or_else(|| anyhow!("port '{}' not found", in_port_name))?;
            let out_port = midi_out
                .ports()
                .into_iter()
                .find(|port| {
                    midi_out
                        .port_name(port)
                        .ok()
                        .iter()
                        .contains(&out_port_name)
                })
                .ok_or_else(|| anyhow!("port '{}' not found", out_port_name))?;
            let cn = midi_in.connect(
                &in_port,
                "midir-read-input",
                move |t: u64, data: &[u8], _: &mut ()| {
                    print(t, data, &options).unwrap();
                },
                (),
            )?;
            let mut out_conn = midi_out.connect(&out_port, "midir-write-output")?;
            for line in std::io::stdin().lines() {
                let bytes: Vec<u8> = Result::from_iter(
                    line?
                        .split(' ')
                        .filter(|s| !s.is_empty())
                        .map(|x| u8::from_str_radix(x, 16)),
                )
                .map_err(|e| anyhow!("parse error: {}", e))?;
                out_conn.send(&bytes)?;
            }
            out_conn.close();
            loop {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
        Command::W { out_port_name } => {
            let midi_out = MidiOutput::new("cust client")?;
            let out_port = midi_out
                .ports()
                .into_iter()
                .find(|port| {
                    midi_out
                        .port_name(port)
                        .ok()
                        .iter()
                        .contains(&out_port_name)
                })
                .ok_or_else(|| anyhow!("port not found"))?;
            let mut out_conn = midi_out.connect(&out_port, "midir-write-output")?;
            for line in std::io::stdin().lines() {
                let bytes: Vec<u8> = Result::from_iter(
                    line?
                        .split(' ')
                        .filter(|s| !s.is_empty())
                        .map(|x| u8::from_str_radix(x, 16)),
                )
                .map_err(|e| anyhow!("parse error: {}", e))?;
                out_conn.send(&bytes)?;
            }
        }
    };
    Ok(())
}
