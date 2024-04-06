pub mod keyvec;
pub mod peek;

macro_rules! assert_yaml_snapshot_with_source {
    ($source:expr => $output:expr) => {{
        insta::with_settings!({
            info => &$source,
            omit_expression => true,
        }, {
            insta::assert_yaml_snapshot!($output);
        })
    }};
}
pub(crate) use assert_yaml_snapshot_with_source;
