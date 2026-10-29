# TBT360 — Đo bóc 2D/3D + PDF, bản thao tác độc lập

**Chạy ngay:** Trên Windows, nhấp đôi `CHAY_DO_BOC.bat`; hoặc mở `index.html` trong Chrome/Edge. Không cần Rust, Cargo hay .NET để dùng bảng đo bóc độc lập này. Đây **không phải EXE TBT360 native**.

## Luồng A — giống workflow Autodesk Takeoff (không phải Autodesk code)

1. Mở bản vẽ mẫu, chọn hai điểm có khoảng cách thực trên bản vẽ; nhập mét để hiệu chỉnh. Không đo chính thức nếu thiếu tỷ lệ.
2. Chọn Chiều dài, Diện tích, Chu vi, Đếm; đánh dấu trực tiếp trên bản vẽ. Nhập nhóm công việc để tổng hợp khối lượng.
3. Dùng bảng QTO bên phải để kiểm tra, xác nhận hoặc loại hạng mục.
4. Dùng `OBJECT_VOLUME_MAP.csv` để nhập 52 thể tích tham chiếu từ BREP/STEP; kiểm tra đơn vị m³, nguồn và mã cấu kiện. **Chưa có hiển thị mô hình Solid 3D tương tác hoặc tính thể tích native**.
5. Xuất `Chi tiết CSV` và `Tổng hợp CSV`. Các nguồn PDF và 3D luôn tách biệt, **không tự cộng trùng**.

## Luồng B — giống workflow Bluebeam (không phải Bluebeam code)

1. Chuyển trang PDF sang ảnh PNG bằng lệnh:

   `python tools/pdf_tools.py render assets/ban_ve_mau.pdf --page 1 --out assets/trang_1.png --dpi 160`

2. Mở PNG trong ứng dụng. Bấm **Chọn 2 điểm chuẩn**, nhập độ dài thật (m), bấm 2 điểm trên hình.
3. Đo bằng chuột; kết thúc vùng, Undo/Redo; lưu project JSON, mở lại để kiểm tra; xuất PNG đánh dấu/CSV.
4. Để xuất phép đo thành **annotation trong PDF**, cần một dự án đã lưu có thêm `pdf_render_dpi: 160` tương ứng DPI khi chuyển ảnh và chưa thay đổi kích thước ảnh; chạy:

   `python tools/pdf_tools.py mark assets/ban_ve_mau.pdf TBT360_QTO_PROJECT.json --page 1 --out PDF_DA_DANH_DAU.pdf`

   Nếu không có dpi đúng, công cụ **từ chối** xuất PDF để tránh ghi dấu nhầm tọa độ.

## Đọc lượng từ DXF đúng đơn vị

`python tools/dxf_qto_units.py path/to/file.dxf --out dxF_result.csv`

Công cụ đọc `$INSUNITS` (m, mm, ...) và chuyển về m/m². File không có đơn vị sẽ bị chặn. Đường cong polyline sử dụng chia nhỏ xấp xỉ với dung sai; không coi là hình học nghiệm thu kỹ thuật.

## Giới hạn nghiệm thu

- HTML độc lập đã có thao tác chuột, bộ nhớ Undo/Redo, lưu/mở JSON, xuất CSV/PNG. Chưa nhúng vào EXE Rust.
- PDF nhiều trang hiện cần chuyển từng trang bằng công cụ Python; không có OCR/AI nhận dạng từ PDF scan.
- Không đo QTO trực tiếp từ IFC native, không có Civil 3D chạy thực tế, không có Solid Boolean/IFC round-trip bên trong ứng dụng.
- `OBJECT_VOLUME_MAP.csv` là dữ liệu 3D ngoài, không giả làm kết quả engine native.
- Cần người đo xác nhận hạng mục, tỷ lệ, trùng lặp và chiều dày/độ mở nếu dùng làm hồ sơ thanh toán.


## R127 — Hai luồng Forma / Bluebeam
- Chọn một dòng QTO ở bảng bên phải rồi sửa mã công tác, diễn giải, ghi chú và chọn **Lưu diễn giải và nhật ký**.
- Chọn trạng thái đã xác nhận, nhấn **Lưu trạng thái** để lưu event thay đổi.
- **Xuất nhật ký CSV** lưu truy vết ID, nguồn, trang, thay đổi trước/sau.
- Chức năng áp dụng cho cả PDF/2D và bảng thể tích 3D; không cộng m² với m³.
- Undo/Redo và JSON Save/Reopen giữ lịch sử mới.
- Đây vẫn là HTML workspace độc lập, không phải EXE Rust native, và chưa có QTO IFC runtime.
