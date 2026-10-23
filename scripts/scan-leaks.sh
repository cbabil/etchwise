#!/usr/bin/env bash
# Blocks secrets, tokens, private keys and email addresses from commits. It
# scans the whole index (what the commit will contain) with two `git grep`
# passes. Allowed: RFC 2606 example domains (example.com/.org/.net and the
# .example TLD) as samples, and the exact strings listed in .secretsignore
# (one per line, # comments): look-alikes and third-party license credits.
set -euo pipefail

# --no-renames: a renamed file counts as new, so its name is checked too.
staged=$(git diff --cached --name-only --no-renames --diff-filter=ACMT)
[ -z "$staged" ] && exit 0

if grep -Eq '(^|/)\.env(\..*)?$|\.(pem|key|p12|pfx|credentials)$|(^|/)core\.json$' <<<"$staged"; then
  echo "pre-commit: blocked, a credential or runtime token file is staged" >&2
  exit 1
fi

secrets='(sk-ant-[A-Za-z0-9_-]{8,}|sk-[A-Za-z0-9]{20,}|AIza[0-9A-Za-z_-]{30,}|gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{20,}|glpat-[A-Za-z0-9_-]{20,}|npm_[A-Za-z0-9]{30,}|AKIA[0-9A-Z]{16}|xox[baprs]-[A-Za-z0-9-]{10,}|-----BEGIN [A-Z ]*PRIVATE KEY-----|"token"[[:space:]]*:[[:space:]]*"[0-9a-f]{64}"|[Bb]earer[[:space:]]+[0-9a-f]{64})'
emails='[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}'
example='@(example\.(com|org|net)|[A-Za-z0-9.-]+\.example)$'
allowed=$(grep -Ev '^[[:space:]]*(#|$)' .secretsignore 2>/dev/null || true)
# Only the dependency lockfiles are skipped: their hashes look like secrets.
skip=(':(exclude,glob)**/bun.lock' ':(exclude,glob)**/Cargo.lock')

status=0
if leaks=$(git grep --cached -IlE "$secrets" -- . "${skip[@]}"); then
  echo "pre-commit: blocked, secret or token pattern in:" >&2
  echo "$leaks" >&2
  status=1
fi
if found=$(git grep --cached -IoE "$emails" -- . "${skip[@]}" | grep -Ev "$example" |
  { if [ -n "$allowed" ]; then grep -vFf <(sed 's/^/:/; s/$//' <<<"$allowed"); else cat; fi; }); then
  echo "pre-commit: blocked, email address in:" >&2
  cut -d: -f1 <<<"$found" | sort -u >&2
  status=1
fi
exit $status
