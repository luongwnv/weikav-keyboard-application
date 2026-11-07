//! Minimal implementation of the NUT75's DriveAll protocol.
//! See docs/protocol.md for provenance and intentionally supported commands.
use anyhow::{Result, bail, ensure};
use std::time::{Duration, Instant};

pub const GET_SIDE: u8 = 0x1d;
pub const SET_SIDE: u8 = 0x2d;
pub const GET_INFO: u8 = 0x10;
pub const GET_SETTINGS: u8 = 0x11;
pub const GET_KEYS: u8 = 0x12;
pub const GET_LIGHT: u8 = 0x13;
pub const GET_FN: u8 = 0x16;
pub const GET_CUSTOM: u8 = 0x14;
pub const GET_MACRO: u8 = 0x15;
pub const SET_SETTINGS: u8 = 0x21;
pub const SET_KEYS: u8 = 0x22;
pub const SET_LIGHT: u8 = 0x23;
pub const SET_FN: u8 = 0x26;
pub const SET_CUSTOM: u8 = 0x24;
pub const SET_MACRO: u8 = 0x25;
const HEADER: usize = 8;

pub trait Transport {
    fn write(&self, packet: &[u8]) -> Result<()>;
    fn read(&self, packet: &mut [u8], timeout_ms: i32) -> Result<usize>;
}

impl Transport for hidapi::HidDevice {
    fn write(&self, packet: &[u8]) -> Result<()> {
        // hidapi requires the report ID even for an unnumbered report.
        let mut report = Vec::with_capacity(packet.len() + 1);
        report.push(0);
        report.extend_from_slice(packet);
        let written = hidapi::HidDevice::write(self, &report)?;
        ensure!(
            written == report.len(),
            "HID chỉ ghi được {written}/{} byte",
            report.len()
        );
        Ok(())
    }

    fn read(&self, packet: &mut [u8], timeout_ms: i32) -> Result<usize> {
        Ok(self.read_timeout(packet, timeout_ms)?)
    }
}

pub fn packet(
    command: u8,
    address: u16,
    size: u8,
    last: bool,
    payload: &[u8],
    report_size: usize,
) -> Result<Vec<u8>> {
    ensure!(
        [32, 64].contains(&report_size),
        "Kích thước báo cáo HID không được hỗ trợ"
    );
    ensure!(
        payload.len() <= report_size - HEADER && usize::from(size) <= report_size - HEADER,
        "Gói dữ liệu HID quá lớn"
    );
    let mut packet = vec![0; report_size];
    packet[0] = 0xaa;
    packet[1] = command;
    packet[2] = size;
    packet[3..5].copy_from_slice(&address.to_le_bytes());
    packet[6] = u8::from(last);
    packet[HEADER..HEADER + payload.len()].copy_from_slice(payload);
    Ok(packet)
}

pub struct Protocol<T> {
    pub transport: T,
    pub report_size: usize,
    pub timeout: Duration,
}

impl<T: Transport> Protocol<T> {
    /// The receiver's live-color packets may have no acknowledgement. The vendor
    /// sends them with a 10 ms wait and zero retries rather than dropping the session.
    pub fn stream_rgb565(&self, data: &[u8]) -> Result<()> {
        ensure!(
            self.report_size == 64 && data.len() == 162,
            "Invalid live lighting frame"
        );
        let capacity = self.report_size - HEADER;
        let mut incoming = [0u8; 1024];
        for (index, chunk) in data.chunks(capacity).enumerate() {
            let address = index * capacity;
            let request = packet(
                0x3c,
                address as u16,
                chunk.len() as u8,
                address + chunk.len() == data.len(),
                chunk,
                self.report_size,
            )?;
            for _ in 0..32 {
                if self.transport.read(&mut incoming, 0)? == 0 {
                    break;
                }
            }
            self.transport.write(&request)?;
            let _ = self.transport.read(&mut incoming, 10)?;
        }
        Ok(())
    }
    /// GIF frames use the vendor's packet-index header, not the normal address header.
    pub fn upload_animation(&self, data: &[u8], check_index: bool) -> Result<()> {
        ensure!(
            [32, 64].contains(&self.report_size) && !data.is_empty(),
            "Invalid animation report"
        );
        let capacity = self.report_size - HEADER;
        let count = data.len().div_ceil(capacity);
        ensure!(count <= u16::MAX as usize, "Animation too large");
        for (index, chunk) in data.chunks(capacity).enumerate() {
            let mut request = vec![0; self.report_size];
            request[0] = 0xaa;
            request[1] = 0x42;
            request[2..4].copy_from_slice(&(index as u16).to_be_bytes());
            request[4..6].copy_from_slice(&(count as u16).to_be_bytes());
            request[HEADER..HEADER + chunk.len()].copy_from_slice(chunk);
            let mut reply = [0u8; 1024];
            for _ in 0..32 {
                if self.transport.read(&mut reply, 0)? == 0 {
                    break;
                }
            }
            self.transport.write(&request)?;
            let deadline = Instant::now() + self.timeout;
            let mut acknowledged = false;
            while Instant::now() < deadline {
                let remaining = deadline
                    .saturating_duration_since(Instant::now())
                    .as_millis()
                    .max(1) as i32;
                let len = self.transport.read(&mut reply, remaining)?;
                if len == 0 {
                    break;
                }
                let data = if reply[0] == 0 && len > 1 {
                    &reply[1..len]
                } else {
                    &reply[..len]
                };
                if data.len() >= HEADER
                    && data[0] == 0x55
                    && data[1] == 0x42
                    && (!check_index || u16::from_be_bytes([data[2], data[3]]) as usize == index)
                {
                    acknowledged = true;
                    break;
                }
            }
            ensure!(
                acknowledged,
                "Animation transfer not acknowledged at packet {index}"
            );
        }
        Ok(())
    }
    pub fn transfer(&self, command: u8, size: usize, data: Option<&[u8]>) -> Result<Vec<u8>> {
        self.transfer_at(command, 0, size, data, true)
    }

    pub fn transfer_at(
        &self,
        command: u8,
        start: usize,
        size: usize,
        data: Option<&[u8]>,
        last: bool,
    ) -> Result<Vec<u8>> {
        self.exchange(command, start, size, data, last, false)
    }

    /// Vendor read queries carry LED IDs in the request and RGB data in the reply.
    pub fn query(&self, command: u8, data: &[u8]) -> Result<Vec<u8>> {
        self.exchange(command, 0, data.len(), Some(data), true, true)
    }
    fn exchange(
        &self,
        command: u8,
        start: usize,
        size: usize,
        data: Option<&[u8]>,
        last: bool,
        query: bool,
    ) -> Result<Vec<u8>> {
        let read_payload = data.is_none() || query;
        ensure!(
            size > 0 && start.checked_add(size).is_some_and(|end| end <= 65536),
            "Kích thước truyền dữ liệu không hợp lệ"
        );
        if let Some(data) = data {
            ensure!(data.len() == size, "Dữ liệu ghi thiếu byte");
        }
        let chunk_size = self.report_size - HEADER;
        let mut result = Vec::with_capacity(size);
        for address in (0..size).step_by(chunk_size) {
            let len = (size - address).min(chunk_size);
            let payload = data
                .map(|data| &data[address..address + len])
                .unwrap_or(&[]);
            let request = packet(
                command,
                (start + address) as u16,
                len as u8,
                last && address + len == size,
                payload,
                self.report_size,
            )?;
            let mut success = false;
            // Bounded, sequential exchanges: no HID work runs on the UI thread.
            for _attempt in 0..3 {
                let mut incoming = [0u8; 1024];
                // Discard late notifications / stale replies before the next request.
                for _ in 0..32 {
                    if self.transport.read(&mut incoming, 0)? == 0 {
                        break;
                    }
                }
                self.transport.write(&request)?;
                if std::env::var_os("NUT75_TRACE_HID").is_some() {
                    eprintln!("HID TX ({} byte): {:02X?}", request.len(), &request[..8]);
                }
                let deadline = Instant::now() + self.timeout;
                while Instant::now() < deadline {
                    let remaining = deadline
                        .saturating_duration_since(Instant::now())
                        .as_millis()
                        .max(1) as i32;
                    let received = self.transport.read(&mut incoming, remaining)?;
                    if received == 0 {
                        break;
                    }
                    if std::env::var_os("NUT75_TRACE_HID").is_some() {
                        eprintln!(
                            "HID RX ({received} byte): {:02X?}",
                            &incoming[..received.min(8)]
                        );
                    }
                    // Reports with ID 0 are normally returned without that byte.
                    let reply = if incoming[0] == 0 && received > 1 {
                        &incoming[1..received]
                    } else {
                        &incoming[..received]
                    };
                    if reply.len() < HEADER || reply[0] != 0x55 || reply[1] != command {
                        continue;
                    }
                    // Read replies must include all requested data. Write ACKs may be header-only.
                    if read_payload && reply.len() < HEADER + len {
                        continue;
                    }
                    if query && u16::from_le_bytes([reply[3], reply[4]]) != (start + address) as u16
                    {
                        continue;
                    }
                    if read_payload {
                        result.extend_from_slice(&reply[HEADER..HEADER + len]);
                    }
                    success = true;
                    break;
                }
                if success {
                    break;
                }
            }
            if !success {
                bail!(
                    "Bàn phím không trả lời lệnh 0x{command:02X} tại byte {address}. Kiểm tra cáp USB và đóng driver web."
                );
            }
        }
        Ok(result)
    }
}

/// Parse the output report size from the device's own HID descriptor.
/// Reject numbered reports instead of guessing a packet length.
pub fn output_report_size(descriptor: &[u8]) -> Result<usize> {
    let (mut size, mut count, mut report_id) = (0usize, 0usize, 0usize);
    let mut stack = Vec::new();
    let mut outputs = std::collections::BTreeMap::<usize, usize>::new();
    let mut i = 0;
    while i < descriptor.len() {
        let prefix = descriptor[i];
        i += 1;
        ensure!(prefix != 0xfe, "HID long item không được hỗ trợ");
        let length = match prefix & 3 {
            3 => 4,
            value => value as usize,
        };
        ensure!(i + length <= descriptor.len(), "HID descriptor bị cắt ngắn");
        let value = descriptor[i..i + length]
            .iter()
            .enumerate()
            .fold(0usize, |value, (bit, byte)| {
                value | (usize::from(*byte) << (bit * 8))
            });
        i += length;
        match prefix & 0xfc {
            0x74 => size = value,
            0x94 => count = value,
            0x84 => report_id = value,
            0xa4 => stack.push((size, count, report_id)),
            0xb4 => {
                (size, count, report_id) = stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("HID global stack không hợp lệ"))?;
            }
            0x90 => {
                *outputs.entry(report_id).or_default() += size
                    .checked_mul(count)
                    .ok_or_else(|| anyhow::anyhow!("HID report quá lớn"))?;
            }
            _ => {}
        }
    }
    let bits = outputs.get(&0).copied().unwrap_or(0);
    ensure!(
        bits.is_multiple_of(8) && [256, 512].contains(&bits),
        "Thiết bị không có output report 32/64 byte với ID 0"
    );
    Ok(bits / 8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct Mock {
        requests: RefCell<Vec<Vec<u8>>>,
        pending: RefCell<Option<Vec<u8>>>,
    }
    impl Transport for Mock {
        fn write(&self, data: &[u8]) -> Result<()> {
            self.requests.borrow_mut().push(data.to_vec());
            let mut reply = data.to_vec();
            reply[0] = 0x55;
            let address = u16::from_le_bytes([data[3], data[4]]) as usize;
            for (offset, byte) in reply[8..].iter_mut().enumerate() {
                *byte = ((address + offset) % 251) as u8;
            }
            *self.pending.borrow_mut() = Some(reply);
            Ok(())
        }
        fn read(&self, data: &mut [u8], _timeout_ms: i32) -> Result<usize> {
            if let Some(reply) = self.pending.borrow_mut().take() {
                data[..reply.len()].copy_from_slice(&reply);
                Ok(reply.len())
            } else {
                Ok(0)
            }
        }
    }

    #[test]
    fn official_packet_header_and_last_flag() {
        let p = packet(SET_KEYS, 0x1f8, 8, true, &[2, 0, 4, 0], 32).unwrap();
        assert_eq!(&p[..8], &[0xaa, 0x22, 8, 0xf8, 1, 0, 1, 0]);
        assert_eq!(&p[8..12], &[2, 0, 4, 0]);
        assert!(packet(SET_KEYS, 0, 25, true, &[], 32).is_err());
    }

    #[test]
    fn animation_uses_big_endian_packet_indices_and_total() {
        let p = Protocol {
            transport: Mock {
                requests: RefCell::default(),
                pending: RefCell::default(),
            },
            report_size: 64,
            timeout: Duration::from_millis(1),
        };
        let data = vec![0x31; 120];
        p.upload_animation(&data, true).unwrap();
        let requests = p.transport.requests.borrow();
        assert_eq!(requests.len(), 3);
        assert_eq!(&requests[0][..8], &[0xaa, 0x42, 0, 0, 0, 3, 0, 0]);
        assert_eq!(&requests[2][..8], &[0xaa, 0x42, 0, 2, 0, 3, 0, 0]);
        assert_eq!(&requests[2][8..16], &[0x31; 8]);
        assert!(requests[2][16..].iter().all(|b| *b == 0));
    }

    #[test]
    fn live_frames_send_three_packets_even_without_acknowledgements() {
        struct NoAck(RefCell<Vec<Vec<u8>>>);
        impl Transport for NoAck {
            fn write(&self, data: &[u8]) -> Result<()> {
                self.0.borrow_mut().push(data.to_vec());
                Ok(())
            }
            fn read(&self, _: &mut [u8], _: i32) -> Result<usize> {
                Ok(0)
            }
        }
        let p = Protocol {
            transport: NoAck(RefCell::default()),
            report_size: 64,
            timeout: Duration::from_millis(1),
        };
        p.stream_rgb565(&[0x55; 162]).unwrap();
        let packets = p.transport.0.borrow();
        assert_eq!(packets.len(), 3);
        assert_eq!(&packets[0][..8], &[0xaa, 0x3c, 56, 0, 0, 0, 0, 0]);
        assert_eq!(&packets[2][..8], &[0xaa, 0x3c, 50, 112, 0, 0, 1, 0]);
    }

    #[test]
    fn keymaps_are_chunked_correctly_for_both_descriptor_sizes() {
        for report_size in [32, 64] {
            let p = Protocol {
                transport: Mock {
                    requests: RefCell::default(),
                    pending: RefCell::default(),
                },
                report_size,
                timeout: Duration::from_millis(1),
            };
            let raw = p.transfer(GET_KEYS, 512, None).unwrap();
            assert_eq!(raw, (0..512).map(|i| (i % 251) as u8).collect::<Vec<_>>());
            let requests = p.transport.requests.borrow();
            assert_eq!(requests.len(), 512usize.div_ceil(report_size - 8));
            assert!(requests[..requests.len() - 1].iter().all(|p| p[6] == 0));
            assert_eq!(requests.last().unwrap()[6], 1);
        }
    }

    #[test]
    fn led_queries_send_ids_and_collect_all_reply_payloads() {
        let p = Protocol {
            transport: Mock {
                requests: RefCell::default(),
                pending: RefCell::default(),
            },
            report_size: 64,
            timeout: Duration::from_millis(1),
        };
        let ids: Vec<u8> = (0u16..81).flat_map(u16::to_le_bytes).collect();
        let result = p.query(0x3b, &ids).unwrap();
        assert_eq!(result.len(), ids.len());
        assert_eq!(
            result,
            (0..162).map(|i| (i % 251) as u8).collect::<Vec<_>>()
        );
        let requests = p.transport.requests.borrow();
        assert_eq!(&requests[0][8..64], &ids[..56]);
        assert_eq!(&requests[2][8..58], &ids[112..]);
    }

    #[test]
    fn macro_offsets_cross_512_and_only_final_block_commits() {
        for report_size in [32, 64] {
            let p = Protocol {
                transport: Mock {
                    requests: RefCell::default(),
                    pending: RefCell::default(),
                },
                report_size,
                timeout: Duration::from_millis(1),
            };
            p.transfer_at(SET_MACRO, 0, 400, Some(&[0; 400]), false)
                .unwrap();
            p.transfer_at(SET_MACRO, 400, 512, Some(&[1; 512]), true)
                .unwrap();
            let requests = p.transport.requests.borrow();
            assert_eq!(requests.iter().filter(|r| r[6] == 1).count(), 1);
            assert_eq!(requests.last().unwrap()[6], 1);
            let first_body = &requests[400usize.div_ceil(report_size - 8)];
            assert_eq!(u16::from_le_bytes([first_body[3], first_body[4]]), 400);
            let last = requests.last().unwrap();
            assert!(u16::from_le_bytes([last[3], last[4]]) > 512);
            assert!(p.transfer_at(GET_MACRO, 65530, 20, None, true).is_err());
        }
    }

    #[test]
    fn descriptor_parser_supports_push_pop_and_rejects_unknown_reports() {
        assert_eq!(
            output_report_size(&[0x75, 8, 0x95, 32, 0x91, 2]).unwrap(),
            32
        );
        assert_eq!(
            output_report_size(&[0x75, 8, 0xa4, 0x95, 64, 0x91, 2, 0xb4]).unwrap(),
            64
        );
        assert!(output_report_size(&[0x85, 1, 0x75, 8, 0x95, 32, 0x91, 2]).is_err());
        assert!(output_report_size(&[0x75]).is_err());
    }
}
