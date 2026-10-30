# CAD360 R67 — Release Readiness / Danh sách chốt trước đóng gói

## Phạm vi thống nhất
**Một** ứng dụng desktop Rust: `SOURCE_CAD_R65/apps/cadcraft/src/main.rs` -> `cadcraft.exe`, có các tab CAD 2D, QTO, PDF, 3D/IFC và BIM cộng tác. Tên thư mục R65 là lịch sử; không phát hành R65/R66 riêng.

## P0 — bắt buộc PASS
- [ ] `cargo check --workspace --locked`.
- [ ] `cargo test --workspace --locked --no-fail-fast`.
- [ ] XREF/Clipboard/DXF/DWG real-file tests giữ TBT360_BIM scope, handle remap và revision.
- [ ] Undo/Redo Mark/Back/Begin/End/Control tests PASS; không sửa expectation để che runtime bug.
- [ ] CAD create/modify/undo/redo/save/reopen nghiệm thu với một file thật.
- [ ] QTO nested references và source-link sau save/reopen được xác nhận.
- [ ] One-binary feature gate: CAD / Takeoff / PDF / IFC / BIM Collaboration trong main.rs.
- [ ] Windows x64 release artifacts tạo từ **cùng một commit** và `cadcraft.exe`.
- [ ] `CAD360_VietNam_TBT_Setup.exe` kiểm tra cài đặt, chạy GUI, shortcut và uninstall trên Windows sạch.
- [ ] Portable ZIP kiểm tra chạy GUI trên Windows sạch; SHA-256 khớp.

## P1 — nghiệm thu tính năng
- [ ] BIM viewport reopen visual regression.
- [ ] IFC clash với file IFC thật và đối chiếu kết quả.
- [ ] BIM federated/shared coordinates trên nhiều model và level/elevation.
- [ ] Reconnect/offline conflict, workset publish và phân quyền đa người.
- [ ] QTO pick nested member, relink thủ công và provenance đa nguồn.
- [ ] TBT.Auth/licensing hoạt động end-to-end nếu tính năng yêu cầu máy chủ.

## P2 — nâng cấp sau khi có bản cài ổn định
- Point-cloud format adapters, polygon XCLIP chuẩn, 3D solids/B-Rep, hidden line, PBR, Dynamic Block và AI classification.
- Regression hiệu năng với DXF/DWG/IFC lớn, memory profiling và crash telemetry.
- Code signing, cập nhật tự động có kiểm soát, release notes và version resource.

## Tình trạng xác nhận
**R67 NOT COMPLETE.** CI sáu luồng đã cấu hình; core PASS ở một số lượt; các lane còn lại và installer chưa đạt full PASS. Chỉ công bố hoàn tất khi có bằng chứng green CI và Windows installer smoke test.
