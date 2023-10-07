pub(crate) fn is_coordinates_in_china(longitude: f64, latitude: f64) -> bool {
    if (72.004..=137.8347).contains(&longitude) && (0.8293..=55.8271).contains(&latitude) {
        return true;
    }

    false
}

#[test]
fn test_is_coordinates_in_china() {
    assert!(is_coordinates_in_china(
        114.3087832333621,
        30.64590540363425
    ));

    assert!(!is_coordinates_in_china(
        114.3087832333621,
        60.64590540363425
    ));
}
