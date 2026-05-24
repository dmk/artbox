import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { TuiPreview } from "@dkkoval/tui-preview";
import "../styles/playground.css";

type FillMode = "none" | "solid" | "linear" | "radial";
type FontFamily = "default" | "banner" | "blocky" | "script" | "slant";
type Alignment =
  | "top-left"
  | "top"
  | "top-right"
  | "left"
  | "center"
  | "right"
  | "bottom-left"
  | "bottom"
  | "bottom-right";

type GradientStop = { position: number; color: string };
type ControlTab = "text" | "layout" | "fill";
type TerminalSize = { cols: number; rows: number };
type DragState = {
  x: number;
  y: number;
  cols: number;
  rows: number;
  cellWidth: number;
  cellHeight: number;
};

const baseUrl = import.meta.env.BASE_URL.endsWith("/")
  ? import.meta.env.BASE_URL
  : `${import.meta.env.BASE_URL}/`;

const wasmUrl = `${baseUrl}wasm/artbox-preview.wasm`;

const terminalTheme = {
  background: "#0b1020",
  foreground: "#dce6ff",
  cursor: "#7aa2f7",
  selectionBackground: "#223355",
  selectionForeground: "#dce6ff",
};

const alignmentOrder: Alignment[] = [
  "top-left", "top", "top-right",
  "left", "center", "right",
  "bottom-left", "bottom", "bottom-right",
];

const alignmentLabels: Record<Alignment, string> = {
  "top-left": "Top Left", top: "Top", "top-right": "Top Right",
  left: "Left", center: "Center", right: "Right",
  "bottom-left": "Bottom Left", bottom: "Bottom", "bottom-right": "Bottom Right",
};

const families: Array<{ label: string; value: FontFamily }> = [
  { label: "Default", value: "default" },
  { label: "Banner", value: "banner" },
  { label: "Blocky", value: "blocky" },
  { label: "Script", value: "script" },
  { label: "Slant", value: "slant" },
];

const fillModes: Array<{ label: string; value: FillMode }> = [
  { label: "None", value: "none" },
  { label: "Solid", value: "solid" },
  { label: "Linear", value: "linear" },
  { label: "Radial", value: "radial" },
];

const ALIGNMENT_RUST: Record<Alignment, string> = {
  "top-left": "TopLeft", top: "Top", "top-right": "TopRight",
  left: "Left", center: "Center", right: "Right",
  "bottom-left": "BottomLeft", bottom: "Bottom", "bottom-right": "BottomRight",
};

const PREVIEW_HEIGHT = 220;
const MIN_PREVIEW_COLS = 12;
const MIN_PREVIEW_ROWS = 6;
const DEFAULT_PREVIEW_COLS = 80;
const DEFAULT_PREVIEW_ROWS = 24;
const DEFAULT_CELL_WIDTH = 8;
const DEFAULT_CELL_HEIGHT = 18;
const SIZE_BADGE_HOLD_MS = 1000;
const SIZE_BADGE_FADE_MS = 220;

/* ── Reusable class strings ── */

const lblCls = "text-[0.58rem] font-pg-mono font-medium uppercase tracking-[0.08em] text-pg-text-dim select-none";
const valCls = "text-[0.62rem] font-pg-mono tabular-nums text-pg-text-mid";

/* ── Helpers ── */

function clamp(v: number, lo: number, hi: number) { return Math.min(Math.max(v, lo), hi); }

function hexToRgb(hex: string): [number, number, number] {
  const h = hex.replace("#", "");
  return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)];
}

function nextStopColor(stops: GradientStop[]): string {
  const palette = ["#00c8ff", "#ff5a78", "#a855f7", "#22d3ee", "#f59e0b", "#10b981"];
  const used = new Set(stops.map((s) => s.color.toLowerCase()));
  return palette.find((c) => !used.has(c)) ?? palette[stops.length % palette.length];
}

/* ── Tiny sub-components ── */

function Slider({ label, value, min, max, step, suffix, onChange, className }: {
  label: string; value: number; min: number; max: number; step: number;
  suffix?: string; onChange: (v: number) => void; className?: string;
}) {
  const display = step < 1 ? value.toFixed(2) : String(value);
  return (
    <div className={`flex flex-col gap-px flex-1 ${className ?? ""}`}>
      <div className="flex justify-between items-baseline">
        <span className={lblCls}>{label}</span>
        <span className={valCls}>{display}{suffix ?? ""}</span>
      </div>
      <input type="range" min={min} max={max} step={step} value={value}
        onChange={(e) => onChange(Number(e.target.value))} />
    </div>
  );
}

/* ── Main component ── */

export function ArtboxPlayground() {
  const [text, setText] = useState("artbox");
  const [family, setFamily] = useState<FontFamily>("default");
  const [alignment, setAlignment] = useState<Alignment>("center");
  const [letterSpacing, setLetterSpacing] = useState(0);

  const [fillMode, setFillMode] = useState<FillMode>("linear");
  const [stops, setStops] = useState<GradientStop[]>([
    { position: 0, color: "#00c8ff" },
    { position: 1, color: "#ff5a78" },
  ]);
  const [angle, setAngle] = useState(90);
  const [radius, setRadius] = useState(0.95);
  const [centerX, setCenterX] = useState(0.5);
  const [centerY, setCenterY] = useState(0.5);

  const [controlTab, setControlTab] = useState<ControlTab>("text");
  const [previewSize, setPreviewSize] = useState<TerminalSize>({
    cols: DEFAULT_PREVIEW_COLS, rows: DEFAULT_PREVIEW_ROWS,
  });
  const [displaySize, setDisplaySize] = useState<TerminalSize>({
    cols: DEFAULT_PREVIEW_COLS, rows: DEFAULT_PREVIEW_ROWS,
  });
  const [isResizing, setIsResizing] = useState(false);
  const [showSizeBadge, setShowSizeBadge] = useState(false);
  const [fadeSizeBadge, setFadeSizeBadge] = useState(false);
  const [codeOpen, setCodeOpen] = useState(false);

  const stageRef = useRef<HTMLDivElement | null>(null);
  const previewSurfaceRef = useRef<HTMLDivElement | null>(null);
  const resizeHandleRef = useRef<HTMLButtonElement | null>(null);
  const previewSizeRef = useRef<TerminalSize>({ cols: DEFAULT_PREVIEW_COLS, rows: DEFAULT_PREVIEW_ROWS });
  const cellSizeRef = useRef({ width: DEFAULT_CELL_WIDTH, height: DEFAULT_CELL_HEIGHT });
  const dragStateRef = useRef<DragState | null>(null);
  const dragPointerIdRef = useRef<number | null>(null);
  const badgeHoldTimeoutRef = useRef<number | null>(null);
  const badgeFadeTimeoutRef = useRef<number | null>(null);
  const bodyUserSelectRef = useRef("");
  const bodyCursorRef = useRef("");
  const initialFitDoneRef = useRef(false);

  /* ── Resize logic ── */

  const clearBadgeTimers = useCallback(() => {
    if (badgeHoldTimeoutRef.current !== null) { window.clearTimeout(badgeHoldTimeoutRef.current); badgeHoldTimeoutRef.current = null; }
    if (badgeFadeTimeoutRef.current !== null) { window.clearTimeout(badgeFadeTimeoutRef.current); badgeFadeTimeoutRef.current = null; }
  }, []);

  const revealSizeBadge = useCallback(() => {
    clearBadgeTimers(); setShowSizeBadge(true); setFadeSizeBadge(false);
  }, [clearBadgeTimers]);

  const scheduleSizeBadgeFade = useCallback(() => {
    clearBadgeTimers(); setShowSizeBadge(true); setFadeSizeBadge(false);
    badgeHoldTimeoutRef.current = window.setTimeout(() => {
      setFadeSizeBadge(true);
      badgeFadeTimeoutRef.current = window.setTimeout(() => {
        setShowSizeBadge(false); setFadeSizeBadge(false);
      }, SIZE_BADGE_FADE_MS);
    }, SIZE_BADGE_HOLD_MS);
  }, [clearBadgeTimers]);

  const restoreBodyStyles = useCallback(() => {
    document.body.style.userSelect = bodyUserSelectRef.current;
    document.body.style.cursor = bodyCursorRef.current;
    bodyUserSelectRef.current = ""; bodyCursorRef.current = "";
  }, []);

  const getResizeBounds = useCallback((cellOverride?: { width: number; height: number }) => {
    const stage = stageRef.current;
    const width = stage?.clientWidth ?? 0;
    const height = stage?.clientHeight ?? 0;
    if (width <= 0 || height <= 0) return {
      minCols: MIN_PREVIEW_COLS, minRows: MIN_PREVIEW_ROWS,
      maxCols: Number.POSITIVE_INFINITY, maxRows: Number.POSITIVE_INFINITY,
    };
    const cell = cellOverride ?? cellSizeRef.current;
    const cw = Math.max(1, cell.width || DEFAULT_CELL_WIDTH);
    const ch = Math.max(1, cell.height || DEFAULT_CELL_HEIGHT);
    const mc = Math.max(1, Math.floor(width / cw));
    const mr = Math.max(1, Math.floor(height / ch));
    const minCols = Math.min(MIN_PREVIEW_COLS, mc);
    const minRows = Math.min(MIN_PREVIEW_ROWS, mr);
    return { minCols, minRows, maxCols: Math.max(minCols, mc), maxRows: Math.max(minRows, mr) };
  }, []);

  const clampPreviewSize = useCallback((size: TerminalSize, cellOverride?: { width: number; height: number }): TerminalSize => {
    const b = getResizeBounds(cellOverride);
    return { cols: clamp(size.cols, b.minCols, b.maxCols), rows: clamp(size.rows, b.minRows, b.maxRows) };
  }, [getResizeBounds]);

  const applyPreviewSize = useCallback((requested: TerminalSize, opts?: {
    showBadge?: boolean; scheduleFade?: boolean; cellOverride?: { width: number; height: number };
  }) => {
    const clamped = clampPreviewSize(requested, opts?.cellOverride);
    const cur = previewSizeRef.current;
    const changed = cur.cols !== clamped.cols || cur.rows !== clamped.rows;
    if (changed) { previewSizeRef.current = clamped; setPreviewSize(clamped); setDisplaySize(clamped); }
    if (opts?.showBadge) revealSizeBadge();
    if (opts?.scheduleFade) scheduleSizeBadgeFade();
    return { changed, size: clamped };
  }, [clampPreviewSize, revealSizeBadge, scheduleSizeBadgeFade]);

  const stopResize = useCallback((event?: PointerEvent) => {
    if (event && dragPointerIdRef.current !== null && event.pointerId !== dragPointerIdRef.current) return;
    if (resizeHandleRef.current && dragPointerIdRef.current !== null &&
        resizeHandleRef.current.hasPointerCapture(dragPointerIdRef.current))
      resizeHandleRef.current.releasePointerCapture(dragPointerIdRef.current);
    const was = dragStateRef.current !== null;
    dragPointerIdRef.current = null; dragStateRef.current = null;
    setIsResizing(false); restoreBodyStyles();
    if (was) scheduleSizeBadgeFade();
  }, [restoreBodyStyles, scheduleSizeBadgeFade]);

  const onResizeHandlePointerDown = useCallback((event: React.PointerEvent<HTMLButtonElement>) => {
    event.preventDefault(); event.currentTarget.focus();
    const cw = Math.max(1, cellSizeRef.current.width || DEFAULT_CELL_WIDTH);
    const ch = Math.max(1, cellSizeRef.current.height || DEFAULT_CELL_HEIGHT);
    dragPointerIdRef.current = event.pointerId;
    dragStateRef.current = { x: event.clientX, y: event.clientY,
      cols: previewSizeRef.current.cols, rows: previewSizeRef.current.rows,
      cellWidth: cw, cellHeight: ch };
    bodyUserSelectRef.current = document.body.style.userSelect;
    bodyCursorRef.current = document.body.style.cursor;
    document.body.style.userSelect = "none"; document.body.style.cursor = "nwse-resize";
    event.currentTarget.setPointerCapture(event.pointerId);
    setIsResizing(true); revealSizeBadge();
  }, [revealSizeBadge]);

  const onResizeHandleKeyDown = useCallback((event: React.KeyboardEvent<HTMLButtonElement>) => {
    const step = event.shiftKey ? 5 : 1;
    let dc = 0, dr = 0;
    switch (event.key) {
      case "ArrowLeft": dc = -step; break; case "ArrowRight": dc = step; break;
      case "ArrowUp": dr = -step; break; case "ArrowDown": dr = step; break;
      default: return;
    }
    event.preventDefault();
    const cur = previewSizeRef.current;
    applyPreviewSize({ cols: cur.cols + dc, rows: cur.rows + dr }, { showBadge: true, scheduleFade: true });
  }, [applyPreviewSize]);

  /* ── Gradient stop helpers ── */

  const updateStop = (i: number, patch: Partial<GradientStop>) =>
    setStops((p) => p.map((s, j) => (j === i ? { ...s, ...patch } : s)));

  const addStop = () => {
    const mid = stops.length >= 2
      ? (stops[stops.length - 2].position + stops[stops.length - 1].position) / 2 : 0.5;
    setStops((p) => [...p, { position: +mid.toFixed(2), color: nextStopColor(p) }]);
  };

  const removeStop = (i: number) => {
    if (stops.length <= 2) return;
    setStops((p) => p.filter((_, j) => j !== i));
  };

  const needsStops = fillMode === "linear" || fillMode === "radial";
  const needsAngle = fillMode === "linear";
  const needsRadial = fillMode === "radial";

  /* ── WASM argv ── */

  const rerunKey = useMemo(() => JSON.stringify({
    text, family, alignment, letterSpacing,
    fillMode, stops, angle, radius, centerX, centerY,
  }), [text, family, alignment, letterSpacing, fillMode, stops, angle, radius, centerX, centerY]);

  const argv = useCallback(({ cols, rows }: { cols: number; rows: number }) => {
    setDisplaySize((p) => (p.cols === cols && p.rows === rows) ? p : { cols, rows });
    const args = [
      text.trim().length > 0 ? text : "artbox",
      String(Math.max(12, cols)), String(Math.max(6, rows - 1)),
      "--family", family, "--align", alignment,
      "--letter-spacing", String(letterSpacing),
      "--plain-fallback",
      "--fill", fillMode,
    ];
    if (fillMode === "solid") args.push("--color-a", stops[0]?.color ?? "#ffffff");
    else if (needsStops) for (const s of stops) args.push("--stop", `${s.position.toFixed(2)}:${s.color}`);
    if (needsAngle) args.push("--angle", String(angle));
    if (needsRadial) {
      args.push("--radius", radius.toFixed(2));
      args.push("--center-x", centerX.toFixed(2));
      args.push("--center-y", centerY.toFixed(2));
    }
    return args;
  }, [text, family, alignment, letterSpacing, fillMode, stops, angle, radius, centerX, centerY, needsStops, needsAngle, needsRadial]);

  /* ── Effects ── */

  useEffect(() => {
    if (!isResizing) return;
    const onMove = (e: PointerEvent) => {
      if (dragPointerIdRef.current !== null && e.pointerId !== dragPointerIdRef.current) return;
      if (!dragStateRef.current) return;
      e.preventDefault();
      const d = dragStateRef.current;
      applyPreviewSize(
        { cols: d.cols + Math.round((e.clientX - d.x) / d.cellWidth), rows: d.rows + Math.round((e.clientY - d.y) / d.cellHeight) },
        { showBadge: true, cellOverride: { width: d.cellWidth, height: d.cellHeight } },
      );
    };
    const onUp = (e: PointerEvent) => stopResize(e);
    const onBlur = () => stopResize();
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    window.addEventListener("pointercancel", onUp);
    window.addEventListener("blur", onBlur);
    return () => { window.removeEventListener("pointermove", onMove); window.removeEventListener("pointerup", onUp); window.removeEventListener("pointercancel", onUp); window.removeEventListener("blur", onBlur); };
  }, [applyPreviewSize, isResizing, stopResize]);

  useEffect(() => {
    if (!previewSurfaceRef.current) return;
    const el = previewSurfaceRef.current;
    const update = () => {
      const w = el.clientWidth, h = el.clientHeight;
      const cur = previewSizeRef.current;
      if (w <= 0 || h <= 0 || cur.cols <= 0 || cur.rows <= 0) return;
      const cell = { width: w / cur.cols, height: h / cur.rows };
      if (cell.width < 4 || cell.height < 8) return;
      cellSizeRef.current = cell;
      if (!initialFitDoneRef.current) {
        initialFitDoneRef.current = true;
        const stage = stageRef.current;
        if (stage && stage.clientWidth > 0 && stage.clientHeight > 0) {
          const bounds = getResizeBounds(cell);
          applyPreviewSize(
            { cols: bounds.maxCols, rows: bounds.maxRows },
            { cellOverride: cell },
          );
        }
      }
    };
    update();
    const obs = new ResizeObserver(update);
    obs.observe(el);
    return () => obs.disconnect();
  }, [applyPreviewSize, getResizeBounds]);

  useEffect(() => {
    if (isResizing || !stageRef.current) return;
    const el = stageRef.current;
    const clampToStage = (badge: boolean) => {
      const r = applyPreviewSize(previewSizeRef.current);
      if (badge && r.changed) { revealSizeBadge(); scheduleSizeBadgeFade(); }
    };
    clampToStage(false);
    const obs = new ResizeObserver(() => clampToStage(true));
    obs.observe(el);
    return () => obs.disconnect();
  }, [applyPreviewSize, isResizing, revealSizeBadge, scheduleSizeBadgeFade]);

  useEffect(() => () => { clearBadgeTimers(); restoreBodyStyles(); }, [clearBadgeTimers, restoreBodyStyles]);

  /* ── Rust code ── */

  const rustCode = useMemo(() => {
    const txt = text.trim().length > 0 ? text : "artbox";
    const fontsExpr = family === "default" ? "fonts::default()" : `fonts::family("${family}").unwrap()`;
    let b = [
      `let renderer = Renderer::new(${fontsExpr})`,
      `    .with_alignment(Alignment::${ALIGNMENT_RUST[alignment]})`,
    ];
    if (letterSpacing !== 0) b.push(`    .with_letter_spacing(${letterSpacing})`);
    b.push(`    .with_plain_fallback()`);
    if (fillMode === "solid") {
      const [r, g, bl] = hexToRgb(stops[0]?.color ?? "#ffffff");
      b.push(`    .with_fill(Fill::solid(Color::rgb(${r}, ${g}, ${bl})))`);
    } else if (fillMode === "linear") {
      const sr = stops.map((s) => { const [r, g, bl] = hexToRgb(s.color); return `        ColorStop::new(${s.position.toFixed(2)}, Color::rgb(${r}, ${g}, ${bl})),`; }).join("\n");
      b.push(`    .with_fill(Fill::Linear(LinearGradient::new(\n        ${angle}.0,\n        vec![\n${sr}\n        ],\n    )))`);
    } else if (fillMode === "radial") {
      const sr = stops.map((s) => { const [r, g, bl] = hexToRgb(s.color); return `        ColorStop::new(${s.position.toFixed(2)}, Color::rgb(${r}, ${g}, ${bl})),`; }).join("\n");
      b.push(`    .with_fill(Fill::Radial(RadialGradient::new(\n        (${centerX.toFixed(2)}, ${centerY.toFixed(2)}),\n        (${centerX.toFixed(2)}, ${centerY.toFixed(2)}),\n        ${radius.toFixed(2)},\n        vec![\n${sr}\n        ],\n    )))`);
    }
    const imports = [
      "fonts", "Alignment", "Artbox", "RenderTarget", "Renderer",
      ...(fillMode !== "none" ? ["Color", "Fill"] : []),
      ...(fillMode === "linear" ? ["ColorStop", "LinearGradient"] : []),
      ...(fillMode === "radial" ? ["ColorStop", "RadialGradient"] : []),
    ];
    return `use artbox::{${imports.join(", ")}};\n\n${b.join("\n")};\n\nlet art = Artbox::from_renderer(renderer);\nlet target = RenderTarget::new(cols, rows);\nlet rendered = art.render_text("${txt}", target)?;\nprint!("{}", rendered.to_ansi_string());`;
  }, [text, family, alignment, letterSpacing, fillMode, stops, angle, radius, centerX, centerY]);

  /* ── Tab content renderers ── */

  const textTab = (
    <div className="flex flex-col gap-2">
      <input type="text" value={text} onChange={(e) => setText(e.target.value)} placeholder="artbox"
        className="w-full px-2 py-[5px] rounded border border-pg-border bg-pg-surface text-pg-text text-[0.78rem] font-pg-mono outline-none" />
      <div className="flex flex-wrap gap-[3px]">
        {families.map((f) => {
          const on = family === f.value;
          return (
            <button key={f.value} type="button" onClick={() => setFamily(f.value)}
              className={`px-2 py-[3px] rounded border font-pg-mono text-[0.62rem] font-medium cursor-pointer transition-all duration-[120ms] ${
                on ? "border-pg-accent bg-pg-accent-dim text-pg-accent"
                   : "border-pg-border bg-transparent text-pg-text-dim"
              }`}>
              {f.label}
            </button>
          );
        })}
      </div>
    </div>
  );

  const layoutTab = (
    <div className="flex gap-3.5 items-end">
      {/* Alignment 3x3 grid */}
      <div>
        <span className={lblCls}>Align</span>
        <div className="grid grid-cols-3 gap-0.5 p-[3px] rounded-[5px] bg-pg-surface border border-pg-border mt-[3px]">
          {alignmentOrder.map((a) => (
            <button key={a} type="button" title={alignmentLabels[a]}
              onClick={() => setAlignment(a)}
              className={`size-3 rounded-full border-none cursor-pointer p-0 transition-all duration-[120ms] ${
                alignment === a
                  ? "bg-pg-accent shadow-[0_0_0_2px_var(--color-pg-accent-mid)]"
                  : "bg-pg-border shadow-none"
              }`} />
          ))}
        </div>
      </div>
      {/* Spacing */}
      <Slider label="Spacing" value={letterSpacing} min={-2} max={4} step={1}
        onChange={setLetterSpacing} />
    </div>
  );

  const fillTab = (
    <div className="flex flex-col gap-1.5">
      {/* Top row: segmented + angle/radius inline */}
      <div className="flex gap-2.5 items-end">
        <div className="inline-flex rounded border border-pg-border overflow-hidden bg-pg-surface shrink-0">
          {fillModes.map((o, i) => {
            const on = fillMode === o.value;
            return (
              <button key={o.value} type="button" onClick={() => setFillMode(o.value)}
                className={`px-[9px] py-[3px] border-none font-pg-mono text-[0.6rem] font-medium cursor-pointer transition-all duration-[120ms] ${
                  i > 0 ? "border-l border-l-pg-border" : ""
                } ${on ? "bg-pg-accent-dim text-pg-accent" : "bg-transparent text-pg-text-dim"}`}>
                {o.label}
              </button>
            );
          })}
        </div>
        {needsAngle && <Slider label="Angle" value={angle} min={0} max={360} step={1} suffix="°" onChange={setAngle} />}
        {needsRadial && <Slider label="Radius" value={radius} min={0.1} max={2} step={0.01} onChange={setRadius} />}
      </div>

      {/* Stops */}
      {(fillMode === "solid" || needsStops) && (
        <div className="flex flex-col gap-1">
          {(fillMode === "solid" ? stops.slice(0, 1) : stops).map((stop, i) => (
            <div key={i} className="flex gap-1.5 items-center">
              <input type="color" value={stop.color} onChange={(e) => updateStop(i, { color: e.target.value })} />
              {needsStops && (
                <>
                  <div className="flex-1">
                    <input type="range" min={0} max={1} step={0.01} value={stop.position}
                      onChange={(e) => updateStop(i, { position: Number(e.target.value) })} />
                  </div>
                  <span className={`${valCls} w-[26px] text-right`}>{stop.position.toFixed(2)}</span>
                  {stops.length > 2 && (
                    <button type="button" onClick={() => removeStop(i)} title="Remove"
                      className="bg-transparent border-none text-pg-text-dim cursor-pointer text-[0.8rem] leading-none p-0">
                      &times;
                    </button>
                  )}
                </>
              )}
            </div>
          ))}
          {needsStops && (
            <button type="button" onClick={addStop}
              className="self-start bg-transparent border border-pg-border rounded text-pg-text-dim cursor-pointer text-[0.58rem] font-pg-mono font-medium px-2 py-[2px] uppercase tracking-[0.06em]">
              + stop
            </button>
          )}
        </div>
      )}

      {/* Radial extras */}
      {needsRadial && (
        <div className="flex gap-2.5">
          <Slider label="Center X" value={centerX} min={0} max={1} step={0.01} onChange={setCenterX} />
          <Slider label="Center Y" value={centerY} min={0} max={1} step={0.01} onChange={setCenterY} />
        </div>
      )}
    </div>
  );

  const tabs: { id: ControlTab; label: string }[] = [
    { id: "text", label: "Text" },
    { id: "layout", label: "Layout" },
    { id: "fill", label: "Fill" },
  ];

  /* ── Render ── */

  return (
    <div className="artbox-pg not-content mt-2 mb-4">
      <div className="rounded-[10px] border border-pg-border overflow-hidden">
        {/* ── Terminal preview ── */}
        <div className="relative" style={{ height: PREVIEW_HEIGHT, overflow: "hidden" }}>
          <div ref={stageRef} className="relative w-full h-full grid place-items-center bg-pg-terminal-bg"
            style={{ overflow: "hidden" }}>
            {showSizeBadge && (
              <div className={`absolute top-1.5 right-1.5 z-[2] pointer-events-none font-pg-mono text-[0.58rem] tabular-nums text-[#dce6ff] bg-[rgba(12,16,24,0.85)] border border-[#1e2b3a] rounded px-[5px] py-[1px] tracking-[0.02em] backdrop-blur-[4px] transition-opacity duration-[220ms] ease-out ${
                fadeSizeBadge ? "opacity-0" : "opacity-100"
              }`}>
                {displaySize.cols}&times;{displaySize.rows}
              </div>
            )}

            <div className="relative inline-block rounded-sm border border-pg-border">
              <div ref={previewSurfaceRef} className="inline-block">
                <TuiPreview key={rerunKey} wasm={wasmUrl} argv={argv}
                  mode="static" fit="none" size={previewSize}
                  terminal={{
                    fontSize: 14,
                    fontFamily: "Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace",
                    theme: terminalTheme,
                  }} />
              </div>
              <button ref={resizeHandleRef} type="button"
                aria-label="Resize terminal preview" title="Resize terminal preview"
                onPointerDown={onResizeHandlePointerDown} onKeyDown={onResizeHandleKeyDown}
                className={`absolute left-1/2 -translate-x-1/2 -bottom-2.5 z-[3] h-[14px] w-[36px] inline-flex items-center justify-center p-0 rounded-[3px] border border-pg-text-dim bg-pg-surface text-pg-text-mid cursor-[ns-resize] touch-none transition-opacity duration-150 text-[7px] font-pg-mono leading-none tracking-[1px] ${
                  isResizing ? "opacity-100" : "opacity-80"
                }`}>
                <span aria-hidden>···</span>
              </button>
            </div>
          </div>
        </div>

        {/* ── Control tabs ── */}
        <div className="flex items-center justify-between border-t border-pg-border bg-pg-surface px-0.5">
          <div className="flex">
            {tabs.map((t) => {
              const on = controlTab === t.id;
              return (
                <button key={t.id} type="button" onClick={() => setControlTab(t.id)}
                  className={`px-3 py-1.5 border-none bg-transparent font-pg-mono text-[0.62rem] font-medium cursor-pointer transition-[color,border-color] duration-[120ms] -mb-px border-b-2 ${
                    on ? "border-b-pg-accent text-pg-text" : "border-b-transparent text-pg-text-dim"
                  }`}>
                  {t.label}
                </button>
              );
            })}
          </div>
          <span className={`${valCls} pr-1.5 text-[0.56rem]`}>
            {displaySize.cols}&times;{displaySize.rows}
          </span>
        </div>

        {/* ── Tab content ── */}
        <div className="border-t border-pg-border bg-pg-panel px-2.5 py-2">
          {controlTab === "text" && textTab}
          {controlTab === "layout" && layoutTab}
          {controlTab === "fill" && fillTab}
        </div>
      </div>

      {/* ── Code (collapsible, outside main card) ── */}
      <details open={codeOpen} onToggle={(e) => setCodeOpen((e.target as HTMLDetailsElement).open)}
        className="mt-1.5">
        <summary className="inline-flex items-center gap-1 font-pg-mono text-[0.58rem] font-medium uppercase tracking-[0.06em] text-pg-text-dim cursor-pointer px-1 py-0.5">
          <span className={`inline-block transition-transform duration-150 ${codeOpen ? "rotate-90" : ""}`}>&#9656;</span>
          Rust code
        </summary>
        <pre className="mt-1 overflow-auto rounded-lg border border-pg-border bg-pg-bg text-pg-text p-3 text-[0.7rem] leading-[1.5] whitespace-pre font-pg-mono">
          <code>{rustCode}</code>
        </pre>
      </details>
    </div>
  );
}
