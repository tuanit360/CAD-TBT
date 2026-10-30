# CAD360 UNIFIED R67 — SINGLE APPLICATION RELEASE GATE

## Single application contract

This branch is the **only active release target** for CAD360 R67. All changes to CAD 2D, BIM/IFC, QTO and collaborative workspaces belong to this repository branch and the single binary `SOURCE_CAD_R65/apps/cadcraft` (`cadcraft.exe`). The directory name `SOURCE_CAD_R65` is historical; it is not a separately shipped R65 app.

The single native Windows entrypoint `apps/cadcraft/src/main.rs` provides CAD, Takeoff, PDF, Model3D/IFC and BIM Collaboration tabs. Do not distribute a standalone old CAD EXE, standalone QTO HTML or separate BIM executable as R67.

## One build, two delivery formats

`CAD360_VietNam_TBT_Setup.exe` — Inno Setup installer of the unified CAD360 executable.

`CAD360_VietNam_TBT_Portable_Win64.zip` — portable archive of the **same** compiled unified CAD360 executable.

Both must come from the same commit and the same release build. The Windows end user must not need Rust/Cargo to install or run either.

## Verification / completion criteria

- Six parallel CI lanes PASS, including the full Rust workspace tests.
- End-to-end CAD create/edit/undo/redo/save/reopen and DXF/DWG/QTO persistence regressions PASS.
- BIM/IFC/QTO tabs launch from the same native executable and real-file smoke checks PASS.
- Inno Setup installer artifact successfully generated and installed/tested on Windows.
- No separate legacy launcher or second app shipped.
- Only then mark **R67 MERGE COMPLETE** and distribute the installer.

## Current status

**NOT YET COMPLETE.** Code has been consolidated on this R67 integration branch and release workflow configured, but runtime/end-to-end validation and Windows release artifact are not yet proven PASS. Do not claim merged/ready or label existing CI failures as acceptable.
