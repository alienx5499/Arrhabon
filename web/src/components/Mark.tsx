// Seal mark: a gold coin under a glass bell jar. Inline copy of the brand logo.svg so it stays crisp at any size.
export function Mark({ size = 36 }: { size?: number }) {
  return (
    <svg viewBox="26 26 548 548" width={size} height={size} aria-hidden="true" focusable="false">
      <circle cx="300" cy="300" r="274" fill="#151d30" />
      <circle cx="300" cy="300" r="250" fill="none" stroke="#f7f5f0" strokeWidth="13" />
      <circle cx="300" cy="149" r="25" fill="#f7f5f0" />
      <rect x="292" y="168" width="16" height="16" fill="#f7f5f0" />
      <path d="M206 438 V284 A94 94 0 0 1 394 284 V438" fill="none" stroke="#f7f5f0" strokeWidth="19" strokeLinejoin="round" />
      <rect x="175" y="430" width="250" height="23" rx="10" fill="#f7f5f0" />
      <circle cx="299" cy="379" r="52" fill="#e0a526" />
    </svg>
  );
}
