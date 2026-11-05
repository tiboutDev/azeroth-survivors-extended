"""Check a release DLL against its own two-build report, never an older release."""
import argparse
import json
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"tools/build"))
from azsx_reproducibility import verify_artifact
from client_registry import selected_client

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate",type=Path,required=True)
    parser.add_argument("--dll",type=Path,required=True)
    args=parser.parse_args()
    verify_artifact(json.loads(args.candidate.read_text()),args.dll.read_bytes(),selected_client()[0])
    print("Release DLL matches its own verified two-build report.")
if __name__=="__main__":main()
