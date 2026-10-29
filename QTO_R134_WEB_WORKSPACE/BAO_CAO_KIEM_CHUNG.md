# Bằng chứng và giới hạn kiểm thử TBT360 QTO R121

| Phạm vi | Kết quả |
|---|---|
| Unit test Python của QTO R120 | 6/6 PASS (thực thi) |
| Dữ liệu Solid từ BREP/STEP | 52 thể tích, 296.98425 m³ (tham chiếu ngoại) |
| DXF 3 tầng, `$INSUNITS=6` (m) | Đọc và đo được 96 entity bằng script mới |
| Xuất PDF annotation từ JSON có DPI đã khai báo | PASS; 5 annotation trên 1 trang; đọc lại bằng PyMuPDF |
| Cú pháp JavaScript của ứng dụng QTO | PASS `node --check` |
| Mở ứng dụng HTML và click chuột bằng Playwright/Chromium | CHƯA ĐẠT: môi trường chặn `file://` và localhost (ERR_BLOCKED_BY_ADMINISTRATOR) |
| QTO tương tác bằng chuột trong EXE native TBT360 | CHƯA TÍCH HỢP |
| IFC native round-trip | CHƯA CÓ |
| OCR tự động PDF scan/ảnh | CHƯA CÓ |
| Undo/Redo và Save/Reopen native Rust | CHƯA CHẠY |

**Dữ liệu mô phỏng:** 2 phép đo ví dụ trong `DU_AN_DO_BOC_MAU.json` là dữ liệu hướng dẫn, không phải giá trị bóc từ thực địa; dự án chỉ tính chính xác khi người dùng tự hiệu chỉnh tỷ lệ và xác nhận từng phép đo.

**Đo bóc PDF:** không được đưa vào hồ sơ thanh toán trước khi kiểm tra kích thước, tỷ lệ, nguồn và loại trùng với khối lượng 3D.

**Kiến trúc bàn giao:** `TAKEOFF_WORKSPACE/index.html` chạy độc lập không cần Rust, có dữ liệu demo và mã nguồn mở dạng HTML/JS; chưa nhúng vào UI egui native. Source R120/R119/R114 vẫn được giữ trong gói lớn. Không sử dụng mã hay logo độc quyền Autodesk/Bluebeam.
