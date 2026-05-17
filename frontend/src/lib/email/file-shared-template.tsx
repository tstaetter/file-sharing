import * as React from "react";

interface FileSharedTemplateProps {
  fileName: string;
  link: string;
  appName: string;
  appUrl: string;
}

export const FileSharedTemplate: React.FC<FileSharedTemplateProps> = ({
  fileName,
  link,
  appName,
  appUrl,
}) => {
  const container: React.CSSProperties = {
    margin: 0,
    padding: "40px 0",
    backgroundColor: "#f8fafc",
    fontFamily:
      "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
  };

  const card: React.CSSProperties = {
    width: 480,
    backgroundColor: "#ffffff",
    borderRadius: 16,
    border: "1px solid #e2e8f0",
    boxShadow: "0 4px 6px -1px rgba(0,0,0,0.05)",
  };

  const header: React.CSSProperties = {
    padding: "32px 40px 0",
    textAlign: "center",
  };

  const body: React.CSSProperties = {
    padding: "24px 40px 32px",
  };

  const heading: React.CSSProperties = {
    margin: "0 0 8px",
    fontSize: 18,
    fontWeight: 600,
    color: "#1e293b",
  };

  const paragraph: React.CSSProperties = {
    margin: "0 0 8px",
    fontSize: 14,
    color: "#64748b",
    lineHeight: 1.6,
  };

  const ctaButton: React.CSSProperties = {
    display: "inline-block",
    padding: "12px 32px",
    fontSize: 14,
    fontWeight: 600,
    color: "#ffffff",
    textDecoration: "none",
    background: "linear-gradient(135deg, #8b5cf6, #7c3aed)",
    borderRadius: 10,
  };

  const codeBlock: React.CSSProperties = {
    display: "inline-block",
    marginTop: 4,
    padding: "4px 8px",
    background: "#f1f5f9",
    borderRadius: 4,
    fontSize: 11,
    color: "#475569",
    wordBreak: "break-all",
  };

  const warning: React.CSSProperties = {
    backgroundColor: "#fef3c7",
    border: "1px solid #fcd34d",
    borderRadius: 8,
    padding: "12px 16px",
  };

  const footer: React.CSSProperties = {
    padding: "0 40px 32px",
    textAlign: "center",
  };

  return (
    <table width="100%" cellPadding={0} cellSpacing={0} style={container}>
      <tr>
        <td align="center">
          <table width={480} cellPadding={0} cellSpacing={0} style={card}>
            {/* Header */}
            <tr>
              <td style={header}>
                <span
                  style={{ fontSize: 20, fontWeight: 700, color: "#1e293b" }}
                >
                  📎 {appName}
                </span>
              </td>
            </tr>
            {/* Body */}
            <tr>
              <td style={body}>
                <h2 style={heading}>Someone shared a file with you</h2>
                <p style={paragraph}>
                  <strong style={{ color: "#334155" }}>{fileName}</strong> was
                  shared securely via {appName}. The file is end-to-end
                  encrypted and will be deleted after the first download.
                </p>
                {/* CTA Button */}
                <table cellPadding={0} cellSpacing={0} style={{ margin: "20px 0" }}>
                  <tr>
                    <td align="center">
                      <a href={link} style={ctaButton}>
                        Download file →
                      </a>
                    </td>
                  </tr>
                </table>
                <p
                  style={{
                    margin: "0 0 16px",
                    fontSize: 12,
                    color: "#94a3b8",
                    lineHeight: 1.5,
                  }}
                >
                  Or copy and paste this link into your browser:
                  <br />
                  <code style={codeBlock}>{link}</code>
                </p>
                {/* Warning */}
                <table width="100%" cellPadding={0} cellSpacing={0}>
                  <tr>
                    <td style={warning}>
                      <p
                        style={{
                          margin: 0,
                          fontSize: 12,
                          color: "#92400e",
                          lineHeight: 1.5,
                        }}
                      >
                        ⚠️ <strong>Burn after reading:</strong> This file will
                        be permanently deleted after it is downloaded. Make
                        sure to save it before opening.
                      </p>
                    </td>
                  </tr>
                </table>
              </td>
            </tr>
            {/* Footer */}
            <tr>
              <td style={footer}>
                <p style={{ margin: 0, fontSize: 11, color: "#94a3b8" }}>
                  Sent via{" "}
                  <a
                    href={appUrl}
                    style={{ color: "#8b5cf6", textDecoration: "none" }}
                  >
                    {appName}
                  </a>{" "}
                  — Secure end-to-end encrypted file sharing
                </p>
              </td>
            </tr>
          </table>
        </td>
      </tr>
    </table>
  );
};

export default FileSharedTemplate;
