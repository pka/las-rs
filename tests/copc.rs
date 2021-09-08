extern crate las;

#[test]
#[ignore]
fn read_laz_header() {
    let reader = las::Reader::from_path("tests/data/autzen-classified.copc.laz");
    assert_eq!(
        reader.unwrap_err().to_string(),
        "offset to the start of the evlrs is too small: 93170741"
    );
}

#[test]
#[ignore]
fn read_copc_header() {
    let mut reader = las::Reader::copc_from_path("tests/data/autzen-classified.copc.laz")
        .expect("Cannot open reader");
    let header = reader.header();
    let vlrs: Vec<&str> = header
        .vlrs()
        .iter()
        .map(|vlr| vlr.user_id.as_str())
        .collect();
    assert_eq!(
        vlrs,
        ["entwine", "laszip encoded", "LASF_Projection", "LASF_Spec"]
    );

    let evlrs: Vec<&str> = header
        .evlrs()
        .iter()
        .map(|vlr| vlr.user_id.as_str())
        .collect();
    assert_eq!(evlrs, ["entwine"]);

    let points: Vec<las::Point> = reader.points(0, 0, 0, 0).map(|r| r.unwrap()).collect();
    assert_eq!(points.len(), 61074);
}
