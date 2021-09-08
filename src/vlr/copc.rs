//! COPC VLR.

use byteorder::{LittleEndian, ReadBytesExt};
use reader::{PointIterator, PointReader, UncompressedPointReader};
use std::io::{Cursor, Read, Seek};
use Header;
use Result;

/// COPC VLR data.
///
/// https://copc.io/DRAFT-SPEC.html
#[derive(Clone, Copy, Default, Debug)]
pub struct CopcData {
    /// Number of voxels in each spatial dimension
    pub span: i64,
    /// File offset to the first hierarchy page
    pub root_hier_offset: u64,
    /// Size of the first hierarchy page in bytes
    pub root_hier_size: u64,
    /// File offset of the *data* of the LAZ VLR
    pub laz_vlr_offset: u64,
    /// Size of the *data* of the LAZ VLR.
    pub laz_vlr_size: u64,
    /// File offset of the *data* of the WKT VLR if it exists, 0 otherwise
    pub wkt_vlr_offset: u64,
    /// Size of the *data* of the WKT VLR if it exists, 0 otherwise
    pub wkt_vlr_size: u64,
    /// File offset of the *data* of the extra bytes VLR if it exists, 0 otherwise
    pub eb_vlr_offset: u64,
    /// Size of the *data* of the extra bytes VLR if it exists, 0 otherwise
    pub eb_vlr_size: u64,
    /// Reserved for future use. Must be 0.
    pub reserved: [u64; 11],
}

impl CopcData {
    /// Reads VLR data from a `Read`.
    pub fn read_from<R: Read>(mut read: R) -> Result<CopcData> {
        let mut data = CopcData::default();
        data.span = read.read_i64::<LittleEndian>()?;
        data.root_hier_offset = read.read_u64::<LittleEndian>()?;
        data.root_hier_size = read.read_u64::<LittleEndian>()?;
        data.laz_vlr_offset = read.read_u64::<LittleEndian>()?;
        data.laz_vlr_size = read.read_u64::<LittleEndian>()?;
        data.wkt_vlr_offset = read.read_u64::<LittleEndian>()?;
        data.wkt_vlr_size = read.read_u64::<LittleEndian>()?;
        data.eb_vlr_offset = read.read_u64::<LittleEndian>()?;
        data.eb_vlr_size = read.read_u64::<LittleEndian>()?;
        Ok(data)
    }
}

/// EPT hierarchy key
#[derive(Clone, Copy, Default, Debug)]
pub struct VoxelKey {
    /// Level
    ///
    /// A value < 0 indicates an invalid VoxelKey
    pub level: i32,
    /// x
    pub x: i32,
    /// y
    pub y: i32,
    /// z
    pub z: i32,
}

impl VoxelKey {
    /// Reads VoxelKey from a `Read`.
    pub fn read_from<R: Read>(read: &mut R) -> Result<VoxelKey> {
        let mut data = VoxelKey::default();
        data.level = read.read_i32::<LittleEndian>()?;
        data.x = read.read_i32::<LittleEndian>()?;
        data.y = read.read_i32::<LittleEndian>()?;
        data.z = read.read_i32::<LittleEndian>()?;
        Ok(data)
    }
}

/// Hierarchy entry
///
/// An entry corresponds to a single key/value pair in an EPT hierarchy, but contains additional information to allow direct access and decoding of the corresponding point data.
#[derive(Clone, Copy, Default, Debug)]
pub struct Entry {
    /// EPT key of the data to which this entry corresponds
    key: VoxelKey,

    /// Absolute offset to the data chunk, or absolute offset to a child hierarchy page
    /// if the pointCount is -1
    offset: u64,

    /// Size of the data chunk in bytes (compressed size) or size of the child hierarchy page if
    /// the pointCount is -1
    byte_size: i32,

    /// Number of points in the data chunk, or -1 if the information
    /// for this octree node and its descendants is contained in other hierarchy pages
    point_count: i32,
}

impl Entry {
    /// Reads hierarchy entry from a `Read`.
    pub fn read_from<R: Read>(read: &mut R) -> Result<Entry> {
        let mut data = Entry::default();
        data.key = VoxelKey::read_from(read)?;
        data.offset = read.read_u64::<LittleEndian>()?;
        data.byte_size = read.read_i32::<LittleEndian>()?;
        data.point_count = read.read_i32::<LittleEndian>()?;
        Ok(data)
    }
}

/// Hierarchy page
///
/// COPC stores hierarchy information to allow a reader to locate points that are in a particular octree node.
/// The hierarchy may be arranged in a tree of pages, but shall always consist of at least one hierarchy page.
#[derive(Clone, Debug)]
pub struct Page {
    /// Hierarchy page entries
    entries: Vec<Entry>,
}

impl Page {
    /// Reads hierarchy page from a `Read`.
    pub fn read_from<R: Read>(mut read: R, page_size: u64) -> Result<Page> {
        let num_entries = page_size as usize / 32;
        let mut entries = Vec::with_capacity(num_entries);
        for _ in 0..num_entries {
            let entry = Entry::read_from(&mut read)?;
            entries.push(entry)
        }
        Ok(Page { entries })
    }
}

/// Page reader
#[derive(Debug)]
pub struct PageReader<R: Read + Seek + Send + std::fmt::Debug> {
    pages: Vec<Page>,
    copc: CopcData,
    point_reader: UncompressedPointReader<R>,
}

impl<R: Read + Seek + Send + std::fmt::Debug> PageReader<R> {
    pub(crate) fn new(source: R, header: Header, copc: CopcData) -> Result<Self> {
        let root_page = Page::read_from(Cursor::new(&header.evlrs()[0].data), copc.root_hier_size)?;
        let point_reader = UncompressedPointReader {
            source,
            header,
            point_count: 0,
            offset_to_point_data: 0,
            last_point_idx: 0,
        };
        Ok(PageReader {
            pages: vec![root_page],
            copc,
            point_reader,
        })
    }
    /// LAS header
    pub fn header(&self) -> &Header {
        &self.point_reader.header
    }
    /// Returns an iterator over points.
    pub fn points(&mut self, level: i32, x: i32, y: i32, z: i32) -> PointIterator {
        let entry = self.page_entry(level, x, y, z);
        let offset = entry.offset;
        let point_count = entry.point_count as u64; // FIXME
        self.point_reader.offset_to_point_data = offset;
        self.point_reader.point_count = point_count;
        self.point_reader.seek(0).unwrap(); // FIXME
        PointIterator {
            point_reader: &mut self.point_reader,
        }
    }
    fn page_entry(&self, _level: i32, _x: i32, _y: i32, _z: i32) -> &Entry {
        &self.pages[0].entries[0] // TODO!
    }
}
