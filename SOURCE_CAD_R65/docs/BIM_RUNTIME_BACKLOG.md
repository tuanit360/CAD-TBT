# BIM runtime reconciliation backlog

This checklist tracks TBT360_BIM identity/revision/QTO safety after the central post-command reconciler.

## Runtime rule now enforced after every command

The runtime pass runs after groups, constraints and associativity.

For every entity that already carries `TBT360_BIM`:
- `entity=` is normalized to the real handle.
- If non-BIM entity content changed and the command did not already advance revision, revision is advanced once.
- If the command already advanced revision, the runtime does not double-bump.
- Imported/copied/renumbered entities keep their scope/revision but get the real handle.
- The same rule applies to model space, paper-space layouts and block definitions.
- Deleted entities need no mutation: QTO source validation sees them as missing.
- Undo/Redo remains snapshot-based, so runtime changes are in the same undo step.

## Closed by the runtime common-cause fix

These no longer need per-command revision patches when they edit an entity in place:
- Properties, MatchProp, layer reassignment and rename/repoint changes.
- Text, dimension, multileader, hatch, spline, table and annotation edits.
- Image/underlay clipping or visual property edits.
- Viewport geometry/property edits.
- Associative dimension/hatch/array updates.
- Dynamic block reference property/grip edits.
- Region/curve/3D geometry edits that keep the same handle.
- Any future command that changes an existing TBT360_BIM entity through the normal command transaction.

## Still needs explicit source-scope policy

These create new entities and therefore runtime cannot infer which source BIM scope should be inherited:
- QDIM / DIMCONTINUE / DIMBASELINE / derived dimension creation from picked BIM geometry.
- CENTERMARK / CENTERLINE from source geometry.
- MLEADER/LEADER creation when explicitly attached to BIM geometry.
- DIVIDE / MEASURE when inserting points/blocks along a BIM source.
- TANCIRCLE / derived construction geometry if product policy says it should inherit source scope.
- BREAKLINE / WIPEOUT-from-polyline / REGION-from-boundary and similar derived geometry.
- TXT2MTXT when many source text objects merge into one output.
- BURST/EXPLODE variants where multiple source scopes can contribute to one result.
- GROUP copy does not create entities and needs no BIM identity work.

## Import/remap policy now centralized

Closed by runtime + adopt/remap normalization:
- XREF load/reload model objects and nested block objects.
- XBIND copies of dependent blocks.
- INSERT_FILE model and nested block entities.
- Clipboard/cross-drawing nested block definitions.
- Any drawing entering a Session is normalized once through DocState::new.
- Every post-command pass re-normalizes real handles for model/layout/block entities.
- A changed block definition automatically invalidates existing BIM INSERT references once, without double-bumping commands that already did it.

Still requires persistence/regression evidence rather than more command patches:
- DXF roundtrip.
- DWG roundtrip.
- WBLOCK -> INSERT roundtrip.
- System clipboard DXF roundtrip.
- XREF reload from a real external file.

## QTO-specific remaining work

- Nested block/array element source identity.
- XREF reload/remap source identity.
- Multi-source derived entity provenance.
- Measurement extraction for Region/Hatch/Dimension where desired.
- Block-definition changes with stable INSERT handles are now handled centrally by runtime revision invalidation.
- Auto-relink ambiguity hardening for imported/remapped files.
- Count remains existence/scope based, not coordinate based.

## BIM workspace remaining work

- Reapply native viewport/render filters after workspace load.
- Make snapshot load transactional: validate into temporary state, then commit.
- Regression for invalid snapshot leaving current state untouched.

## Integration/build remaining work

- Compare shared baseline files from feature branches #12-#21 against the unified branch.
- cargo metadata --locked.
- cargo check --workspace --locked.
- cargo test --workspace --locked.
- End-to-end Create -> Modify -> Undo -> Redo -> Save -> Reopen regression.
- DXF and DWG persistence gates for handle, TBT360_BIM XDATA, QTO source link and workspace.
