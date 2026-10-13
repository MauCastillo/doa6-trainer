//! Codificación de los saltos x86-64 que enlazan el código del juego con las code caves.

const JMP_REL32: u8 = 0xE9;
const NOP: u8 = 0x90;
pub const REL32_JUMP_LEN: usize = 5;

/// `jmp rel32` desde `from` hasta `to`, rellenado con NOPs hasta `total_len` bytes.
/// Devuelve `None` si el destino está a más de ±2 GiB.
pub fn relative_jump(from: u64, to: u64, total_len: usize) -> Option<Vec<u8>> {
    if total_len < REL32_JUMP_LEN {
        return None;
    }
    let next_instruction = from.wrapping_add(REL32_JUMP_LEN as u64);
    let displacement = i32::try_from(to.wrapping_sub(next_instruction) as i64).ok()?;
    let mut bytes = Vec::with_capacity(total_len);
    bytes.push(JMP_REL32);
    bytes.extend_from_slice(&displacement.to_le_bytes());
    bytes.resize(total_len, NOP);
    Some(bytes)
}

/// `jmp qword [rip+0]` seguido de la dirección destino: salta a cualquier dirección.
pub fn absolute_jump(to: u64) -> [u8; 14] {
    let mut bytes = [0u8; 14];
    bytes[..6].copy_from_slice(&[0xFF, 0x25, 0, 0, 0, 0]);
    bytes[6..].copy_from_slice(&to.to_le_bytes());
    bytes
}

/// Dirección a la que apunta una instrucción con operando relativo a RIP.
pub fn rip_relative_target(instruction_address: u64, instruction_len: usize, displacement: i32) -> u64 {
    instruction_address
        .wrapping_add(instruction_len as u64)
        .wrapping_add_signed(i64::from(displacement))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_jump_forward_with_padding() {
        let bytes = relative_jump(0x1000, 0x2000, 7).unwrap();
        assert_eq!(bytes, [0xE9, 0xFB, 0x0F, 0x00, 0x00, 0x90, 0x90]);
    }

    #[test]
    fn relative_jump_backward() {
        let bytes = relative_jump(0x2000, 0x1000, 5).unwrap();
        assert_eq!(i32::from_le_bytes(bytes[1..5].try_into().unwrap()), -0x1005);
    }

    #[test]
    fn relative_jump_rejects_far_targets_and_short_sites() {
        assert_eq!(relative_jump(0x1000, 0x1_0000_1000, 5), None);
        assert_eq!(relative_jump(0x1000, 0x2000, 4), None);
    }

    #[test]
    fn absolute_jump_encodes_target() {
        let bytes = absolute_jump(0x7FF6_1234_5678);
        assert_eq!(&bytes[..6], &[0xFF, 0x25, 0, 0, 0, 0]);
        assert_eq!(u64::from_le_bytes(bytes[6..].try_into().unwrap()), 0x7FF6_1234_5678);
    }

    #[test]
    fn rip_relative_target_adds_displacement_after_instruction() {
        assert_eq!(rip_relative_target(0x1000, 7, 0x10), 0x1017);
        assert_eq!(rip_relative_target(0x1000, 7, -0x10), 0xFF7);
    }
}
