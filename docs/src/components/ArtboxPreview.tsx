import { TuiPreview } from "@dkkoval/tui-preview";

const terminalTheme = {
  background: "#0b1020",
  foreground: "#dce6ff",
  cursor: "#7aa2f7",
  selectionBackground: "#223355",
  selectionForeground: "#dce6ff",
};

const baseUrl = import.meta.env.BASE_URL.endsWith("/")
  ? import.meta.env.BASE_URL
  : `${import.meta.env.BASE_URL}/`;

const wasmUrl = `${baseUrl}wasm/artbox-preview.wasm`;

export function ArtboxPreview() {
  return (
    <div
      style={{
        width: "100%",
        height: "100px",
        borderRadius: 10,
        overflow: "hidden",
        border:
          "1px solid color-mix(in srgb, var(--sl-color-gray-4), transparent 35%)",
      }}
    >
      <TuiPreview
        wasm={wasmUrl}
        argv={({ cols, rows }) => ["artbox", String(cols), String(rows)]}
        mode="static"
        fit="container"
        terminal={{
          fontSize: 14,
          fontFamily:
            "Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace",
          theme: terminalTheme,
        }}
        style={{ width: "100%", height: "100%" }}
      />
    </div>
  );
}
