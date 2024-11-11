use std::{
    fs::File,
    io::{BufReader, Read, Write},
    path::Path,
};

use xot::{output::Indentation, Node, Xot};

pub(crate) fn indent(xml: &Path, w: &mut impl Write) -> anyhow::Result<()> {
    let mut xot = Xot::new();
    let root = load_xml_file(xml, &mut xot)?;

    let parameters = xot::output::xml::Parameters {
        indentation: Some(Indentation::default()),
        ..Default::default()
    };

    Ok(xot.serialize_xml_write(parameters, root, w)?)
}

fn load_xml_file(xml_path: &Path, xot: &mut Xot) -> anyhow::Result<Node> {
    let xml_file = File::open(xml_path)?;
    let mut buf_reader = BufReader::new(xml_file);
    let mut xml = String::new();
    buf_reader.read_to_string(&mut xml)?;
    Ok(xot.parse(&xml)?)
}
