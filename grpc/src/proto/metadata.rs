use base64;

use bytes::Bytes;
use chars::Chars;

use httpbis::Header;
use httpbis::Headers;

#[derive(Debug, Clone)]
pub struct MetadataKey {
    pub name: Chars,
}

impl MetadataKey {
    pub fn from<S: Into<Chars>>(s: S) -> MetadataKey {
        let chars = s.into();

        // TODO: assert ASCII
        assert!(!chars.is_empty());

        MetadataKey { name: chars }
    }

    pub fn is_bin(&self) -> bool {
        self.name.ends_with("-bin")
    }

    pub fn into_chars(self) -> Chars {
        self.name
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct MetadataEntry {
    pub key: MetadataKey,
    pub value: Bytes,
}

#[derive(Debug)]
pub enum MetadataDecodeError {
    Base64(base64::DecodeError),
}

impl From<base64::DecodeError> for MetadataDecodeError {
    fn from(decode_error: base64::DecodeError) -> Self {
        MetadataDecodeError::Base64(decode_error)
    }
}

impl MetadataEntry {
    fn into_header(self) -> Header {
        let is_bin = self.key.is_bin();

        let value = match is_bin {
            true => Bytes::from(base64::encode(&self.value)),
            false => self.value,
        };

        Header::new(self.key.name.into_inner(), value)
    }

    fn from_header(header: Header) -> Result<Option<MetadataEntry>, MetadataDecodeError> {
        if header.name().starts_with(b":") {
            return Ok(None);
        }
        if header.name().starts_with(b"grpc-") {
            return Ok(None);
        }
        let key = MetadataKey {
            name: Chars::try_from(header.name).expect("utf-8"),
        };
        let value = match key.is_bin() {
            true => Bytes::from(base64::decode(&header.value)?),
            false => header.value,
        };
        Ok(Some(MetadataEntry {
            key: key,
            value: value,
        }))
    }
}

#[derive(Default, Debug, Clone)]
pub struct Metadata {
    pub entries: Vec<MetadataEntry>,
}

impl Metadata {
    pub fn new() -> Metadata {
        Default::default()
    }

    pub fn from_headers(headers: Headers) -> Result<Metadata, MetadataDecodeError> {
        let mut r = Metadata::new();
        for h in headers.0 {
            if let Some(e) = MetadataEntry::from_header(h)? {
                r.entries.push(e);
            }
        }
        Ok(r)
    }

    pub fn into_headers(self) -> Headers {
        Headers(
            self.entries
                .into_iter()
                .map(MetadataEntry::into_header)
                .collect(),
        )
    }

    // Get metadata by key
    pub fn get<'a>(&'a self, name: &str) -> Option<&'a [u8]> {
        for e in &self.entries {
            if e.key.as_str() == name {
                return Some(&e.value[..]);
            }
        }
        None
    }

    pub fn extend(&mut self, extend: Metadata) {
        self.entries.extend(extend.entries);
    }

    pub fn add(&mut self, key: MetadataKey, value: Bytes) {
        self.entries.push(MetadataEntry {
            key: key,
            value: value,
        });
    }
}
