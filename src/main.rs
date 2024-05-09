use std::fs::{self, read_dir, DirEntry, File};
use std::io::{Error, Read};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use chrono::Local;
use clap::Parser;

// Todos:
//  - copy files from specified import path first!
//  - remove /notes/* which are not used. and make import folder specifiable
//  - do not create notebook with same name, use that.
//  - save save.json files as backup.
//  - better messaging!
//  - stop codex and restart after import

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum CodexItem {
    Notebook(Notebook),
    Note(Note)
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
struct RootItem {
    schema_version: u32,
    items: Vec<CodexItem>,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
struct Notebook {
    color: String,
    icon: String,
    id: String,
    name: String,
    children: Vec<CodexItem>, // Vec<Note>
    opened: bool
}

#[derive(Debug)]
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

#[derive(Parser, Debug)]
struct Args {
    /// Path to import files
    #[arg(short, long)]
    path: PathBuf
}

fn main() {
    //read_files();
    let args = Args::parse();
    if args.path.exists() {
        println!("Success: {:?}", args.path);
        edit_save_json(args.path);
    } else {
        println!("Path does not exist: {:?}", args);
    };
}

fn read_filenames(import_path: PathBuf) -> Vec<String> {
    // read files in codex/notes/
    let mut filenames_in_dir: Vec<String> = Vec::new();
    match import_path.read_dir() {
        Ok(dir) => {
            for entry in dir {
                filenames_in_dir.push(get_filename_of(entry));
            }
        },
        Err(e) => {
            eprintln!("{}", e);
        }
    }
    filenames_in_dir
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

fn edit_save_json(import_path: PathBuf) {
    // open save.json file
    let mut file = File::open("/home/khamui/.config/codex/save.json")
        .expect("Reading save.json not possible");

    let mut buffer = String::new();

    let json_as_string = match file.read_to_string(&mut buffer) {
        Ok(_) => buffer,
        Err(_) => String::from("Reading file into buffer not possible!")
    };

    let mut json: RootItem = serde_json::from_str::<RootItem>(&json_as_string)
        .expect("Error parsing as json");

    // Desctructure and check if at least one "items" is an array
    let all_filenames_in_json: Vec<String> = get_identifiers_of(&json.items);
    let all_filenames_in_filetree: Vec<String> = read_filenames(import_path);
    //println!("json: {:?}, filetree: {:?}", all_filenames_in_json, all_filenames_in_filetree);
    let delta_filenames = get_delta(all_filenames_in_json, all_filenames_in_filetree);
    //println!("{:?}", json);

    let notes = create_notebook_children(delta_filenames);
    let notebook = create_notebook(notes);

    if !notebook.is_none() {
        json.items.push(CodexItem::Notebook(notebook.unwrap()));

        let new_save_json = match serde_json::to_string(&json) {
            Ok(nb_string) => nb_string,
            Err(e) => {
                eprint!("{e}");
                String::new()
            }
        };

        match fs::write("/home/khamui/.config/codex/save.json", new_save_json) {
            Ok(_) => println!("File successfully written!"),
            Err(e) => eprintln!("{e}")
        }
    } else {
        println!("Nothing imported. All up to date!")
    }
}

// helper: edit_save_json()
fn get_identifiers_of(items: &Vec<CodexItem>) -> Vec<String> {
    let mut items_filenames: Vec<String> = vec!();
    for item in items {
        match item {
            CodexItem::Note(note) => {
                items_filenames.push(note.file_name.clone());
            },
            CodexItem::Notebook(notebook) => {
                get_identifiers_of(&notebook.children);
            }
        }
    }
    items_filenames
}

// helper: edit_save_json()
// FIXME: if json has note, but according file is missing, it remains in .json
fn get_delta(node_filenames: Vec<String>, tree_filenames: Vec<String>) -> Vec<String> {
    let mut delta: Vec<String> = vec![];

    for tree_fname in tree_filenames {
        if !node_filenames.contains(&tree_fname) {
            delta.push(tree_fname);
        };
    }
    delta
}

fn create_notebook_children(notenames: Vec<String>) -> Vec<Note> {
    // create items of type Note for each input Vec items
    let mut notes: Vec<Note> = vec!();
    for (index, notename) in notenames.iter().enumerate() {
        let create_dt = format!("{}", Local::now().format("%Y%m%d"));
        let id = format!("automated_note_{}_{}", index + 1, &create_dt);
        let name = format!("unnamed {}", index + 1);

        notes.push(Note {
            color: "#999999".to_owned(),
            icon: String::from("file-text"),
            id,
            name,
            favorited: false,
            file_name: notename.to_owned(),
            text_content: String::from("")
        });
    }
    notes
}

fn create_notebook(children: Vec<Note>) -> Option<Notebook> {
    // create one notebook to bundle all automatically added notes

    if children.is_empty() {
        return None;
    }

    let create_dt = format!("{}", Local::now().format("%Y%m%d"));
    let id = format!("automated_{}", &create_dt);
    let name = format!("AUTO NOTEBOOK ({})", &create_dt);
    let codex_children = children.into_iter().map(CodexItem::Note).collect();

    Some(Notebook {
        color: "#00CD00".to_owned(),
        icon: "book-2".to_owned(),
        id,
        name,
        opened: true,
        children: codex_children
    })
}
