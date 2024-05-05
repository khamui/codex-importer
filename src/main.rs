use std::fs::{DirEntry, File, read_dir};
use std::io::{Error, Read};

use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

#[derive(Serialize, Deserialize)]
enum CodexItem {
    Notebook(Notebook),
    Note(Note)
}

#[derive(Serialize, Deserialize)]
struct RootItem {
    schema_version: Number,
    items: Vec<CodexItem>,
}

#[derive(Serialize, Deserialize)]
struct Notebook {
    color: String,
    icon: String,
    id: String,
    name: String,
    children: Vec<CodexItem>, // Vec<Note>
    opened: bool
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Note {
    color: String,
    icon: String,
    id: String,
    name: String,
    favorited: bool,
    file_name: String,
    text_content: String
}


fn main() {
    //read_files();
    edit_save_json();
}

fn read_files() {
    // read files in codex/notes/
    let mut filenames_in_dir: Vec<String> = Vec::new();
    match read_dir("/home/khamui/.config/codex/notes/") {
        Ok(dir) => {
            for entry in dir {
                filenames_in_dir.push(get_filename_of(entry));
            }
        },
        Err(e) => {
            eprintln!("{}", e);
        }
    }
    println!("{:?}", filenames_in_dir)
}

// helper: read_files()
fn get_filename_of(entry: Result<DirEntry, Error>) -> String {
    let filename_str = entry.map(|en| en
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned()
    );
    filename_str.expect("Could not figure out filename!")
}

fn edit_save_json() {
    // open save.json file
    let mut file = File::open("/home/khamui/.config/codex/save.json")
        .expect("Reading save.json not possible");

    let mut buffer = String::new();

    let json_as_string = match file.read_to_string(&mut buffer) {
        Ok(_) => buffer,
        Err(_) => String::from("Reading file into buffer not possible!")
    };

    let json: RootItem = serde_json::from_str::<RootItem>(&json_as_string)
        .expect("Error parsing as json");

    // Desctructure and check if at least one "items" is an array
    recurse_and_find_items(json.items);

    //let new_notebook = Notebook {
    //    color: "#999999".to_owned(),
    //    icon: "book-2".to_owned(),
    //    id: "xx".to_owned(),
    //    name: "Autogen from Codex-Importer".to_owned(),
    //    opened: true,
    //    children: vec!()
    //};

    // 2. traverse through all children and item>>children and compare
    //    if matched filenames found. Only keep those which did not match.

    // 3. create new item (book, maybe with date) and insert children for each
    //    not-matched. The signature is as follows:
    //    {
    //      "name": "Example Note",
    //      "id": "1a80bb86-467d-40fa-b35f-64c58e69a51e",
    //      "color": "#999999",
    //      "icon": "file-text",
    //      "fileName": "Example-Note1a80bb86-467d-40",
    //      "textContent": "",
    //      "favorited": false
    //    },
}

// helper: edit_save_json()
fn recurse_and_find_items(items: Vec<CodexItem>) {
    let mut item_names: Vec<String> = vec!();
    for item in items {
        match item {
            CodexItem::Note(note) => {
                item_names.push(note.name);
            },
            CodexItem::Notebook(notebook) => {
                item_names.push(notebook.name);
                recurse_and_find_items(notebook.children);
            }
        }
        //if item.get("children") != None {
        //    recurse_and_find_items(item["children"].as_array())
        //}
    }
    println!("{:?}", item_names);
}

// helper: edit_save_json()
fn keep_to_be_migrated() {
    // compare files with save.json entries
}

fn save_json() {
    // save save.json
}
