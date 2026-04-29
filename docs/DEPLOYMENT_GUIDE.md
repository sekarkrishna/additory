# Deployment Guide for Additory Quarto Documentation

This guide explains how to deploy the Additory Quarto Book documentation to GitHub Pages with custom domain support.

## Prerequisites

- GitHub repository: `https://github.com/sekarkrishna/additory`
- Quarto CLI installed locally (for testing)
- GitHub Pages enabled in repository settings

## Directory Structure in Repository

The Quarto book should be placed in your repository at:
```
additory/
├── docs/
│   └── examples/
│       └── quarto_book/
│           ├── _quarto.yml
│           ├── index.qmd
│           ├── intro.qmd
│           ├── installation.qmd
│           ├── references.qmd
│           ├── to/
│           ├── transform/
│           ├── synthetic/
│           ├── scan/
│           ├── lineage/
│           └── guides/
```

## Deployment Options

### Option 1: GitHub Actions (Recommended)

This is the easiest and most automated approach.

#### Step 1: Create GitHub Actions Workflow

Create `.github/workflows/quarto-publish.yml` in your repository root:

```yaml
name: Publish Quarto Documentation

on:
  push:
    branches: [main]
    paths:
      - 'docs/examples/quarto_book/**'
  workflow_dispatch:

permissions:
  contents: write
  pages: write
  id-token: write

jobs:
  build-deploy:
    runs-on: ubuntu-latest
    
    steps:
      - name: Check out repository
        uses: actions/checkout@v4

      - name: Set up Quarto
        uses: quarto-dev/quarto-actions/setup@v2
        with:
          version: 1.4.550

      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.11'

      - name: Install Python dependencies
        run: |
          python -m pip install --upgrade pip
          pip install additory polars pandas

      - name: Render Quarto Book
        run: |
          cd docs/examples/quarto_book
          quarto render

      - name: Publish to GitHub Pages
        uses: quarto-dev/quarto-actions/publish@v2
        with:
          target: gh-pages
          path: docs/examples/quarto_book
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

#### Step 2: Enable GitHub Pages

1. Go to repository Settings → Pages
2. Source: Deploy from a branch
3. Branch: `gh-pages` / `root`
4. Save

#### Step 3: Push Changes

```bash
git add .github/workflows/quarto-publish.yml
git add docs/examples/quarto_book/
git commit -m "Add Quarto documentation"
git push origin main
```

The documentation will automatically build and deploy!

### Option 2: Manual Deployment with Quarto CLI

#### Step 1: Render Locally

```bash
cd docs/examples/quarto_book
quarto render
```

#### Step 2: Publish to GitHub Pages

```bash
quarto publish gh-pages
```

This will:
1. Render the book
2. Create/update the `gh-pages` branch
3. Push the rendered content

### Option 3: Manual Git Deployment

#### Step 1: Render Locally

```bash
cd docs/examples/quarto_book
quarto render
```

#### Step 2: Copy to gh-pages Branch

```bash
# Create gh-pages branch if it doesn't exist
git checkout --orphan gh-pages
git rm -rf .

# Copy rendered content
cp -r docs/examples/quarto_book/_book/* .

# Commit and push
git add .
git commit -m "Deploy documentation"
git push origin gh-pages

# Switch back to main
git checkout main
```

## Custom Domain Setup

### Step 1: Add CNAME File

Create `docs/examples/quarto_book/CNAME` with your custom domain:

```
docs.additory.io
```

Or add it to `_quarto.yml`:

```yaml
website:
  site-url: "https://docs.additory.io"
```

### Step 2: Configure DNS

Add DNS records for your custom domain:

**Option A: CNAME Record (Recommended)**
```
Type: CNAME
Name: docs (or your subdomain)
Value: sekarkrishna.github.io
```

**Option B: A Records**
```
Type: A
Name: @ (or your subdomain)
Value: 185.199.108.153
Value: 185.199.109.153
Value: 185.199.110.153
Value: 185.199.111.153
```

### Step 3: Enable Custom Domain in GitHub

1. Go to repository Settings → Pages
2. Custom domain: Enter your domain (e.g., `docs.additory.io`)
3. Check "Enforce HTTPS"
4. Save

### Step 4: Wait for DNS Propagation

DNS changes can take 24-48 hours to propagate. Check status:

```bash
dig docs.additory.io
```

## Verification

After deployment, verify:

1. **GitHub Pages URL**: `https://sekarkrishna.github.io/additory/`
2. **Custom Domain** (if configured): `https://docs.additory.io/`

## Updating Documentation

### With GitHub Actions

Simply edit `.qmd` files and push:

```bash
git add docs/examples/quarto_book/
git commit -m "Update documentation"
git push origin main
```

GitHub Actions will automatically rebuild and deploy.

### Manual Update

```bash
cd docs/examples/quarto_book
quarto render
quarto publish gh-pages
```

## Editing on GitHub

All `.qmd` files can be edited directly on GitHub:

1. Navigate to the file on GitHub
2. Click the pencil icon (Edit)
3. Make changes
4. Commit changes

If GitHub Actions is configured, the site will automatically rebuild.

## Troubleshooting

### Build Fails

Check GitHub Actions logs:
1. Go to repository → Actions tab
2. Click on the failed workflow
3. Review error messages

Common issues:
- Missing Python dependencies
- Quarto version mismatch
- Invalid `.qmd` syntax

### Custom Domain Not Working

1. Verify DNS records: `dig your-domain.com`
2. Check GitHub Pages settings
3. Wait for DNS propagation (up to 48 hours)
4. Ensure CNAME file exists in gh-pages branch

### 404 Errors

1. Verify gh-pages branch exists
2. Check GitHub Pages source settings
3. Ensure `index.html` exists in root of gh-pages branch

## Local Preview

Test locally before deploying:

```bash
cd docs/examples/quarto_book
quarto preview
```

This opens a live preview at `http://localhost:4200`

## Additional Resources

- **Quarto Documentation**: [https://quarto.org/docs/publishing/github-pages.html](https://quarto.org/docs/publishing/github-pages.html)
- **GitHub Pages**: [https://docs.github.com/en/pages](https://docs.github.com/en/pages)
- **Custom Domains**: [https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site)

## Support

For deployment issues:
- GitHub Issues: [https://github.com/sekarkrishna/additory/issues](https://github.com/sekarkrishna/additory/issues)
- Email: krishnamoorthy.sankaran@sekrad.org
