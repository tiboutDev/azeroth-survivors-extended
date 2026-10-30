"""OFL face identity, physical glyph scale and deterministic generation."""
import importlib.util
from pathlib import Path
import tempfile
import unittest
try:
    from fontTools.ttLib import TTFont
except ImportError:
    TTFont=None
ROOT=Path(__file__).resolve().parents[1]
FONT=ROOT/"addons/ASXUI/Fonts/ASXUI-Numbers.ttf"

@unittest.skipUnless(TTFont,"Explicit ASXUI profile requires fonttools")
class FontTests(unittest.TestCase):
    def test_renamed_font_reduces_glyphs_without_changing_vertical_metrics(self):
        with TTFont(FONT) as font:
            self.assertEqual(font["head"].unitsPerEm,1000)
            self.assertEqual((font["hhea"].ascent,font["hhea"].descent),(1000,-200))
            zero=font.getBestCmap()[ord("0")]
            self.assertEqual(font["hmtx"][zero][0],150)
            glyph=font["glyf"][zero]
            self.assertTrue(239<=glyph.yMax-glyph.yMin<=242)
            families={n.toUnicode() for n in font["name"].names if n.nameID in (1,16)}
            self.assertEqual(families,{"ASXUI Combat"})
            for character in "0123456789+-.,kKmM":self.assertIn(ord(character),font.getBestCmap())
        self.assertIn("SIL OPEN FONT LICENSE",(ROOT/"addons/ASXUI/Fonts/OFL.txt").read_text())
    def test_generation_is_deterministic_for_identical_input(self):
        spec=importlib.util.spec_from_file_location("asxui_font_builder",ROOT/"tools/build/asxui_font.py")
        module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
        with tempfile.TemporaryDirectory() as folder:
            a,b=Path(folder)/"a.ttf",Path(folder)/"b.ttf"
            module.build(FONT,a);module.build(FONT,b)
            self.assertEqual(a.read_bytes(),b.read_bytes())

if __name__=="__main__":unittest.main()
