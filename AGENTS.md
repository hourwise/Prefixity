# Prefixity — Agent Instructions

## Start here

For substantial work:

1. Read `docs/INDEX.md`.
2. Read `docs/tasks/ACTIVE.md`.
3. Read only the research, source-of-truth documents, plans, decisions,
   benchmarks, and code relevant to the active task.
4. Inspect the existing implementation before making changes.

Do not recursively read all repository documentation unless the task requires it.

## Project principles

- Treat performance and token/cache claims as hypotheses until supported by evidence.
- Distinguish measured results from estimates, assumptions, and proposed behaviour.
- Existing accepted decisions and source-of-truth documents override assumptions.
- Preserve reproducible benchmark evidence.
- Do not alter benchmarks or evaluation criteria merely to improve reported results.
- Prefer simple measurable implementations over premature architecture.
- Do not silently expand task scope.
- Do not begin a later phase after completing the active task.
- Do not introduce dependencies without a clear need.
- Never expose credentials, API keys, private prompts, or user data.

## Completion

Before finishing:

1. Verify the acceptance criteria in `docs/tasks/ACTIVE.md`.
2. Run relevant tests, checks, and benchmarks.
3. Record evidence separately from interpretation.
4. Update the active task with work completed, validation performed,
   remaining issues, and important deviations.
5. Do not start another task.