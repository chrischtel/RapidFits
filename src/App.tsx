import { createSignal, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [brightness, setBrightness] = createSignal(50);
  const [contrast, setContrast] = createSignal(50);
  const [zoom, setZoom] = createSignal(100);
  const [panX, setPanX] = createSignal(0);
  const [panY, setPanY] = createSignal(0);
  
  let isDragging = false;
  let lastMouseX = 0;
  let lastMouseY = 0;

  // Send view updates to Rust backend
  const updateView = () => {
    invoke("update_view", {
      zoom: zoom() / 100, // Convert percentage to scale
      panX: panX(),
      panY: panY(),
    });
  };

  // Mouse wheel for zoom
  const handleWheel = (e: WheelEvent) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1; // Zoom in/out
    const newZoom = Math.max(10, Math.min(500, zoom() * delta));
    setZoom(newZoom);
    updateView();
  };

  // Mouse drag for pan
  const handleMouseDown = (e: MouseEvent) => {
    isDragging = true;
    lastMouseX = e.clientX;
    lastMouseY = e.clientY;
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (!isDragging) return;
    
    const deltaX = (e.clientX - lastMouseX) / window.innerWidth;
    const deltaY = (e.clientY - lastMouseY) / window.innerHeight;
    
    setPanX(panX() + deltaX * 2); // Scale factor for sensitivity
    setPanY(panY() + deltaY * 2);
    
    lastMouseX = e.clientX;
    lastMouseY = e.clientY;
    
    updateView();
  };

  const handleMouseUp = () => {
    isDragging = false;
  };

  onMount(() => {
    updateView(); // Initial view
  });

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
                onInput={(e) => {
                  setZoom(Number(e.currentTarget.value));
                  updateView();
                }}
              />
              <span>{zoom()}%</span>
            </div>
            
            <div class="property">
              <button onClick={() => { setZoom(100); setPanX(0); setPanY(0); updateView(); }}>
                Reset View
              </button>
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
        <div 
          class="viewer-area"
          onWheel={handleWheel}
          onMouseDown={handleMouseDown}
          onMouseMove={handleMouseMove}
          onMouseUp={handleMouseUp}
          onMouseLeave={handleMouseUp}
          style={{ cursor: isDragging ? "grabbing" : "grab" }}
        >
          {/* WGPU renders the full window, this area is just transparent */}
          <div class="viewer-overlay">
            {/* Optional overlays, crosshairs, info text, etc. */}
            <div style={{
              position: "absolute",
              top: "10px",
              right: "10px",
              background: "rgba(0,0,0,0.7)",
              padding: "8px",
              "border-radius": "4px",
              color: "white",
              "font-size": "12px"
            }}>
              Zoom: {zoom().toFixed(0)}% | Pan: ({panX().toFixed(2)}, {panY().toFixed(2)})
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
