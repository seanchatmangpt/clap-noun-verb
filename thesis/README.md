# PhD Thesis: Runtime CLI Generation for Distributed AI Agents

## Building the Thesis

### Requirements
- `xetex` or `pdflatex`
- `biblatex` package
- TeX Live or MiKTeX

### Compile

```bash
cd thesis
pdflatex -interaction=nonstopmode main.tex
biber main
pdflatex -interaction=nonstopmode main.tex
pdflatex -interaction=nonstopmode main.tex
```

Or using `latexmk`:
```bash
latexmk -pdf main.tex
```

## Document Structure

- **main.tex** - Master thesis document
- **chapters/**
  - **01_introduction.tex** - Problem statement and thesis contributions
  - **02_background.tex** - Related work and historical context
  - **03_architecture.tex** - System design and architecture
  - **04_implementation.tex** - Implementation details and techniques
  - **05_benchmarks.tex** - Comprehensive performance characterization
  - **06_applications.tex** - Real-world applications and deployment patterns
  - **07_conclusion.tex** - Conclusions, limitations, and future work
- **references.bib** - Bibliography database

## Key Thesis Results

**Primary Claim**: A fully functional 64-command noun-verb CLI can be generated, built, and executed in **40.9 microseconds**.

### Derived Metrics
- CLI Generation Rate: 24,450 CLIs/second
- Command Execution Throughput: >1.6M commands/second
- SLO Compliance: 2,442× faster than 100ms target
- Scaling: Linear O(n²) with command count

## Citation

```bibtex
@phdthesis{thesis2024,
    title = {Runtime CLI Generation for Distributed AI Agents:
             Dynamic Noun-Verb Command Architecture in Model Context Protocol Systems},
    author = {MCP Agent Capabilities Research},
    school = {Your University},
    year = {2024}
}
```

## Related Work

- Source code: `https://github.com/seanchatmangpt/clap-noun-verb/src/agent_cli.rs`
- Benchmarks: `https://github.com/seanchatmangpt/clap-noun-verb/benches/agent_cli_*.rs`
- Performance results: `benches/AGENT_CLI_JTBD_RESULTS.md`

## License

This thesis and accompanying code are provided under MIT or Apache 2.0 licenses.
