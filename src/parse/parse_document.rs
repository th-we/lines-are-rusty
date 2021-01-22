use crate::{Document, LinesError, Orientation};
use json::{self, JsonValue};
use std::{
    fs::{self},
    path::{Path, PathBuf},
};

impl Document {
    /// relative to the supplied path, the following files are expected to be
    /// found:
    ///
    /// Assuming the path is e.g. `foo/be26389680f0`, then the following files
    /// are expected to exist:
    /// * `foo/be26389680f0.content`
    /// * `foo/be26389680f0.metadata`
    /// * `foo/be26389680f0/*.rm` (some lines files inside the folder)
    /// * `foo/be26389680f0/*-metadata.json` (one JSON file for each *.rm file
    ///   with the same prefix)
    /// * potentially a `foo/be26389680f0.pdf` or `foo/be26389680f0.epub` file
    ///
    /// The prefixes of the `*.rm` and `*-metadata.json` files are typically
    /// UUIDs. the `*.content` files are JSON files that have a `pages` field
    /// with an array of all theses UUIDs.
    pub fn load(doc_path: &Path) -> Result<Document, LinesError> {
        let content_json = get_content_json(doc_path)?;
        let orientation = get_orientation(content_json);
        Ok(Document { orientation })
    }
}

#[test]
fn test_load() {
    let doc = Document::load(Path::new(
        "test/data/version5/annotated_square_pdf/c30a5220-ae5b-4107-ae2f-eeea97becd4e",
    ))
    .unwrap_or_else(|e| panic!("{}", e));
    assert_eq!(
        doc.orientation.unwrap_or_else(|e| panic!("{}", e)),
        Orientation::Portrait
    );
}

fn get_orientation(object: json::object::Object) -> Result<Orientation, LinesError> {
    Ok(match get_json_string(&object, "orientation")?.as_ref() {
        "landscape" => Orientation::Landscape,
        "portrait" => Orientation::Portrait,
        s => Err(LinesError::JsonStructure(format!(
            "{} is not a recognized orientation value",
            s
        )))?,
    })
}

fn get_json_string(object: &json::object::Object, key: &str) -> Result<String, LinesError> {
    match object.get("orientation") {
        None => Err(LinesError::JsonStructure(format!("Missing {} entry", key))),
        // JsonValue knows two types of strings: String and Short
        Some(JsonValue::String(s)) => Ok(s.to_string()),
        Some(JsonValue::Short(s)) => Ok(s.to_string()),
        _ => Err(LinesError::JsonStructure(format!(
            "{} entry must be a string value",
            key
        ))),
    }
}

fn get_content_json(doc_path: &Path) -> Result<json::object::Object, LinesError> {
    let mut path = PathBuf::from(doc_path);
    path.set_extension("content");
    let json = json::parse(fs::read_to_string(path.as_path())?.as_str())?;
    match json {
        JsonValue::Object(obj) => Ok(obj),
        _ => Err(LinesError::JsonStructure(format!(
            "Expected an object at top level of .content JSON file"
        ))),
    }
}
