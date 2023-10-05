pub(crate) fn is_coordinate_out_of_china(longitude: f64, latitude: f64) -> bool {
    if !(72.004..=137.8347).contains(&longitude) || !(0.8293..=55.8271).contains(&latitude) {
        return true;
    }

    false
}
