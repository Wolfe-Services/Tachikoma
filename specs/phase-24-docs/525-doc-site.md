# Spec 525: Documentation Site

## Overview
Static documentation website generation, hosting, and deployment infrastructure using modern documentation tooling.


## Acceptance Criteria
- [x] Implementation complete per spec

## Requirements

### Static Site Generator
- Hugo, Docusaurus, or MkDocs
- Markdown content support
- Custom theme/styling
- Plugin extensibility
- Fast build times

### Site Features
- Full-text search
- Version selector
- Dark/light mode
- Mobile responsive
- Print-friendly pages
- Syntax highlighting
- Copy code buttons
- Table of contents
- Edit on GitHub links

### Navigation
- Sidebar navigation
- Breadcrumbs
- Previous/next links
- Category grouping
- Search integration

### Hosting Infrastructure
- GitHub Pages or Netlify
- Custom domain: docs.tachikoma.dev
- SSL/TLS certificate
- CDN distribution
- Preview deployments for PRs

### Build Pipeline
```yaml
# .github/workflows/docs.yml
name: Deploy Docs
on:
  push:
    branches: [main]
    paths: ['docs/**']

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build docs
        run: make docs-build
      - name: Deploy
        uses: peaceiris/actions-gh-pages@v3
```

### Analytics
- Privacy-respecting analytics
- Page view tracking
- Search query analysis
- User journey tracking
- Performance metrics

### SEO Optimization
- Meta tags
- Open Graph tags
- Sitemap generation
- robots.txt
- Canonical URLs
- Structured data

### Accessibility
- WCAG 2.1 AA compliance
- Keyboard navigation
- Screen reader support
- Color contrast
- Alt text for images

### Performance
- Lighthouse score > 90
- Lazy loading images
- Minified assets
- Gzip compression
- Preloading critical assets

## Site Configuration
```yaml
# docusaurus.config.js equivalent
site:
  title: Tachikoma Documentation
  tagline: Autonomous development agents
  url: https://docs.tachikoma.dev

  navbar:
    - label: Guides
      to: /guides
    - label: Reference
      to: /reference
    - label: API
      to: /api

  footer:
    links:
      - title: Community
        items:
          - label: Discord
          - label: Twitter
```

## Dependencies
- Spec 511: Documentation Structure
- All Phase 24 documentation specs

## Verification
- [ ] Site builds successfully
- [ ] Search functional
- [ ] Mobile responsive
- [ ] Lighthouse score > 90
- [ ] Accessibility audit passes
