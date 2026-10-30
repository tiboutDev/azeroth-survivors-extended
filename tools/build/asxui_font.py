"""Generate ASXUI's renamed OFL number face; never reads or bundles native game fonts.
Run with uv run --with fonttools python tools/build/asxui_font.py --source <font> --output <font>.
"""
import argparse
from pathlib import Path
from fontTools.ttLib import TTFont
from fontTools.pens.recordingPen import DecomposingRecordingPen
from fontTools.pens.transformPen import TransformPen
from fontTools.pens.ttGlyphPen import TTGlyphPen

def build(source,output):
    font=TTFont(source,recalcTimestamp=False)
    glyphs=font.getGlyphSet();replacement={};scale=1/3
    for name in font.getGlyphOrder():
        recording=DecomposingRecordingPen(glyphs);glyphs[name].draw(recording)
        pen=TTGlyphPen(None);recording.replay(TransformPen(pen,(scale,0,0,scale,0,350*(1-scale))))
        replacement[name]=pen.glyph()
    for name,glyph in replacement.items():
        font["glyf"][name]=glyph
        advance,left=font["hmtx"][name];font["hmtx"][name]=(max(1,round(advance*scale)),round(left*scale))
    for tag in ("GPOS","GSUB","kern","fpgm","prep","cvt "):
        if tag in font:del font[tag]
    names={1:"ASXUI Combat",2:"SemiBold",3:"ASXUI Combat SemiBold 1",4:"ASXUI Combat SemiBold",6:"ASXUICombat-SemiBold",16:"ASXUI Combat",17:"SemiBold"}
    for record in font["name"].names:
        if record.nameID in names:record.string=names[record.nameID].encode(record.getEncoding())
    output=Path(output);output.parent.mkdir(parents=True,exist_ok=True);font.save(output)

if __name__=="__main__":
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument("--source",required=True,type=Path);parser.add_argument("--output",required=True,type=Path)
    args=parser.parse_args();build(args.source,args.output)
