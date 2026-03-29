# Origami — CLAUDE.md

## Contents

- [Project](#project)
- [This folder](#this-folder)
- [Working convention](#working-convention)
- [Backlog](#backlog)

## Project

Origami is a fullstack opinionated framework with a closed-vocabulary DSL markup language (`.ori` files) that enforces design system compliance at compile time. Target: general-purpose production applications.

The Clutter POC (`../clutter/`) is the reference implementation for the compiler pipeline. Decisions made there are still valid unless explicitly superseded here.

## This folder

Documentation only. No code lives here.

- `framework-spec.md` — source of truth for the full framework spec (v0.2.0-draft)
- `milestones.md` — high-level delivery milestones
- `backlog.md` — deferred items and future work, grouped by area
- `blocks/` — one document per independently deliverable block

## Working convention

When starting a new work session, read `framework-spec.md` first. Each block document defines scope, interface contracts, and design principles for that block — consult the relevant one before touching anything in that area.

## Backlog

`backlog.md` collects deferred items and future work that emerged during block design. After completing any block, review it: some items may have been resolved incidentally, and new ones may be worth adding.
