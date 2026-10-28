"""Classify native Lua diagnostics without suppressing errors in the game."""


def assess(text):
    errors, known = [], []
    for line in text.splitlines():
        lower = line.lower()
        if "runtime error:" in lower and "unsupported script 'OnInputLanguageChanged'" in line and "FloatingChatFrame.xml" in line:
            known.append(line)
        elif any(token in lower for token in ("runtime error:", "has been blocked from an action", "lua error", "script error")):
            errors.append(line)
    return {"passed": not errors, "errors": errors, "known_base_warnings": known}
