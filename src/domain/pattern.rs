//! Firmas y plantillas de bytes escritas en la notación del trainer original:
//!
//! * `8B` — un byte literal.
//! * `*`  — un byte comodín (solo en firmas).
//! * `s1` — captura de 2 bytes: en la firma se comporta como comodín y su
//!   valor se reutiliza en las plantillas (el trainer original solo captura la
//!   mitad baja de desplazamientos de structs).
//! * `$player` — símbolo de 8 bytes que se resuelve a una dirección absoluta
//!   (solo en plantillas).

use std::collections::HashMap;
use std::fmt;

const CAPTURE_LEN: usize = 2;
const SYMBOL_LEN: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternError {
    InvalidToken(String),
    WildcardInTemplate,
    SymbolInSignature(String),
    MissingCapture(String),
    MissingSymbol(String),
}

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidToken(token) => write!(f, "token inválido: {token}"),
            Self::WildcardInTemplate => write!(f, "una plantilla no puede contener comodines"),
            Self::SymbolInSignature(name) => write!(f, "una firma no puede contener el símbolo ${name}"),
            Self::MissingCapture(name) => write!(f, "falta la captura {name}"),
            Self::MissingSymbol(name) => write!(f, "falta el símbolo ${name}"),
        }
    }
}

impl std::error::Error for PatternError {}

/// Bytes capturados durante el escaneo, por nombre de captura.
pub type Captures = HashMap<String, Vec<u8>>;

/// Direcciones absolutas disponibles para las plantillas, por nombre de símbolo.
pub type Symbols = HashMap<String, u64>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Byte(u8),
    Wildcard,
    Capture(String),
    Symbol(String),
}

impl Token {
    fn parse(text: &str) -> Result<Self, PatternError> {
        if text == "*" {
            return Ok(Self::Wildcard);
        }
        if let Some(name) = text.strip_prefix('$') {
            return Ok(Self::Symbol(name.to_owned()));
        }
        if text.len() == 2 {
            if let Ok(byte) = u8::from_str_radix(text, 16) {
                return Ok(Self::Byte(byte));
            }
        }
        if is_capture_name(text) {
            return Ok(Self::Capture(text.to_owned()));
        }
        Err(PatternError::InvalidToken(text.to_owned()))
    }

    fn len(&self) -> usize {
        match self {
            Self::Byte(_) | Self::Wildcard => 1,
            Self::Capture(_) => CAPTURE_LEN,
            Self::Symbol(_) => SYMBOL_LEN,
        }
    }
}

fn is_capture_name(text: &str) -> bool {
    let mut chars = text.chars();
    chars.next() == Some('s') && chars.as_str().chars().all(|c| c.is_ascii_digit()) && text.len() > 1
}

fn tokenize(text: &str) -> Result<Vec<Token>, PatternError> {
    text.split_whitespace().map(Token::parse).collect()
}

/// Posición de una captura dentro de una firma.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureSlot {
    pub name: String,
    pub offset: usize,
    pub len: usize,
}

/// Patrón de búsqueda (AOB) con comodines y capturas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    text: String,
    mask: Vec<Option<u8>>,
    captures: Vec<CaptureSlot>,
}

impl Signature {
    pub fn parse(text: &str) -> Result<Self, PatternError> {
        let mut mask = Vec::new();
        let mut captures = Vec::new();
        for token in tokenize(text)? {
            match token {
                Token::Byte(byte) => mask.push(Some(byte)),
                Token::Wildcard => mask.push(None),
                Token::Capture(name) => {
                    captures.push(CaptureSlot { name, offset: mask.len(), len: CAPTURE_LEN });
                    mask.extend([None; CAPTURE_LEN]);
                }
                Token::Symbol(name) => return Err(PatternError::SymbolInSignature(name)),
            }
        }
        Ok(Self { text: text.to_owned(), mask, captures })
    }

    pub fn len(&self) -> usize {
        self.mask.len()
    }

    /// Byte esperado en `offset`, o `None` si es comodín/captura.
    pub fn byte_at(&self, offset: usize) -> Option<u8> {
        self.mask.get(offset).copied().flatten()
    }

    /// Primera aparición de la firma en `haystack`.
    pub fn find_in(&self, haystack: &[u8]) -> Option<usize> {
        if self.mask.is_empty() || haystack.len() < self.mask.len() {
            return None;
        }
        let last_start = haystack.len() - self.mask.len();
        (0..=last_start).find(|&start| self.matches_at(haystack, start))
    }

    fn matches_at(&self, haystack: &[u8], start: usize) -> bool {
        self.mask
            .iter()
            .zip(&haystack[start..])
            .all(|(expected, actual)| expected.is_none_or(|byte| byte == *actual))
    }

    /// Extrae los bytes capturados de una coincidencia.
    pub fn read_captures(&self, matched: &[u8]) -> Captures {
        self.captures
            .iter()
            .map(|slot| (slot.name.clone(), matched[slot.offset..slot.offset + slot.len].to_vec()))
            .collect()
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// Secuencia de bytes con huecos para capturas y símbolos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    tokens: Vec<Token>,
}

impl Template {
    pub fn parse(text: &str) -> Result<Self, PatternError> {
        let tokens = tokenize(text)?;
        if tokens.contains(&Token::Wildcard) {
            return Err(PatternError::WildcardInTemplate);
        }
        Ok(Self { tokens })
    }

    pub fn len(&self) -> usize {
        self.tokens.iter().map(Token::len).sum()
    }

    pub fn render(&self, captures: &Captures, symbols: &Symbols) -> Result<Vec<u8>, PatternError> {
        let mut bytes = Vec::with_capacity(self.len());
        for token in &self.tokens {
            match token {
                Token::Byte(byte) => bytes.push(*byte),
                Token::Capture(name) => {
                    let value = captures.get(name).ok_or_else(|| PatternError::MissingCapture(name.clone()))?;
                    bytes.extend_from_slice(value);
                }
                Token::Symbol(name) => {
                    let address = symbols.get(name).ok_or_else(|| PatternError::MissingSymbol(name.clone()))?;
                    bytes.extend_from_slice(&address.to_le_bytes());
                }
                Token::Wildcard => unreachable!("rechazado en Template::parse"),
            }
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_matches_wildcards_and_reads_captures() {
        let signature = Signature::parse("66 83 B8 s1 00 00 14 77 *").unwrap();
        let memory = [0x90, 0x66, 0x83, 0xB8, 0x34, 0x12, 0x00, 0x00, 0x14, 0x77, 0xAB];

        let start = signature.find_in(&memory).unwrap();

        assert_eq!(start, 1);
        assert_eq!(signature.read_captures(&memory[start..])["s1"], vec![0x34, 0x12]);
    }

    #[test]
    fn signature_not_found_returns_none() {
        let signature = Signature::parse("AA BB").unwrap();
        assert_eq!(signature.find_in(&[0xAA, 0xAA, 0xBC]), None);
    }

    #[test]
    fn template_substitutes_captures_and_symbols() {
        let template = Template::parse("48 A1 $player 66 C7 80 s1 00 00").unwrap();
        let captures = Captures::from([("s1".to_owned(), vec![0x34, 0x12])]);
        let symbols = Symbols::from([("player".to_owned(), 0x1122_3344_5566_7788)]);

        let bytes = template.render(&captures, &symbols).unwrap();

        assert_eq!(
            bytes,
            [0x48, 0xA1, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x66, 0xC7, 0x80, 0x34, 0x12, 0x00, 0x00]
        );
        assert_eq!(template.len(), bytes.len());
    }

    #[test]
    fn template_reports_missing_capture() {
        let template = Template::parse("89 86 s1 00 00").unwrap();
        assert_eq!(
            template.render(&Captures::new(), &Symbols::new()),
            Err(PatternError::MissingCapture("s1".to_owned()))
        );
    }

    #[test]
    fn invalid_tokens_are_rejected() {
        assert!(Signature::parse("GG").is_err());
        assert!(Signature::parse("$player").is_err());
        assert!(Template::parse("90 *").is_err());
    }
}
