pub trait ToDotGraph<T> {
    fn to_graph(self, data: &T) -> String;
}

pub(crate) const FG_COLOR: &'static str = "\"#839496\"";
