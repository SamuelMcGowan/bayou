pub mod keyvec;
pub mod peek;

macro_rules! assert_yaml_snapshot_with_source {
    ($test_name:expr; $source:expr => $output:expr) => {{
        insta::with_settings!({
            info => &$source,
            omit_expression => true,
        }, {
            insta::assert_yaml_snapshot!($test_name, $output);
        })
    }};
}
pub(crate) use assert_yaml_snapshot_with_source;
