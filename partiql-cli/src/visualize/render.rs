use crate::visualize::ast_to_dot::{AstToDot, ToDotGraph};

use std::convert::AsRef;
use std::os::raw::c_char;
use std::slice;
use strum::AsRefStr;

use graphviz_sys as gv;
use partiql_ast::ast;

/// Convert an AST into JSON
#[inline]
pub fn to_json(ast: &Box<ast::Expr>) -> String {
    serde_json::to_string_pretty(&ast).expect("json print")
}

/// Graphviz output formats
#[derive(AsRefStr, Debug, Copy, Clone)]
#[strum(serialize_all = "lowercase")]
#[non_exhaustive]
pub enum GraphVizFormat {
    /// Pretty-print
    Canon,
    /// Pretty-print; internal alias for graphviz's `canon`
    /// #[strum(serialize = "cannon")]
    PrettyPrint,
    /// Attributed dot
    Dot,
    /// Extended dot
    XDot,
    /// Svg
    Svg,
    /// Png
    Png,
}

/// FFI to graphviz-sys to convert a dot-formatted graph into the specified format.
fn gv_render(format: GraphVizFormat, graph_str: String) -> Vec<u8> {
    let c_graph_str = std::ffi::CString::new(graph_str).expect("cstring new failed");
    let c_dot = std::ffi::CString::new("dot").expect("cstring new failed");
    let c_format = std::ffi::CString::new(format.as_ref()).expect("cstring new failed");

    unsafe {
        let gvc = gv::gvContext();
        // TODO gvParseArgs to pass 'theme' colors, etc?
        //    See section 4 of https://www.graphviz.org/pdf/libguide.pdf
        //    See `dot --help`
        let g = gv::agmemread(c_graph_str.as_ptr());

        gv::gvLayout(gvc, g, c_dot.as_ptr());

        let mut buffer_ptr: *mut std::os::raw::c_char = std::ptr::null_mut();
        let mut length = 0 as std::os::raw::c_uint;
        gv::gvRenderData(gvc, g, c_format.as_ptr(), &mut buffer_ptr, &mut length);
        let c_bytes = slice::from_raw_parts_mut(buffer_ptr, length as usize);

        let bytes = std::mem::transmute::<&mut [c_char], &[u8]>(c_bytes);
        let out = Vec::from(bytes);

        gv::gvFreeRenderData(buffer_ptr);
        gv::gvFreeLayout(gvc, g);
        gv::agclose(g);
        gv::gvFreeContext(gvc);

        out
    }
}

/// Convert an AST into a graphviz dot-formatted string
#[inline]
fn ast_to_dot(ast: &Box<ast::Expr>) -> String {
    AstToDot::default().to_graph(ast)
}

/// FFI to graphviz-sys to convert a dot-formatted graph into the specified text format.
#[inline]
fn render_to_string(format: GraphVizFormat, ast: &Box<ast::Expr>) -> String {
    String::from_utf8(gv_render(format, ast_to_dot(ast))).expect("valid utf8")
}

/// Convert an AST into an attributed dot graph.
#[inline]
pub fn to_dot_raw(ast: &Box<ast::Expr>) -> String {
    ast_to_dot(ast)
}

/// Convert an AST into an attributed dot graph.
#[inline]
pub fn to_dot(ast: &Box<ast::Expr>) -> String {
    render_to_string(GraphVizFormat::Dot, &ast)
}

/// Convert an AST into a pretty-printed dot graph.
#[inline]
pub fn to_pretty_dot(ast: &Box<ast::Expr>) -> String {
    render_to_string(GraphVizFormat::Canon, &ast)
}

/// Convert an AST into a graphviz svg.
#[inline]
pub fn to_svg(ast: &Box<ast::Expr>) -> String {
    render_to_string(GraphVizFormat::Svg, &ast)
}

/// Convert an AST into a graphviz svg and render it to png.
pub fn to_png(ast: &Box<ast::Expr>) -> Vec<u8> {
    let svg_data = to_svg(ast);

    let mut opt = usvg::Options::default();
    opt.fontdb.load_system_fonts();

    let rtree = usvg::Tree::from_data(svg_data.as_bytes(), &opt.to_ref()).unwrap();
    let pixmap_size = rtree.svg_node().size.to_screen_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(
        &rtree,
        usvg::FitTo::Original,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();
    pixmap.encode_png().expect("png encoding failed")
}

/// Convert an AST into a graphviz svg and render it to png, then display in the console.
pub fn display(ast: &Box<ast::Expr>) {
    let png = to_png(ast);

    let conf = viuer::Config {
        absolute_offset: false,
        transparent: true,
        use_sixel: false,
        ..Default::default()
    };

    let img = image::load_from_memory(&png).expect("png loading failed.");
    viuer::print(&img, &conf).expect("Image printing failed.");
}
