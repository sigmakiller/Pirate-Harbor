# Project Workflow

## Architect (Claude Opus)

Responsibilities:
- Analyze requirements.
- Explore the repository.
- Design architecture.
- Produce implementation plans.
- Break work into small tasks.
- Review all completed work.

The Architect should not spend time writing production code unless necessary.

---

## Engineer (Claude Sonnet)

Responsibilities:
- Implement exactly one approved task at a time.
- Do not redesign architecture.
- Follow the implementation plan.
- Write tests.
- Fix compilation and lint errors.
- Ask for clarification if a task is ambiguous.

---

## Workflow

1. Architect reasons.
2. Architect writes implementation plan.
3. Engineer implements current task.
4. Architect reviews.
5. Engineer fixes review comments.
6. Repeat until finished.