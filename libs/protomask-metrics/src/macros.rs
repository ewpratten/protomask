
/// A short-hand way to access one of the metrics in `protomask_metrics::metrics`
#[macro_export]
macro_rules! metric {
    // Accept and name and multiple labels
    ($metric_name: ident, $($label_name: ident),+) => {
        protomask_metrics::metrics::$metric_name.with_label_values(&[$(protomask_metrics::metrics::label_values::$label_name),+])
    };

}
