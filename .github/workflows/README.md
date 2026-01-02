# GitHub Actions Workflows

## flathub-stats.yml

Automatically generates and publishes Flathub statistics file.

### Schedule
- Runs weekly on Sundays at 2 AM UTC
- Can be triggered manually via GitHub Actions UI

### What it does
1. Builds and runs `flathub-stats` tool (~30-60 minutes)
2. Generates `res/flathub-stats.bitcode-v0-7` file
3. Publishes to GitHub Releases with tag `latest`
4. Users download this file at runtime (cached for 30 days)

### Manual Trigger
1. Go to Actions tab in GitHub
2. Select "Generate Flathub Stats" workflow
3. Click "Run workflow"

### Cost
Free for public repositories (unlimited minutes)
