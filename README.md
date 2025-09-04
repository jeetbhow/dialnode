<h1 align="center">Dialnode</h1>

<p align="center"><strong>An open-source, graph-based dialogue editor built for indie game developers.</strong></p>
<p align="center"><em>Currently in proof-of-concept stage.</em></p>

<img src="./images/example.gif" alt="Example" />

## What is this not?
Dialnode is not:
- A plugin for a game engine
- A dialogue system for a game engine

Instead, it exports a file (JSON, TOML, YAML, etc.) that implements a specification. You are then free to implement your dialogue system around that specification. This will typically involve parsing the file and then converting it into some internal representation that your game can understand. 

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

## Features
- Visual graph-based editor for building complex branching video-game dialogues
- Repository manager for storing your dialogues, portraits, speakers, and skills associated with a particular project all in one folder
- Explorer on the sidebar to organize your dialogues into folders and view them
- Export to JSON for use in a game engine (portraits only work with Godot for now)


## Technologies Used
- [Svelte](https://svelte.dev/) – UI framework
- [Electron](https://www.electronjs.org/) – For building cross-platform desktop apps
- [Node.js](https://nodejs.org/) – Runtime environment
- [TypeScript](https://www.typescriptlang.org/) – Type-safe code
- [SQLite](https://www.sqlite.org/) – Database

## Building the project
This section is for users that want to build the project on their own. If you just want to use the app, you can download it from the [releases page](https://github.com/jeetbhow/dialnode/releases).

### Recommended IDE Setup

[VSCode](https://code.visualstudio.com/) + [ESLint](https://marketplace.visualstudio.com/items?itemName=dbaeumer.vscode-eslint) + [Prettier](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode)

### Project Setup

#### Install

```bash
npm install
```

#### Development

```bash
npm run dev
```

#### Build

```bash
# For Windows
npm run build:win

# For macOS
npm run build:mac

# For Linux
npm run build:linux
```

## License
This project is licensed under the MIT License – see the [LICENSE](LICENSE) file for details.

## Author
Created by [Jeet Bhowmik](https://github.com/jeetbhow).