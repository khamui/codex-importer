//! starting the program to import file(s) under given path.
//!
//! To import notes into codex, this program *synchronizes the save.json
//! with the note-files in the folder of the given path.
//! - save.json: stores the state of notes and notebooks (load & save)
//! - path: cmd line arg of path, where note files are stored to be imported

// Todos:
//  - copy files from specified import path first! [done]
//  - remove /notes/* which are not in save.json [done]
//  - do not create notebook with same name, use that.
//  - save save.json files as backup.
//  - better messaging!
//  - stop codex and restart after import

use std::fs::{self, DirEntry, File};
use std::io::{Error, Read};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use chrono::Local;
use clap::Parser;

const NOTES_PATH: &str = "/home/khamui/.config/codex/notes/";
const SAVE_JSON_PATH: &str = "/home/khamui/.config/codex/save.json";

/// custom types used to type input json tree
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

/// entry point
fn main() {
    let args = Args::parse();
    if args.path.exists() {
        println!("Processing notes in {:?}.", args.path);
        edit_save_json(args.path);
    } else {
        println!("The specified path \"{:?}\" does not exist.", args);
    };
}

/// reading filenames of given path.
fn read_filenames(import_path: &PathBuf) -> Vec<String> {
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

/// extracting filename from DirEntry
fn get_filename_of(entry: Result<DirEntry, Error>) -> String {
    let filename_str = entry.map(|en| en
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned()
    );
    filename_str.expect("Not able to get filename.")
}

/// editing save.json based on files found under import path.
/// - argument: import_path
fn edit_save_json(import_path: PathBuf) {
    let mut file = File::open(SAVE_JSON_PATH)
        .expect("Reading save.json not possible.");
    let mut buffer = String::new();
    let json_as_string = match file.read_to_string(&mut buffer) {
        Ok(_) => buffer,
        Err(_) => String::from("Reading file into buffer not possible.")
    };

    let mut json: RootItem = serde_json::from_str::<RootItem>(&json_as_string)
        .expect("Error parsing as json");

    // Desctructure and check if at least one "items" is an array
    let all_filenames_in_json: Vec<String> = get_identifiers_of(&json.items);
    let all_filenames_in_filetree: Vec<String> = read_filenames(&import_path);
    let (delta_tree_filenames, delta_node_filenames) =
        get_deltas(&all_filenames_in_json, &all_filenames_in_filetree);

    //println!("delta files {:?}", &delta_tree_filenames);
    //println!("all json {:?}", &all_filenames_in_json);
    //println!("delta json {:?}", &delta_node_filenames);
    //println!("json content {:?}", &json.items);

    copy_notes_files(import_path, &all_filenames_in_filetree);
    delete_stale(delta_node_filenames, &mut json);

    println!("json content {:?}", &json.items);

    let notes = create_notebook_children(delta_tree_filenames);
    let notebook = create_notebook(notes);

    if !notebook.is_none() {
        json.items.push(CodexItem::Notebook(notebook.unwrap()));
    } else {
        println!("Nothing imported. All up to date!")
    }

    let new_save_json = match serde_json::to_string(&json) {
        Ok(nb_string) => nb_string,
        Err(e) => {
            eprint!("{e}");
            String::new()
        }
    };

    match fs::write(SAVE_JSON_PATH, new_save_json) {
        Ok(_) => println!("File successfully written!"),
        Err(e) => eprintln!("{e}")
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
                let mut items_filenames_recursed =
                    get_identifiers_of(&notebook.children);
                items_filenames.append(&mut items_filenames_recursed);
            }
        }
    }
    items_filenames
}

// case a: a exists in json, but not as file --> copy a --> [done]
// case b: b already exists in save.json and as file. --> msg
// case c: c is stale. -> remove json-node of c from save.json --> [done]
// case d: d to be copied and imported as unnamed -> create notebook and note(s) --> [done]
// [a,b,c] -> save.json
// [b] -> /codex/notes
// [a,b,d] -> /import_folder

// helper: edit_save_json()
// FIXME: if json has note, but according file is missing, it remains in .json
fn get_deltas(node_filenames: &Vec<String>, tree_filenames: &Vec<String>) -> (Vec<String>, Vec<String>) {
    let mut delta_tree: Vec<String> = vec![];

    // delta_tree collects all files, which are not in node_filenames yet
    for tree_fname in tree_filenames.clone() {
        if !node_filenames.contains(&tree_fname) {
            delta_tree.push(tree_fname.clone());
        };
    }

    let mut delta_node: Vec<String> = vec![];

    for node_fname in node_filenames {
        if !tree_filenames.contains(&node_fname) {
            delta_node.push(node_fname.clone());
        };
    }

    (delta_tree, delta_node)
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

fn copy_notes_files(import_path: PathBuf, filenames: &Vec<String>) {
    let notes_path = PathBuf::from(NOTES_PATH);
    for filename in filenames {
        let dest_path = notes_path.join(filename);
        let source_path = import_path.join(filename);

        match fs::copy(&source_path, &dest_path) {
            Ok(_) => println!("Note {} file copied successfully.", filename),
            Err(e) => eprintln!("{}", e)
        }
    }
}

fn delete_stale(notes_fnames: Vec<String>, json: &mut RootItem) ->  &mut RootItem {
    json.items.retain(|item| {
        match item {
            CodexItem::Note(note) => !notes_fnames.contains(&note.file_name),
            CodexItem::Notebook(_) => true
        }
    });
    json
}
