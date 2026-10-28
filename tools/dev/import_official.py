"""Pinned 0.125.0 private import; never imports saves/settings or modifies source assets."""
import argparse
from pathlib import Path
from native_runtime_import import NAMES,SHA256,inspect,import_runtime
if __name__=="__main__":
 p=argparse.ArgumentParser(description=__doc__);p.add_argument("--source",type=Path,required=True);p.add_argument("--receiver",type=Path,required=True);a=p.parse_args()
 import_runtime(a.source,a.receiver,inspect(a.source,SHA256,NAMES),"0.125.0")
