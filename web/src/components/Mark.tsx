// Seal mark: a gold coin under a glass bell jar. Placeholder until public/logo.png is delivered; swap the SVG for <Image src="/logo.png" /> then.
export function Mark({ size = 36 }: { size?: number }) {
  return (
    <svg viewBox="0 0 40 40" width={size} height={size} aria-hidden="true" focusable="false">
      <circle cx="20" cy="20" r="20" fill="#151d30" />
      <circle cx="20" cy="20" r="17.25" fill="none" stroke="#f7f5f0" strokeOpacity="0.35" strokeWidth="0.75" />
      <circle cx="20" cy="10.1" r="1.5" fill="#f7f5f0" />
      <path d="M12.5 28.5V19.5a7.5 7.5 0 0 1 15 0v9" fill="none" stroke="#f7f5f0" strokeWidth="1.6" strokeLinejoin="round" />
      <path d="M10.5 29h19" stroke="#f7f5f0" strokeWidth="1.6" strokeLinecap="round" />
      <circle cx="20" cy="24.6" r="3.4" fill="#e0a526" />
    </svg>
  );
}
