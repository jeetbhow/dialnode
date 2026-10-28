<h1 align="center">Dialnode</h1>

<p align="center"><strong>An open-source, graph-based dialogue editor built for indie game developers.</strong></p>
<p align="center"><em>Currently in proof-of-concept stage.</em></p>

![Example](./images/example.gif)

<p align="center">
  <a href="https://github.com/jeetbhow/dialnode/releases">Get the pre-release version</a>
</p>


**Note: Dialnode is now a native Rust app built with [egui](https://github.com/emilk/egui). It replaces the earlier Electron + Svelte version. Dialnode now works on any folder of dialogue JSON files, so repositories created by the Electron version (which used SQLite) can't be opened.**

## What problem does this solve?

 While working on my own indie game, I found that managing complex, branching dialogue with skill checks in traditional text files was a nightmare. It was difficult to visualize the flow of conversation, track different paths, and ensure all branches connected correctly.

To solve this, I created Dialnode: a visual, graph-based editor designed to make dialogue creation intuitive and efficient. Instead of getting lost in lines of code or tangled scripts, you can build your conversations visually, node by node, just like a flowchart.

## What is this not?
Dialnode is not:
- A plugin for a game engine
- A dialogue system for a game engine

Instead, it saves dialogue JSON files that your game can read directly. You build your game's dialogue system around this format, typically by parsing each file and converting it into an internal representation your game can use.

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

The dialogue files Dialnode saves use this same format. Each node may also have a `layout` object that records where the node sits on the canvas, and which handles its outgoing connection uses. Your game can ignore it.

## Folder layout
Dialnode opens any folder. There's no setup file: every dialogue JSON file in the folder and its subfolders shows up in the sidebar, so the folder can live anywhere and works well with git:

```text
my-dialogues/
├── Chapter 1/               folders in the sidebar are real folders
│   └── Favourite colour.json
└── Untitled.json            each dialogue is one JSON file
```

Creating, renaming, moving, and deleting dialogues or folders in the sidebar does the same to the files on disk straight away. Deleted items go to your system trash. Folders and files whose names start with `.` are ignored, as are files that aren't valid dialogues (the app lists those when the folder opens).

Speakers on text nodes and skills on skill checks are plain text that you type in the node panel.

## Features
- Visual graph-based editor for building complex branching video-game dialogues
- Works on any folder of dialogue files, with no project setup
- Explorer on the sidebar to organize your dialogues into folders and view them
- Dialogue files your game can read directly


## Using the editor
- **Folders:** Dialnode opens straight into the editor. Use File → Open Folder… to open a folder of dialogues, or an empty folder to start a new set.
- **Canvas:** drag the background to pan and scroll to zoom. Drag a node to move it, and click a node then press <kbd>Delete</kbd> to remove it.
- **Editing nodes:** click a node to open the panel on the right, where you edit its speaker, text, skill, or difficulty. Click the empty canvas to close it. Cards on the canvas show a summary and can't be typed into.
- **Connecting nodes:** drag from a blue (source) handle to a red (target) handle. Click an edge to remove it.
- **Adding nodes:** use the toolbar at the top of the canvas, or right-click the background to add a node at that spot.
- **Sidebar:** drag dialogues and folders onto a folder to move them. Double-click an item, or right-click it, to rename it.
- **Saving:** <kbd>Ctrl</kbd>+<kbd>S</kbd> (or File → Save) writes every changed dialogue to its file. Changed dialogues are marked with • in the sidebar.

## Technologies Used
- [Rust](https://www.rust-lang.org/)
- [egui / eframe](https://github.com/emilk/egui) – immediate-mode UI
- [serde_json](https://github.com/serde-rs/json) – dialogue files
- [rfd](https://github.com/PolyMeilex/rfd) – native file dialogs
- [trash](https://github.com/Byron/trash-rs) – moving deleted items to the system trash

## Building the project
This section is for users who want to build the project themselves. If you just want to use the app, you can download it from the [releases page](https://github.com/jeetbhow/dialnode/releases).

Install a stable [Rust toolchain](https://rustup.rs/), then:

```bash
cargo run            # development build
cargo build --release
```

The release binary is written to `target/release/dialnode`.

### WSL

On WSL the app adjusts two graphics settings automatically:

- It uses Mesa's software renderer, unless you've set Mesa variables yourself.
- It opens its window through X11 rather than Wayland, because WSLg's Wayland compositor often refuses new windows (`NoCompositor` or `Connection reset by peer`). To use Wayland anyway, run with `DIALNODE_WAYLAND=1`.

### Debug helpers

Debug builds read two environment variables, which are handy for checking the UI without clicking through it:

- `DIALNODE_OPEN=path/to/folder` opens a folder of dialogues on startup.
- `DIALNODE_SCREENSHOT=out.png` saves a screenshot after a moment and then quits.

## License
This project is licensed under the MIT License – see the [LICENSE](LICENSE) file for details.

## Author
Created by [Jeet Bhowmik](https://github.com/jeetbhow).
