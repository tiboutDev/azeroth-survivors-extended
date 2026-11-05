"""Names for new releases; historical version strings remain readable."""
import re

REVISION=re.compile(r"([0-9]+\.[0-9]+\.[0-9]+)-r([1-9][0-9]*)\Z")

def default_version(game_version):
    return game_version+"-r1"

def release_title(version):
    match=REVISION.fullmatch(version)
    if match:
        return f"{match[1]} — AZSX Update {match[2]}"
    return "AZSX "+version
