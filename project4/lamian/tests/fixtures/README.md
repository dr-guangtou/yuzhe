# Test Fixtures

This folder stores small image fixtures used by CLI integration tests.

## Fixture Policy

- Keep fixture files small to avoid repository bloat and slow test runs.
- Prefer public-domain or clearly redistributable files.
- Record source URL and license context for every added file.
- Add at least one negative fixture for unsupported or malformed input cases.

## Files

- `2602.17205_1.png`
  - Source URL: https://arxiv.org/abs/2602.17205
  - DOI URL: https://doi.org/10.1126/science.ady9404
  - Note: verify redistribution rights before publishing this repository publicly.

- `500px-Elliptical_galaxy_IC_2006.jpg`
  - Source URL: https://en.wikipedia.org/wiki/Elliptical_galaxy
  - Intended license status: public domain (as provided by source context)

## Future Additions Checklist

- `webp` fixture (positive case)
- `gif` fixture (positive case; animated if feasible)
- `tiff` fixture (positive case)
- `bmp` fixture (positive case)
- `svg` fixture (positive case)
- `heic` or `avif` fixture (negative case until support is implemented)
- extension/content mismatch fixture (negative case)
