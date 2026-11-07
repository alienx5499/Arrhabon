import { ImageResponse } from "next/og";

// shortcut: drawn card until the poster art lands; delete this file when app/opengraph-image.png is added.
export const alt = "Arrhabon: a tuition deposit that refunds itself.";
export const size = { width: 1200, height: 630 };
export const contentType = "image/png";

export default function OpengraphImage() {
  return new ImageResponse(
    (
      <div style={{ width: "100%", height: "100%", display: "flex", background: "#f7f5f0", color: "#151d30" }}>
        <div style={{ flex: 1, display: "flex", flexDirection: "column", justifyContent: "center", padding: "0 72px" }}>
          <div style={{ fontSize: 44, fontWeight: 700, display: "flex", alignItems: "center", gap: 20 }}>
            <div style={{ width: 56, height: 56, borderRadius: 28, background: "#151d30", display: "flex", alignItems: "center", justifyContent: "center" }}>
              <div style={{ width: 18, height: 18, borderRadius: 9, background: "#e0a526" }} />
            </div>
            Arrhabon
          </div>
          <div style={{ fontSize: 76, fontWeight: 700, lineHeight: 1.02, letterSpacing: -2, marginTop: 40 }}>
            A tuition deposit that refunds itself.
          </div>
        </div>
        <div style={{ width: 360, background: "#1f2a44", display: "flex", alignItems: "flex-end", justifyContent: "center", paddingBottom: 150 }}>
          <div style={{ width: 150, height: 200, border: "6px solid #f7f5f0", borderBottom: "none", borderRadius: "75px 75px 0 0", display: "flex", alignItems: "flex-end", justifyContent: "center", paddingBottom: 24 }}>
            <div style={{ width: 56, height: 56, borderRadius: 28, background: "#e0a526" }} />
          </div>
        </div>
      </div>
    ),
    size,
  );
}
