function WindowChrome() {
  return (
    <header
      aria-label="Window chrome"
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '12px 16px',
        borderBottom: '1px solid rgba(255, 255, 255, 0.12)',
      }}
    >
      <strong>RustFiles</strong>
      <span style={{ fontSize: 12, opacity: 0.7 }}>Window controls placeholder</span>
    </header>
  );
}

export default WindowChrome;
