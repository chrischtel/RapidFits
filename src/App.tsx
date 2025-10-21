import { createSignal, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import "./App.css";

interface ImageStats {
  min: number;
  max: number;
  mean: number;
  stddev: number;
  median: number;
  histogram: number[];
}

function App() {
  const [brightness, setBrightness] = createSignal(50);
  const [contrast, setContrast] = createSignal(50);
  const [zoom, setZoom] = createSignal(100);
  const [panX, setPanX] = createSignal(0);
  const [panY, setPanY] = createSignal(0);
  const [stats, setStats] = createSignal<ImageStats | null>(null);
  const [stretchMin, setStretchMin] = createSignal(0);
  const [stretchMax, setStretchMax] = createSignal(65535);

  let isDragging = false;
  let lastMouseX = 0;
  let lastMouseY = 0;
  let histogramCanvas: HTMLCanvasElement | undefined;

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

  // Load image statistics
  const loadStats = async () => {
    const imageStats = await invoke<ImageStats>("get_image_stats");
    setStats(imageStats);
    setStretchMin(imageStats.min);
    setStretchMax(imageStats.max);
  };

  // Update stretch values
  const updateStretch = () => {
    invoke("update_stretch", {
      min: stretchMin(),
      max: stretchMax(),
    });
  };

  // Auto-stretch (percentile clipping)
  const autoStretch = () => {
    const s = stats();
    if (!s) return;

    // Simple percentile estimation from histogram
    const totalPixels = s.histogram.reduce((a, b) => a + b, 0);
    const lowThreshold = totalPixels * 0.005; // 0.5%
    const highThreshold = totalPixels * 0.995; // 99.5%

    let cumulative = 0;
    let minIdx = 0;
    let maxIdx = 255;

    for (let i = 0; i < s.histogram.length; i++) {
      cumulative += s.histogram[i];
      if (cumulative > lowThreshold && minIdx === 0) {
        minIdx = i;
      }
      if (cumulative > highThreshold) {
        maxIdx = i;
        break;
      }
    }

    const range = s.max - s.min;
    const newMin = s.min + (minIdx / 255) * range;
    const newMax = s.min + (maxIdx / 255) * range;

    setStretchMin(newMin);
    setStretchMax(newMax);
    updateStretch();
  };

  // Draw histogram
  const drawHistogram = () => {
    const s = stats();
    if (!s || !histogramCanvas) return;

    const ctx = histogramCanvas.getContext("2d");
    if (!ctx) return;

    const width = histogramCanvas.width;
    const height = histogramCanvas.height;

    // Clear canvas
    ctx.fillStyle = "#1a1a1a";
    ctx.fillRect(0, 0, width, height);

    // Find max value for scaling
    const maxCount = Math.max(...s.histogram);

    // Draw histogram bars
    const barWidth = width / s.histogram.length;
    ctx.fillStyle = "#24c8db";

    for (let i = 0; i < s.histogram.length; i++) {
      const barHeight = (s.histogram[i] / maxCount) * height;
      ctx.fillRect(i * barWidth, height - barHeight, barWidth, barHeight);
    }

    // Draw stretch markers
    const minPos = ((stretchMin() - s.min) / (s.max - s.min)) * width;
    const maxPos = ((stretchMax() - s.min) / (s.max - s.min)) * width;

    ctx.strokeStyle = "#ff6b6b";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(minPos, 0);
    ctx.lineTo(minPos, height);
    ctx.stroke();

    ctx.strokeStyle = "#51cf66";
    ctx.beginPath();
    ctx.moveTo(maxPos, 0);
    ctx.lineTo(maxPos, height);
    ctx.stroke();
  };

  onMount(async () => {
    updateView(); // Initial view
    await loadStats(); // Load statistics
    drawHistogram(); // Draw histogram
  });

  return (
    <div class="app-layout">
      {/* Top Toolbar */}
      <header class="toolbar">
        <h1>RapidFits - GPU FITS Viewer</h1>
        <div class="toolbar-actions">
          <button onClick={openFileDialog}>Open File</button>
          <button>Export</button>
          <button type="button">Settings</button>
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
              <button
                type="button"
                onClick={() => {
                  setZoom(100);
                  setPanX(0);
                  setPanY(0);
                  updateView();
                }}
              >
                Reset View
              </button>
            </div>
          </div>

          <div class="panel">
            <h3>Color Mapping</h3>
            <button class="full-width">Linear</button>
            <button class="full-width">Logarithmic</button>
            <button class="full-width">Square Root</button>
            <button type="button" class="full-width">
              Histogram Eq.
            </button>
          </div>

          <div class="panel">
            <h3>Histogram & Stretch</h3>
            <canvas
              ref={histogramCanvas}
              width="240"
              height="100"
              style={{
                width: "100%",
                border: "1px solid rgba(255,255,255,0.2)",
                "border-radius": "4px",
                "margin-bottom": "12px",
              }}
            />

            <div class="property">
              <label>Stretch Min</label>
              <input
                type="range"
                min={stats()?.min ?? 0}
                max={stats()?.max ?? 65535}
                step={stats() ? (stats()!.max - stats()!.min) / 1000 : 1}
                value={stretchMin()}
                onInput={(e) => {
                  setStretchMin(Number(e.currentTarget.value));
                  updateStretch();
                  drawHistogram();
                }}
              />
              <span>{stretchMin().toFixed(0)}</span>
            </div>

            <div class="property">
              <label>Stretch Max</label>
              <input
                type="range"
                min={stats()?.min ?? 0}
                max={stats()?.max ?? 65535}
                step={stats() ? (stats()!.max - stats()!.min) / 1000 : 1}
                value={stretchMax()}
                onInput={(e) => {
                  setStretchMax(Number(e.currentTarget.value));
                  updateStretch();
                  drawHistogram();
                }}
              />
              <span>{stretchMax().toFixed(0)}</span>
            </div>

            <button
              type="button"
              class="full-width"
              onClick={() => {
                autoStretch();
                drawHistogram();
              }}
            >
              Auto Stretch
            </button>
          </div>

          <div class="panel">
            <h3>Statistics</h3>
            <div class="stat">
              <span>Min:</span>
              <span>{stats()?.min.toFixed(2) ?? "N/A"}</span>
            </div>
            <div class="stat">
              <span>Max:</span>
              <span>{stats()?.max.toFixed(2) ?? "N/A"}</span>
            </div>
            <div class="stat">
              <span>Mean:</span>
              <span>{stats()?.mean.toFixed(2) ?? "N/A"}</span>
            </div>
            <div class="stat">
              <span>Median:</span>
              <span>{stats()?.median.toFixed(2) ?? "N/A"}</span>
            </div>
            <div class="stat">
              <span>StdDev:</span>
              <span>{stats()?.stddev.toFixed(2) ?? "N/A"}</span>
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
            <div
              style={{
                position: "absolute",
                top: "10px",
                right: "10px",
                background: "rgba(0,0,0,0.7)",
                padding: "8px",
                "border-radius": "4px",
                color: "white",
                "font-size": "12px",
              }}
            >
              Zoom: {zoom().toFixed(0)}% | Pan: ({panX().toFixed(2)},{" "}
              {panY().toFixed(2)})
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

async function openFileDialog() {
  const filepath = await open({
    multiple: false,
    directory: false,
  });
  invoke("open_single_fits_file", { path: filepath });
}

export default App;
