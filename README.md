# Open Download Manager

An open-source download manager written in Rust and React + Typescript using Tauri.

Currently I am working on it in my free time. The reason I am making this project is that I could not find a good & modern looking download manager which is reliable.

No worries about background mining, if you don't believe it, look at the code and compile it yourself!

## Current status and planned features
Currently download pausing and resuming works.

### Planned features:
- File name change before downloading
- Download Scheduling
- Adding downloads from links
- Multithreaded Download (after release)


## For Developers (Build from source)

To build and run this repo, run the following:
```sh
# Clone this repo
git clone https://github.com/coderarda/the-download-manager
cd the-download-manager
# Install Dependencies
pnpm i # Or npm, yarn

# Run the App
pnpm tauri dev

# For debugging, check out tauri debugging documentation
```