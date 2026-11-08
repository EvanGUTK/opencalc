"""Local regression checks for notice collection and pinned upstream inputs."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
import sys
import struct

sys.dont_write_bytecode = True


def load_script(name: str):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


collector = load_script("collect-dependency-licenses")
fetcher = load_script("fetch-license-supplements")
pe_inspector = load_script("inspect-pe-imports")


class NoticeCollectionTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="opencalc-notice-test-")
        self.root = Path(self.temporary.name)
        self.vendor = self.root / "vendor"
        self.crate = self.vendor / "fixture"
        self.crate.mkdir(parents=True)
        self.lock = self.root / "Cargo.lock"
        self.lock.write_text('version = 4\n[[package]]\nname = "fixture"\nversion = "1.0.0"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\n', encoding="utf-8")
        (self.crate / "Cargo.toml").write_text('[package]\nname = "fixture"\nversion = "1.0.0"\nlicense = "MIT"\n', encoding="utf-8")
        self.output = self.root / "output"

    def tearDown(self):
        self.temporary.cleanup()

    def test_missing_notice_fails_before_creating_partial_output(self):
        with self.assertRaisesRegex(ValueError, "Missing upstream license"):
            collector.collect(self.lock, self.vendor, self.output, None)
        self.assertFalse(self.output.exists())

    def test_unchanged_notice_is_hashed_and_unrelated_source_is_not_copied(self):
        notice = b"Copyright fixture contributors\r\nLicense text stays unchanged.\r\n"
        (self.crate / "LICENSE-MIT").write_bytes(notice)
        (self.crate / "unrelated-private.txt").write_text("Do not include me", encoding="utf-8")
        result = collector.collect(self.lock, self.vendor, self.output, None)
        index = json.loads((self.output / "index.json").read_text(encoding="utf-8"))
        copied = index["packages"][0]["files"][0]
        self.assertEqual((result["packages"], result["notices"]), (1, 1))
        self.assertEqual((self.output / copied["path"]).read_bytes(), notice)
        self.assertEqual(copied["sha256"], hashlib.sha256(notice).hexdigest())
        self.assertFalse(list(self.output.rglob("unrelated-private.txt")))

    def supplement(self, path: str, digest: str):
        manifest = self.root / "supplements.json"
        manifest.write_text(json.dumps({"entries": [{"name": "fixture", "version": "1.0.0", "files": [{"path": path, "sha256": digest, "url": "https://raw.githubusercontent.com/example/project/" + "1" * 40 + "/LICENSE"}]}]}), encoding="utf-8")
        return manifest

    def test_supplement_hash_mismatch_fails_before_copy(self):
        (self.root / "upstream.txt").write_text("Actual notice", encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "hash mismatch"):
            collector.collect(self.lock, self.vendor, self.output, self.supplement("upstream.txt", "0" * 64))
        self.assertFalse(self.output.exists())

    def test_supplement_path_cannot_escape_reviewed_input_root(self):
        with self.assertRaises((ValueError, OSError)):
            collector.collect(self.lock, self.vendor, self.output, self.supplement("../outside.txt", "0" * 64))
        self.assertFalse(self.output.exists())

    def test_upstream_fetch_rejects_mutable_branch_before_network(self):
        manifest = self.supplement("notice.txt", "0" * 64)
        content = json.loads(manifest.read_text(encoding="utf-8"))
        content["entries"][0]["files"][0]["url"] = "https://raw.githubusercontent.com/example/project/main/LICENSE"
        manifest.write_text(json.dumps(content), encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "immutable"):
            fetcher.fetch(manifest, self.output)
        self.assertFalse(self.output.exists())

    def test_pe_probe_identifies_separate_vc_runtime_import(self):
        data = bytearray(1024)
        data[:2] = b"MZ"
        struct.pack_into("<I", data, 0x3C, 128)
        data[128:132] = b"PE\0\0"
        struct.pack_into("<HH", data, 132, 0x8664, 1)
        struct.pack_into("<H", data, 148, 240)
        struct.pack_into("<H", data, 152, 0x20B)
        struct.pack_into("<Q", data, 176, 0x140000000)
        struct.pack_into("<II", data, 272, 0x1000, 40)
        struct.pack_into("<IIII", data, 400, 512, 0x1000, 512, 512)
        struct.pack_into("<I", data, 524, 0x1080)
        data[640:656] = b"VCRUNTIME140.dll\0"
        executable = self.root / "fixture.exe"
        executable.write_bytes(data)
        report = pe_inspector.inspect(executable)
        self.assertTrue(report["separate_vc_runtime_required"])
        self.assertEqual(report["visual_cpp_redistributable_imports"], ["VCRUNTIME140.dll"])

    def test_pe_probe_rejects_truncated_untrusted_image(self):
        executable = self.root / "invalid.exe"
        executable.write_bytes(b"MZ")
        with self.assertRaisesRegex(ValueError, "Truncated"):
            pe_inspector.inspect(executable)


if __name__ == "__main__":
    unittest.main()
