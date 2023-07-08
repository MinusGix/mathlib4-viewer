# MathLib4 Viewer
This is a viewer for MathLib 4 that tries to improve on some aspects.

## Architecture
The original [mathlib docs](https://leanprover-community.github.io/mathlib4_docs/) are hosted on github.io which hosts static sites (basically). This means their search functionality is limited in how it can be implemented, and it has to download the data on the client side.  
The current implementation compresses the ~74M JSON pretty well (to ~7.4M) and it caches it in the browser's indexedDB. However, this is still slow to search through on any browser I've used and makes the site feel sluggish.  

So, this architecture has a server (written in Rust, it could be written in Lean 4 but I don't really use Lean 4 for normal programming) that can process the search requests itself and thus provide responses faster.

## Data
The docs are downloaded from https://github.com/leanprover-community/mathlib4_docs which are prebuilt docs, the same as what you see on the official community website.  
They are put into a data folder, which depends on the platform.  
Linux: `~/.local/share/m4doc/mathlib_docs`

The structure is expected to have:
```bash
- m4doc
  - mathlib_docs
    - doc
      - index.html
      - ...
```

## Ideas
Some modifications of the views might have issues if they're generated for each HTML file. This can be scripted to modify them all, though requires parsing HTML and modifying many files.  
It might be possible to modify the mathlib doc generation to put it into a more usable format (Q: do they already transform them to an intermediate form?) and then make it easy to do custom output.

- Open the webpage in the browser when you start it
- Customize the rendering more.
    - Ex: Hide Navbar
    - Ex: Focus in on a specific navbar. I usually stay within Mathlib
    - Ex: Hide the right sidebar, it usually has too many entries for it to be useful
- More complex search
    - Ex: search based on formula somehow? We might be able to hook up to the lean LSP somehow?
    - We could make search take into account the current folder better?
        - Like prioritize probability related files if we're in the probability folder.
    - Could do some manual downplaying of rarely relevant results, like impl details of Real and stuff.
- Host this somewhere? Might be able to get by with a pretty cheap host? Especially if we do some smarter stuff?
- Replace 'how about'? I don't know where this is even used.
- Replace instances.js?