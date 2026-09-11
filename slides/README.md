# Workshop slides

These decks are written in [Marp](https://marp.app/) Markdown. There is one deck for each workshop
module, and each deck follows the corresponding concept pages and labs.

## Prerequisites

Marp CLI needs Node.js plus Google Chrome, Microsoft Edge, Mozilla Firefox, or another compatible
Chromium-based browser to export PPTX, PDF, and image files. HTML export does not need a browser.

The repository's devcontainer and GitHub Codespaces configuration supplies Node.js LTS and Chromium.
Rebuild an existing container after pulling the configuration change. Outside the devcontainer,
install Node.js and one of the supported browsers for your operating system before exporting.

Verify the tools from the repository root:

```bash
node --version
chromium --version # use your installed browser's command outside the devcontainer
```

## Export one deck to PowerPoint

```bash
mkdir -p slides/out
npx @marp-team/marp-cli slides/module-00-orientation.md \
  --pptx \
  --output slides/out/module-00-orientation.pptx
```

## Export every deck

```bash
mkdir -p slides/out
for deck in slides/module-*.md; do
  name=$(basename "$deck" .md)
  npx @marp-team/marp-cli "$deck" --pptx --output "slides/out/$name.pptx"
done
```

Use `--pdf` instead of `--pptx` for PDF output.

## Browser detection

Marp normally finds the installed browser automatically. If it reports `No suitable browser found`,
pass the executable explicitly. In the devcontainer:

```bash
npx @marp-team/marp-cli slides/module-00-orientation.md \
  --browser-path "$(command -v chromium)" \
  --pptx \
  --output slides/out/module-00-orientation.pptx
```

Use the equivalent path to Chrome, Edge, or Firefox on another platform. Marp also supports
`--browser firefox` when you need to select Firefox rather than a Chromium-based browser.

Generated files under `slides/out/` are ignored local artifacts and should not be committed.

The decks are deliberately self-contained: the labs remain the source of truth for code and
commands, while the slides provide the instructor's narrative, diagrams, decision points, and
checkpoints.
