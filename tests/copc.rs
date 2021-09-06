extern crate las;

use las::Read;

#[test]
#[ignore]
fn read_laszip() {
    let mut reader = las::Reader::from_path("tests/data/autzen.laz").expect("Cannot open reader");
    let points: Vec<las::Point> = reader.points().map(|r| r.unwrap()).collect();
    assert_eq!(points.len(), 106);

    let reader = las::Reader::from_path("tests/data/autzen-classified.copc.laz");
    assert_eq!(
        reader.unwrap_err().to_string(),
        "offset to the start of the evlrs is too small: 93170741"
    )
}
