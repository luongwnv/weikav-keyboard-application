# Đối chiếu NUT75 web driver / native 0.6

Nguồn: cấu hình `JU`/NUT75, routes `customKeys`, `lighting`, `macro`, `advancedKeys`, `settings` trong [driver chính hãng](https://weikav.driveall.cn/custom-keys), khảo sát 2026-11-07. Các tuỳ chọn Hall effect/RT/DKS và LCD không thuộc cấu hình NUT75 này. NUT75 có đèn hai bên theo tài liệu phần cứng, nhưng cấu hình web không bật bảng side/light-box và receiver đang thử trả bản ghi toàn số 0.

| Chức năng NUT75 | Native 0.6 | Kiểm chứng |
|---|---|---|
| Phím thường/Fn/media/chuột/con lăn | Có | Đã đọc keymap trên macOS 2.4G; chưa thử ghi phần cứng |
| Chức năng nội bộ bàn phím | Nhóm chức năng thông dụng | Chưa thử từng lệnh trên phần cứng |
| RGB 1–19, 23–25; tắt đèn | Có, toàn bộ 24 lựa chọn kể cả custom, icon và demo | Đã đọc RGB; demo gần đúng |
| Màu đơn/RGB, sáng/tốc độ/hướng | Có | Người dùng đã bấm Apply RGB; sửa lỗi so sánh trường hướng không dùng, cần thử lại |
| Đèn hai bên | Có luồng đọc/ghi có kiểm tra hỗ trợ | USB và receiver firmware 1.40 trả toàn 0 nên chưa cho ghi; Fn+Backspace người dùng thử cũng chưa đổi |
| Đèn báo Caps Lock | Chưa xác minh điều khiển màu | Khác đèn nền phím Caps Lock; không giả làm LED từng phím |
| Tô LED từng phím | Có | Đã đọc 512 byte custom LED; chưa thử ghi |
| Ghi/sửa/lưu/gán macro, các chế độ lặp | Có | Đã đọc index macro; chưa thử ghi/chạy macro phần cứng |
| SOCD/MT/TGL/CB | Có trong bảng gán phím | Theo encoding của web; chưa thử trên phần cứng |
| Sleep/mức chống dội/polling/cờ hệ thống | Có | Đã đọc game mode 56 byte; chưa thử ghi |
| Hồ sơ local, sao chép, JSON | Có | Test validation và tương thích hồ sơ cũ |
| Bộ GIF dựng sẵn + nhập/biên tập GIF/animation | Một phần | Có bảy preset và nhập JSON; Heart đã được người dùng xác nhận chạy trên receiver 2.4G, chưa có trình biên tập |
| Đèn theo nhạc, Music DIY | Chưa có | Cần native audio capture và bộ phát RGB trực tiếp; web giới hạn một số chức năng theo Windows |
| AI lighting | Một phần | Có ba preset native, Rainbow đã được người dùng xác nhận chạy trên receiver 2.4G; chưa nhập được mã hiệu ứng của web |
| Kiểm tra/cập nhật firmware | Chưa có | Cần chuỗi xác thực firmware, model/revision và bootloader; không giả lập thành công |
| Factory reset | Chưa có | Là thao tác riêng, không thay bằng ghi hồ sơ rỗng |

Ứng dụng chưa đạt parity đầy đủ. Các mục còn thiếu được giữ ở đây để tiếp tục triển khai; chúng không được thay bằng nút giả hoặc thông báo thành công giả. Không thực thi JavaScript của hãng trong app và không nhúng web driver.

## Khảo sát hai tab lighting

Trong cấu hình NUT75, `isShowGifEffect` và `isEnableAILighting` đều bật. Tab GIF có bảy preset cho NUT75 và trình tạo/sửa chuỗi khung hình. Driver đọc giới hạn số khung hình từ `gifMaxFrames` (byte 24–25 của thông tin thiết bị), sau đó gửi header bốn byte chứa số khung và tốc độ, tiếp theo là RGB từng phím từng khung qua lệnh `SET_LED_DATA` (`0x42`). Các gói dùng header riêng chứa số thứ tự và tổng số gói, khác header ghi cấu hình thông thường. Cần xác nhận kích thước report và giới hạn trên cả USB lẫn receiver trước khi ghi.

Tab AI là tính năng riêng, không đồng nghĩa với đèn theo nhạc: driver có các preset canvas và chức năng nhập mã hiệu ứng. Cần cách biểu diễn hoặc môi trường chạy an toàn cho hiệu ứng nhập, cùng luồng gửi màu thời gian thực; không thể chỉ gắn nhãn AI lên hiệu ứng RGB có sẵn. Chưa xác nhận hành vi trên firmware 1.40 hoặc Windows.

Nguồn khảo sát: [driver WEIKAV](https://weikav.driveall.cn/lighting), các mô-đun lighting và protocol tải ngày 2026-11-07. Đây là phân tích mã client, chưa phải xác nhận ghi trên thiết bị thật.
