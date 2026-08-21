# Archived: legacy verb-wrapper ggen pipeline

This `ggen.toml` + `ontology/` predate the pack/gate/frontmatter-template
ggen convention this repo's own generated artifacts now use (see the repo
root's current `ggen.toml`, composing
`~/ggen-marketplace/packs/clap-noun-verb-autonomic-pack` for
`src/autonomic.rs`). Moved here, not deleted, because:

- It targeted a different generation scheme (`[generation.rules]` with
  `query`/`template`/`output_file`/`mode` fields), incompatible with the
  `[packs]`/frontmatter-template convention every other ggen-generated
  artifact in this repo and `~/ggen-marketplace` now uses.
- It referenced a CI workflow, `.github/workflows/ggen-authority.yml`,
  that does not exist anywhere in this repository.
- Its declared output directory, `src/verbs/`, never existed either.
- Its `ontology/` directory contains content with no connection to
  clap-noun-verb itself (Hebrew Bible morphology/reference files:
  `oshb-morphology-source.ttl`, `oshb-reference.ttl`,
  `nehemiah-operating-grammar.ttl`, `gospel-passage-pattern.ttl`).

Together this is strong evidence the pipeline was leftover from an
earlier, abandoned architecture pass rather than active infrastructure --
archived per this repo's own documentation convention (dead docs move to
`docs/archive/` with a note on why they were superseded; this is the
source-generation-config equivalent), not deleted, so the history and
content remain fully recoverable.
