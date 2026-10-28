<h1 align="center">Dialnode</h1>

<p align="center"><strong>An open-source, graph-based dialogue editor built for indie game developers.</strong></p>
<p align="center"><em>Currently in proof-of-concept stage.</em></p>

![Example](./images/example.gif)

<p align="center">
  <a href="https://github.com/jeetbhow/dialnode/releases">Get the pre-release version</a>
</p>


**Note: Dialnode is now a native Rust app built with [egui](https://github.com/emilk/egui). It replaces the earlier Electron + Svelte version. Repositories are now plain folders of JSON files, so repositories created by the Electron version (which used SQLite) can't be opened.**

## What problem does this solve?

 While working on my own indie game, I found that managing complex, branching dialogue with skill checks in traditional text files was a nightmare. It was difficult to visualize the flow of conversation, track different paths, and ensure all branches connected correctly.

To solve this, I created Dialnode: a visual, graph-based editor designed to make dialogue creation intuitive and efficient. Instead of getting lost in lines of code or tangled scripts, you can build your conversations visually, node by node, just like a flowchart.

## What is this not?
Dialnode is not:
- A plugin for a game engine
- A dialogue system for a game engine

Instead, it exports a file (JSON, TOML, YAML, etc.) that implements a specification. You will then build your game's dialogue system around that specification. This will typically involve parsing the file and then converting it into some internal representation that your game can understand. 

This is an example of the JSON file produced by this program: 
```json
[
  {
    "id": "35ccc1a7-7203-4bf2-91aa-ccb37dd16809",
    "type": "start",
    "next": "6eef3c3e-84e7-4a08-b8a3-50219c45957f"
  },
  {
    "id": "6eef3c3e-84e7-4a08-b8a3-50219c45957f",
    "type": "text",
    "text": "What's your favourite color? ",
    "speaker": "Jeet",
    "next": "9256b0a2-f87e-4f11-b702-9e5e1eac547e"
  },
  {
    "id": "9256b0a2-f87e-4f11-b702-9e5e1eac547e",
    "type": "branchContainer",
    "branches": [
      "175250a8-2783-4792-9952-35c5c7cbf8b1",
      "b1a367cd-1ddd-4678-a0ac-7198323305d5"
    ]
  },
  {
    "id": "175250a8-2783-4792-9952-35c5c7cbf8b1",
    "parentId": "9256b0a2-f87e-4f11-b702-9e5e1eac547e",
    "type": "branch",
    "text": "Blue",
    "next": "2d8289c9-9570-49ca-9d72-d4983e8bf132"
  },
  {
    "id": "b1a367cd-1ddd-4678-a0ac-7198323305d5",
    "parentId": "9256b0a2-f87e-4f11-b702-9e5e1eac547e",
    "type": "branch",
    "text": "Red",
    "next": "603462cd-c28e-4005-b0c1-41228e693c52"
  },
  {
    "id": "2d8289c9-9570-49ca-9d72-d4983e8bf132",
    "type": "text",
    "text": "Cool, I'm a big fan of red as well.",
    "speaker": "Jeet",
    "next": "5cd0c026-cbec-43e4-b634-c3f15c2974d9"
  },
  {
    "id": "603462cd-c28e-4005-b0c1-41228e693c52",
    "type": "text",
    "text": "That color sucks.",
    "speaker": "Jeet",
    "next": "5cd0c026-cbec-43e4-b634-c3f15c2974d9"
  },
  {
    "id": "5cd0c026-cbec-43e4-b634-c3f15c2974d9",
    "type": "end"
  }
]
```

When parsing this file in your game, you should use the `type` property to implement different behaviour. You should use the `next` property of a node to navigate to the next node in the dialogue. 

The dialogue files Dialnode saves use this same format. Each node may also have a `layout` object that records where the node sits on the canvas, and which handles its outgoing connection uses. Your game can ignore it. File → Export writes a copy of the open dialogue without `layout`.

## Repository layout
A repository is an ordinary folder, so it works well with git:

```text
my-repository/
├── manifest.dial            repository details (name, Godot project location)
├── database.json            speakers, skill categories, and portraits
├── Chapter 1/               folders in the sidebar are real folders
│   └── Favourite colour.json
└── Untitled.json            each dialogue is one JSON file
```

Creating, renaming, moving, and deleting dialogues or folders in the sidebar does the same to the files on disk straight away. Deleted items go to your system trash. Folders and files whose names start with `.` are ignored, as are files that aren't valid dialogues (the app lists those when the repository opens).

Text nodes refer to speakers by name and to portraits by their `res://` path. Skill checks refer to skills by name.

## Features
- Visual graph-based editor for building complex branching video-game dialogues
- Repository folders that hold your dialogues, portraits, speakers, and skills for a project
- Explorer on the sidebar to organize your dialogues into folders and view them
- Dialogue files your game can read directly (portraits only work with Godot for now)


## Using the editor
- **Canvas:** drag the background to pan and scroll to zoom. Drag a node to move it, and click a node then press <kbd>Delete</kbd> to remove it.
- **Connecting nodes:** drag from a blue (source) handle to a red (target) handle. Click an edge to remove it.
- **Adding nodes:** use the toolbar at the top of the canvas, or right-click the background to add a node at that spot.
- **Sidebar:** drag dialogues and folders onto a folder to move them. Double-click an item, or right-click it, to rename it.
- **Saving:** <kbd>Ctrl</kbd>+<kbd>S</kbd> (or File → Save) writes every changed dialogue to its file. Changed dialogues are marked with • in the sidebar. Speakers, portraits, and skills are saved to `database.json` as soon as you change them.

## Technologies Used
- [Rust](https://www.rust-lang.org/)
- [egui / eframe](https://github.com/emilk/egui) – immediate-mode UI
- [serde_json](https://github.com/serde-rs/json) – dialogue and database files
- [rfd](https://github.com/PolyMeilex/rfd) – native file dialogs
- [trash](https://github.com/Byron/trash-rs) – moving deleted items to the system trash

## Building the project
This section is for users who want to build the project themselves. If you just want to use the app, you can download it from the [releases page](https://github.com/jeetbhow/dialnode/releases).

Install a stable [Rust toolchain](https://rustup.rs/), then:

```bash
cargo run            # development build
cargo test           # unit tests and headless UI tests
cargo build --release
```

The release binary is written to `target/release/dialnode`.

### WSL

On WSL the app switches to Mesa's software renderer automatically, unless you've set Mesa variables yourself. WSLg's Wayland compositor can fail to accept new windows (`NoCompositor` or `Connection reset by peer`). If that happens, run the app through X11 instead:

```bash
env -u WAYLAND_DISPLAY cargo run
```

### Debug helpers

Debug builds read two environment variables, which are handy for checking the UI without clicking through it:

- `DIALNODE_OPEN=path/to/repository` opens a repository folder on startup.
- `DIALNODE_SCREENSHOT=out.png` saves a screenshot after a moment and then quits.

`DIALNODE_SEED_DIR=/tmp cargo test seed_sample -- --ignored` writes a sample repository to `/tmp/sample`.

## License
This project is licensed under the MIT License – see the [LICENSE](LICENSE) file for details.

## Author
Created by [Jeet Bhowmik](https://github.com/jeetbhow).
