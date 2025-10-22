# RapidFits

RapidFits is our vision of viewing and sorting a big batch of FITS images.  
We’re two passionate astrophotographers trying to build something we’d actually use ourselves.  
Right now it’s still **super early**, more of a small prototype than a finished app.

At the moment we’re just exploring what a fast, simple and modern FITS viewer could look and feel like.  
There isn’t a clear direction yet, just experiments, ideas, and a lot of motivation to see where it goes.

Our goal is simple: make it **easy to browse, check and organize astronomical images** without all the slow or old-school software out there.  
Maybe RapidFits will turn into a full desktop app, maybe a toolkit for FITS handling, or maybe something else entirely — we’ll see along the way.

---

### Current State

RapidFits is currently in its **prototype phase**.  
We’re building the basic structure, testing ideas and figuring out how everything should work together.

- **Christian** – working on the **backend** and **FITS renderer**, focusing on efficient data loading, parsing, and GPU-based rendering  
- **Julian** – working on the **basic UI and preview interface**, experimenting with layouts, state management, and the first user interaction flow

Right now nothing is stable, and that’s fine. We’re still exploring and shaping what RapidFits could become.

---

### Roadmap

**Short-term goals**
- [ ] FITS header parsing (read only)  
- [ ] Simple image preview window  
- [ ] Multi-file batch browsing  
- [ ] Clean and minimal UI prototype  

**Long-term vision**
- Advanced stretch algorithms  
- High performance rendering  
- Automatic detection of bad or blurry FITS images (for stacking or filtering)  
- Customizable thresholds, automatic folder sorting, and more

---

### Tech Stack

RapidFits is built using:
- **Rust** – for backend performance, FITS parsing, and rendering work  
- **Tauri** – for the desktop shell and native integration  
- **TypeScript + HTML/CSS** – for the experimental UI layer

The stack will probably change as we experiment and figure out what works best.

---

### Technical Infos

#### General
Right now, our backend renders your FITS files directly on the **GPU**, which gives really good performance and smooth image handling.  
At the moment, only GPU rendering is supported since that’s standard for most computers today.  
However, we plan to add a CPU fallback later so the app can still run on systems without a dedicated or compatible GPU.

#### Performance
Performance is one of our main focuses. We’re experimenting with different data loading and caching strategies to make opening large batches of FITS files feel instant.  
In the future, we want to benchmark GPU vs CPU performance, and possibly even add adaptive rendering — so the viewer can automatically switch between modes depending on your hardware.

---

### Contributing

Right now RapidFits is **not ready for public contributions**, but we really appreciate feedback, ideas, and general discussion.  
If you’re into FITS processing, Rust, or astrophotography software, feel free to open an issue or reach out.

---

### About

RapidFits is made by **Christian** and **Julian** – two astrophotographers building tools they wish existed.  
This project is developed openly as an experiment, a learning process, and hopefully something useful for the astro community later on.

---

**License:** MIT  
**Status:** Prototype / Work in Progress
