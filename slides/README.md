# Workshop slides

These decks are written in [Marp](https://marp.app/) Markdown. There is one deck for each workshop
module, and each deck follows the corresponding concept pages and labs.

## Export one deck to PowerPoint

Install Node.js, then run Marp through `npx`:

```bash
npx @marp-team/marp-cli slides/module-00-orientation.md --pptx --output slides/out/module-00-orientation.pptx
```

## Export every deck

```bash
mkdir -p slides/out
for deck in slides/module-*.md; do
  name=$(basename "$deck" .md)
  npx @marp-team/marp-cli "$deck" --pptx --output "slides/out/$name.pptx"
done
```

Use `--pdf` instead of `--pptx` for PDF output. The generated files in `slides/out/` are local
artifacts and should not be committed.

The decks are deliberately self-contained: the labs remain the source of truth for code and
commands, while the slides provide the instructor's narrative, diagrams, decision points, and
checkpoints.
