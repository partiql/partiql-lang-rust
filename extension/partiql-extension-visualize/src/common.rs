pub trait ToDotGraph<T> {
    fn to_graph(self, data: &T) -> String;
}

#[cfg(feature = "visualize-dot")]
pub(crate) const FG_COLOR: &'static str = "\"#839496\"";
