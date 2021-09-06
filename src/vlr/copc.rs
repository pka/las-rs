//! COPC VLR.

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;
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
