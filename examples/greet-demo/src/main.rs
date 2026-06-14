// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// greet-demo binary — hand-written, STATIC entry point (zero ontology input, so
// not generated). It pulls in the rendered verb wrappers (which self-register via
// the #[verb] macro's linkme slice) and the example-scoped handlers behind the
// delegation seam, then hands control to the pack's dispatcher.

mod handlers;
mod verbs;

fn main() -> clap_noun_verb::Result<()> {
    clap_noun_verb::run()
}
