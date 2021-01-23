use crate::{Document, DocumentType, LinesData, LinesError, Orientation, Page};
use json::{self, JsonValue};
use std::{
    fs::{self, File},
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
        let (pages, version) = get_pages(doc_path, &content_json)?;
        Ok(Document {
            orientation: get_orientation(&content_json),
            document_type: get_document_type(doc_path, &content_json),
            pages,
            version,
        })
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
    match doc.document_type {
        Err(e) => panic!("{}", e),
        Ok(DocumentType::Pdf(_)) => {}
        _ => panic!("Expected PDF document"),
    }
    assert_eq!(
        5,
        doc.version
            .unwrap_or_else(|_| panic!("Version could not be determined"))
    );
    assert_eq!(1, doc.pages.len());
}

fn get_orientation(object: &json::object::Object) -> Result<Orientation, LinesError> {
    Ok(match get_json_string(&object, "orientation")?.as_ref() {
        "landscape" => Orientation::Landscape,
        "portrait" => Orientation::Portrait,
        s => Err(LinesError::JsonStructure(format!(
            "{} is not a recognized orientation value",
            s
        )))?,
    })
}

fn get_document_type(
    doc_path: &Path,
    object: &json::object::Object,
) -> Result<DocumentType, LinesError> {
    Ok(match get_json_string(&object, "fileType")?.as_ref() {
        "pdf" => {
            let mut path = PathBuf::from(doc_path);
            path.set_extension("pdf");
            DocumentType::Pdf(lopdf::Document::load(path)?)
        }
        "epub" => DocumentType::Epub,
        "notebook" => DocumentType::Notebook,
        s => Err(LinesError::JsonStructure(format!(
            "{} is not a recognized document type",
            s
        )))?,
    })
}

fn get_json_string(object: &json::object::Object, key: &str) -> Result<String, LinesError> {
    match object.get(key) {
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

/// Returns the pages and the found reMarkable file format versions
fn get_pages(
    doc_path: &Path,
    content_json: &json::object::Object,
) -> Result<(Vec<Page>, Result<i32, LinesError>), LinesError> {
    // TODO: Also read *-metadata.json for layer names
    let page_id_array = match content_json.get("pages") {
        Some(JsonValue::Array(array)) => Ok(array),
        _ => Err(LinesError::JsonStructure("Mising pages array".to_string())),
    }?;
    let mut page_ids = Vec::with_capacity(page_id_array.len());
    for id_json_value in page_id_array {
        match id_json_value {
            JsonValue::String(id) => page_ids.push(id.to_string()),
            JsonValue::Short(id) => page_ids.push(id.to_string()),
            _ => {
                return Err(LinesError::JsonStructure(
                    "Values of the `pages` array must be strings".to_string(),
                ))
            }
        }
    }
    let mut pages = Vec::with_capacity(page_ids.len());
    const UNSET: i32 = -1;
    let mut version = if page_ids.len() == 0 {
        Err(LinesError::VersionError("Can't determine version for document without pages".to_string()))
    } else {
        Ok(UNSET)
    };
    for id in page_ids {
        let mut file = File::open(doc_path.join(format!("{}.rm", id)))?;
        let lines_data = LinesData::parse(&mut file)?;
        version = match version {
            Err(e) => Err(e),
            Ok(UNSET) => Ok(lines_data.version),
            Ok(version) if version == lines_data.version => Ok(version),
            _ => Err(LinesError::VersionError("Mixed versions".to_string())),
        };
        for page in lines_data.pages {
            pages.push(page);
        }
    }
    Ok((pages, version))
}
