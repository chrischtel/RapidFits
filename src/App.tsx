import { createSignal } from "solid-js";
import "./App.css";

function App() {
  const [brightness, setBrightness] = createSignal(50);
  const [contrast, setContrast] = createSignal(50);
  const [zoom, setZoom] = createSignal(100);

  return (
    <div class="app-layout">
      {/* Top Toolbar */}
      <header class="toolbar">
        <h1>RapidFits - GPU FITS Viewer</h1>
        <div class="toolbar-actions">
          <button>Open File</button>
          <button>Export</button>
          <button>Settings</button>
        </div>
      </header>

      {/* Main content area */}
      <div class="main-content">
        {/* Left Sidebar - Properties & Controls */}
        <aside class="sidebar">
          <div class="panel">
            <h3>Image Properties</h3>
            <div class="property">
              <label>Brightness</label>
              <input 
                type="range" 
                min="0" 
                max="100" 
                value={brightness()}
                onInput={(e) => setBrightness(Number(e.currentTarget.value))}
              />
              <span>{brightness()}%</span>
            </div>
            
            <div class="property">
              <label>Contrast</label>
              <input 
                type="range" 
                min="0" 
                max="100" 
                value={contrast()}
                onInput={(e) => setContrast(Number(e.currentTarget.value))}
              />
              <span>{contrast()}%</span>
            </div>

            <div class="property">
              <label>Zoom</label>
              <input 
                type="range" 
                min="10" 
                max="500" 
                value={zoom()}
                onInput={(e) => setZoom(Number(e.currentTarget.value))}
              />
              <span>{zoom()}%</span>
            </div>
          </div>

          <div class="panel">
            <h3>Color Mapping</h3>
            <button class="full-width">Linear</button>
            <button class="full-width">Logarithmic</button>
            <button class="full-width">Square Root</button>
            <button class="full-width">Histogram Eq.</button>
          </div>

          <div class="panel">
            <h3>Statistics</h3>
            <div class="stat">
              <span>Min:</span>
              <span>0.0</span>
            </div>
            <div class="stat">
              <span>Max:</span>
              <span>65535.0</span>
            </div>
            <div class="stat">
              <span>Mean:</span>
              <span>1024.5</span>
            </div>
          </div>
        </aside>

        {/* Center - WGPU renders here (behind this transparent div) */}
        <div class="viewer-area">
          {/* WGPU renders the full window, this area is just transparent */}
          <div class="viewer-overlay">
            {/* Optional overlays, crosshairs, info text, etc. */}
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
