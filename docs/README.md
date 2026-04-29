# Additory Documentation - Quarto Book

This directory contains the Quarto Book documentation for the Additory library.

## Building the Documentation

### Prerequisites

- Quarto CLI installed ([https://quarto.org/docs/get-started/](https://quarto.org/docs/get-started/))
- Python 3.9+ with additory installed

### Build Commands

**Preview locally:**
```bash
quarto preview
```

**Render to HTML:**
```bash
quarto render
```

**Render to PDF:**
```bash
quarto render --to pdf
```

**Render to EPUB:**
```bash
quarto render --to epub
```

## Publishing to GitHub Pages

### Option 1: Using Quarto Publish

```bash
quarto publish gh-pages
```

### Option 2: Manual Deployment

1. Render the book:
```bash
quarto render
```

2. The output will be in `_book/` directory

3. Push `_book/` contents to the `gh-pages` branch of your repository

### GitHub Actions (Recommended)

Create `.github/workflows/quarto-publish.yml` in your repository:

```yaml
name: Publish Quarto Documentation

on:
  push:
    branches: [main]
  workflow_dispatch:

jobs:
  build-deploy:
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - name: Check out repository
        uses: actions/checkout@v4

      - name: Set up Quarto
        uses: quarto-dev/quarto-actions/setup@v2

      - name: Render and Publish
        uses: quarto-dev/quarto-actions/publish@v2
        with:
          target: gh-pages
          path: docs/examples/quarto_book
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

## Custom Domain Setup

1. Add a `CNAME` file to the `quarto_book` directory with your custom domain:
```
docs.yourdomain.com
```

2. Configure DNS:
   - Add a CNAME record pointing to `sekarkrishna.github.io`
   - Or add A records pointing to GitHub Pages IPs

3. Enable custom domain in GitHub repository settings

## Directory Structure

```
quarto_book/
├── _quarto.yml           # Main configuration
├── index.qmd             # Landing page
├── intro.qmd             # Introduction
├── installation.qmd      # Installation guide
├── references.qmd        # References and links
├── references.bib        # Bibliography
├── styles.css            # Custom styles (minimal)
├── to/                   # add.to() examples
├── transform/            # add.transform() examples
├── synthetic/            # add.synthetic() examples
├── scan/                 # add.scan() examples
├── lineage/              # Lineage tracking examples
└── guides/               # Guides and troubleshooting
```

## Editing on GitHub

All `.qmd` files can be edited directly on GitHub. The book will automatically rebuild when changes are pushed to the main branch (if GitHub Actions is configured).

## Local Development

1. Make changes to `.qmd` files
2. Preview locally: `quarto preview`
3. Commit and push changes
4. GitHub Actions will automatically rebuild and deploy

## Support

For issues with the documentation:
- Open an issue on [GitHub](https://github.com/sekarkrishna/additory/issues)
- Email: krishnamoorthy.sankaran@sekrad.org

## License

MIT License - Same as the Additory library
