# Operating model

Where this lives, who owns it, and what people actually do with it. This is a
proposal for the pilot, not a description of anything currently in place.

The technical question this answers is small. The organisational one is not: a
monitor that nobody is accountable for reading is the same as no monitor.

## The three roles

| Role | Who | What they do |
|---|---|---|
| **Sponsor** | Product actuary who raised the original share-loss question | Agrees the tracked dimensions and their weights. Receives the report. Decides whether a finding becomes a case worth working. |
| **Custodian** | A named person in product or pricing | Keeps competitor values current and sourced. Reviews what the monitor raises. Owns the quality of the comparison. |
| **Operator** | Whoever runs the platform | Keeps the scheduled check running and the report published. Does not interpret it. |

The custodian is the role most likely to be skipped, and skipping it is how the
comparison quietly rots. A stale competitor value produces a confident, wrong
number, and nothing in the system can tell that from a correct one.

## What people receive

**The report.** One self-contained HTML page per run: what our position is, what
changed, the full comparison against every provider, and what the numbers do not
tell you. It opens in any browser with no install, so it can be emailed,
attached to a committee pack, or bookmarked. It prints cleanly.

**The daily check.** The scheduled run publishes the report to a fixed URL and
fails loudly when our position has moved against us by more than the agreed
threshold.

**A case.** When drift is material the monitor opens a case with its diagnosis
and evidence attached, and every approval gate unmet. That is the handover point
from the machine to the people.

## Getting sponsor agreement

Two things need signing off before the numbers mean anything, and neither is a
technical decision:

1. **The tracked dimensions.** Which eight or so things a retirement annuity is
   genuinely chosen on. The seeded set is a starting proposal, not a
   recommendation — effective annual cost at two balances, platform fee,
   Section 14 transfer turnaround, digital onboarding, fund range, minimum
   contribution, adviser fee options.
2. **The weights.** Which of those matter most. The weights decide which gaps
   look largest, so they decide what the system tells you to worry about. They
   are currently a guess and should be replaced by the sponsor's judgement.

Until both are agreed, the position score is a demonstration of a mechanism
rather than a measure of anything.

A useful way to run that conversation: put the current comparison in front of
the sponsor and ask where it is wrong. Disagreement about a specific number is
much easier to act on than agreement in principle.

## Where it runs

The pilot needs one thing the demonstration does not have: **a database that
persists between runs.** Every drift finding is a comparison against the
position recorded by the previous run, so without persistence each run
re-discovers the same thing and the exit code means nothing.

```bash
MARKETSHIELD_DATABASE_URL=sqlite:///var/lib/marketshield/marketshield.db \
  monitor --report position.md --html /var/www/position/index.html
```

Anywhere that can run a binary on a schedule and serve a static file is enough.
The published HTML is the product for most readers; the API and the browser UI
are for the people working a case.

SQLite is appropriate for a pilot at this size and is not a long-term answer —
see `ARCHITECTURE.md`.

## Keeping competitor values current

Values are entered through `POST /api/observations`, each with a source
classification and a source reference. Both are enforced: a value without usable
provenance is refused, and competitively sensitive information is refused
outright.

Recording a new value supersedes the old one rather than overwriting it, so the
change becomes a competitor move the monitor can find and the report can show
with its source. This is the mechanism that answers "when did they move, and how
do we know" — but it only works if someone is entering the values.

Document collection is not built. Until it is, this is a person reading a
published fee schedule and typing in a number.

## What would make this fail

- **No named custodian.** Values go stale, the comparison silently misleads.
- **Weights never agreed.** Findings get argued rather than acted on.
- **The report goes somewhere nobody looks.** A daily email nobody opens is
  worse than nothing, because it creates the belief that someone is watching.
- **The score gets quoted as a competitiveness rating.** It is a relative
  ranking on a chosen set of dimensions. The report says so; people will still
  do it.
- **A breach with no owner.** If a failing check has no route to a decision, the
  check will be switched off within a month.
