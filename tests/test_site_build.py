"""Unit tests for hummbl-forge-engine static edge SPA builder and artifacts."""

import json
import unittest
from pathlib import Path


class TestSiteBuild(unittest.TestCase):
    def setUp(self):
        self.root = Path(__file__).resolve().parent.parent
        self.public_dir = self.root / "public"

    def test_public_html_exists_and_valid(self):
        index_file = self.public_dir / "index.html"
        self.assertTrue(index_file.exists(), "index.html must exist")
        content = index_file.read_text(encoding="utf-8")
        self.assertIn("<!DOCTYPE html>", content)
        self.assertIn("HUMMBL Forge Engine", content)
        self.assertIn('id="app"', content)
        self.assertIn('id="crates-container"', content)
        self.assertIn('id="targets-container"', content)
        self.assertNotIn("https://cdn.jsdelivr.net", content)
        self.assertNotIn("https://unpkg.com", content)

    def test_api_endpoints_valid_json(self):
        data_json = self.public_dir / "api" / "data.json"
        crates_json = self.public_dir / "api" / "crates.json"
        targets_json = self.public_dir / "api" / "targets.json"

        self.assertTrue(data_json.exists(), "data.json must exist")
        self.assertTrue(crates_json.exists(), "crates.json must exist")
        self.assertTrue(targets_json.exists(), "targets.json must exist")

        with open(data_json, "r", encoding="utf-8") as f:
            data = json.load(f)
            self.assertEqual(data.get("name"), "hummbl-forge-engine")
            self.assertIn("crates", data)
            self.assertIn("targets", data)

        with open(crates_json, "r", encoding="utf-8") as f:
            crates = json.load(f)
            self.assertIsInstance(crates, list)
            self.assertEqual(len(crates), 8)

        with open(targets_json, "r", encoding="utf-8") as f:
            targets = json.load(f)
            self.assertIsInstance(targets, list)
            self.assertGreater(len(targets), 0)

    def test_security_headers_and_manifest(self):
        headers_file = self.public_dir / "_headers"
        manifest_file = self.public_dir / "manifest.json"

        self.assertTrue(headers_file.exists(), "_headers must exist")
        self.assertTrue(manifest_file.exists(), "manifest.json must exist")

        headers_content = headers_file.read_text(encoding="utf-8")
        self.assertIn("Content-Security-Policy", headers_content)
        self.assertIn("X-Frame-Options: DENY", headers_content)


if __name__ == "__main__":
    unittest.main()
