//! `.bmap` block maps (upstream `bmap.{h,cpp}`, format of intel/bmap-tools):
//! which block ranges of an image hold data. Unmapped ranges are free
//! space the sparse encoder may skip as DONT_CARE.

/// "BMAP", the fastbootd `oem bmap-load` wire magic.
pub const BMAP_WIRE_MAGIC: u32 = 0x424D_4150;
/// Wire header size.
pub const BMAP_WIRE_HEADER_SIZE: usize = 16;
/// Wire size of one range.
pub const BMAP_WIRE_RANGE_SIZE: usize = 40;

/// Half-open block range `[begin, end)` with an optional SHA-256.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BlockRange {
    pub begin: u64,
    pub end: u64,
    /// All-zero unless `has_sha256`.
    pub sha256: [u8; 32],
    pub has_sha256: bool,
}

/// The XML could not be parsed (upstream's `errorMsg`).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct BmapError(pub String);

/// Sorted mapped ranges plus the document's counters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockMap {
    block_size: u64,
    block_count: u64,
    mapped_block_count: u64,
    has_checksums: bool,
    ranges: Vec<BlockRange>,
    cursor: usize,
}

/// `QString::toULongLong` on trimmed text: 0 when not a number.
fn number(text: &str) -> u64 {
    text.trim().parse().unwrap_or(0)
}

fn parse_sha256(hex_text: &str) -> Option<[u8; 32]> {
    let mut out = [0u8; 32];
    hex::decode_to_slice(hex_text, &mut out).ok()?;
    Some(out)
}

fn parse_range(node: roxmltree::Node<'_, '_>) -> BlockRange {
    let mut range = BlockRange::default();
    if let Some(sum) = node.attribute("chksum").and_then(parse_sha256) {
        range.sha256 = sum;
        range.has_sha256 = true;
    }
    let text: String = node.children().filter_map(|c| c.text()).collect();
    let mut parts = text.split('-');
    range.begin = number(parts.next().unwrap_or(""));
    // "begin-end" is inclusive; a single number is one block.
    range.end = parts.next().map_or(range.begin + 1, |e| number(e) + 1);
    range
}

impl BlockMap {
    /// Parses a bmap document (`parse`). An empty document yields an
    /// empty map, as Qt's premature-end tolerance does upstream.
    ///
    /// # Errors
    /// Invalid UTF-8 or malformed XML.
    pub fn parse(xml: &[u8]) -> Result<Self, BmapError> {
        let text = std::str::from_utf8(xml).map_err(|e| BmapError(e.to_string()))?;
        let mut map = Self {
            block_size: 4096,
            block_count: 0,
            mapped_block_count: 0,
            has_checksums: false,
            ranges: Vec::new(),
            cursor: 0,
        };
        if text.trim().is_empty() {
            return Ok(map);
        }
        let options = roxmltree::ParsingOptions {
            allow_dtd: true,
            ..roxmltree::ParsingOptions::default()
        };
        let doc = roxmltree::Document::parse_with_options(text, options)
            .map_err(|e| BmapError(e.to_string()))?;
        let mut in_block_map = false;
        for node in doc.descendants().filter(roxmltree::Node::is_element) {
            let own_text = || node.text().unwrap_or("");
            match node.tag_name().name() {
                "BlockSize" => map.block_size = number(own_text()),
                "BlocksCount" => map.block_count = number(own_text()),
                "MappedBlocksCount" => map.mapped_block_count = number(own_text()),
                "BlockMap" => in_block_map = true,
                "Range" if in_block_map => {
                    let range = parse_range(node);
                    map.has_checksums |= range.has_sha256;
                    map.ranges.push(range);
                }
                _ => {}
            }
        }
        map.ranges.sort_by_key(|r| r.begin);
        Ok(map)
    }

    /// The fastbootd `oem bmap-load` payload: a 16-byte header then one
    /// 40-byte inclusive range per entry, little-endian.
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf =
            Vec::with_capacity(BMAP_WIRE_HEADER_SIZE + self.ranges.len() * BMAP_WIRE_RANGE_SIZE);
        buf.extend_from_slice(&BMAP_WIRE_MAGIC.to_le_bytes());
        buf.extend_from_slice(&(self.block_size as u32).to_le_bytes());
        buf.extend_from_slice(&(self.ranges.len() as u32).to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        for r in &self.ranges {
            buf.extend_from_slice(&(r.begin as u32).to_le_bytes());
            buf.extend_from_slice(&(r.end.wrapping_sub(1) as u32).to_le_bytes());
            buf.extend_from_slice(&r.sha256);
        }
        buf
    }

    /// True when block `index` lies in a mapped range (binary search).
    pub fn is_mapped(&self, index: u64) -> bool {
        let after = self.ranges.partition_point(|r| r.begin <= index);
        after > 0 && index < self.ranges[after - 1].end
    }

    /// Cursor-based [`Self::is_mapped`] for ascending queries.
    pub fn is_mapped_sequential(&mut self, index: u64) -> bool {
        while self.cursor < self.ranges.len() && self.ranges[self.cursor].end <= index {
            self.cursor += 1;
        }
        self.ranges
            .get(self.cursor)
            .is_some_and(|r| index >= r.begin)
    }

    /// `<BlockSize>` (default 4096).
    pub fn block_size(&self) -> u64 {
        self.block_size
    }

    /// `<BlocksCount>`.
    pub fn block_count(&self) -> u64 {
        self.block_count
    }

    /// `<MappedBlocksCount>`.
    pub fn mapped_block_count(&self) -> u64 {
        self.mapped_block_count
    }

    /// Mapped ranges, sorted by start.
    pub fn ranges(&self) -> &[BlockRange] {
        &self.ranges
    }

    /// No mapped ranges.
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// At least one range carried a valid checksum.
    pub fn has_checksums(&self) -> bool {
        self.has_checksums
    }
}
