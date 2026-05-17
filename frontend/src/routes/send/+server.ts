import { Resend } from 'resend';
import { RESEND_API_KEY } from '$env/static/private';
import { PUBLIC_PREFIX } from '$env/static/public';
import type { RequestHandler } from './$types';

const resend = new Resend(RESEND_API_KEY);

interface SendBody {
	to: string;
	link: string;
	fileName?: string;
}

export const POST: RequestHandler = async ({ request }) => {
	let body: SendBody;
	try {
		body = await request.json();
	} catch {
		return Response.json({ error: 'Invalid JSON body' }, { status: 400 });
	}

	if (!body.to || !body.link) {
		return Response.json({ error: 'Missing required fields: to, link' }, { status: 400 });
	}

	// Basic email format validation
	if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(body.to)) {
		return Response.json({ error: 'Invalid email address' }, { status: 400 });
	}

	const fileName = body.fileName || 'a file';
	const appName = 'filez.zone';
	const appUrl = PUBLIC_PREFIX || 'https://filez.zone';

	const html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
</head>
<body style="margin:0;padding:0;background-color:#f8fafc;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
  <table width="100%" cellpadding="0" cellspacing="0" style="background-color:#f8fafc;padding:40px 0;">
    <tr>
      <td align="center">
        <table width="480" cellpadding="0" cellspacing="0" style="background-color:#ffffff;border-radius:16px;border:1px solid #e2e8f0;box-shadow:0 4px 6px -1px rgba(0,0,0,0.05);">
          <!-- Header -->
          <tr>
            <td style="padding:32px 40px 0;text-align:center;">
              <span style="font-size:20px;font-weight:700;color:#1e293b;">📎 ${appName}</span>
            </td>
          </tr>
          <!-- Body -->
          <tr>
            <td style="padding:24px 40px 32px;">
              <h2 style="margin:0 0 8px;font-size:18px;font-weight:600;color:#1e293b;">Someone shared a file with you</h2>
              <p style="margin:0 0 8px;font-size:14px;color:#64748b;line-height:1.6;">
                <strong style="color:#334155;">${escapeHtml(fileName)}</strong> was shared securely via ${appName}.
                The file is end-to-end encrypted and will be deleted after the first download.
              </p>
              <!-- CTA Button -->
              <table cellpadding="0" cellspacing="0" style="margin:20px 0;">
                <tr>
                  <td align="center" style="background:linear-gradient(135deg,#8b5cf6,#7c3aed);border-radius:10px;">
                    <a href="${escapeHtml(body.link)}" style="display:inline-block;padding:12px 32px;font-size:14px;font-weight:600;color:#ffffff;text-decoration:none;">Download file →</a>
                  </td>
                </tr>
              </table>
              <p style="margin:0 0 16px;font-size:12px;color:#94a3b8;line-height:1.5;">
                Or copy and paste this link into your browser:<br>
                <code style="display:inline-block;margin-top:4px;padding:4px 8px;background:#f1f5f9;border-radius:4px;font-size:11px;color:#475569;word-break:break-all;">${escapeHtml(body.link)}</code>
              </p>
              <!-- Warning -->
              <table width="100%" cellpadding="0" cellspacing="0" style="background-color:#fef3c7;border:1px solid #fcd34d;border-radius:8px;">
                <tr>
                  <td style="padding:12px 16px;">
                    <p style="margin:0;font-size:12px;color:#92400e;line-height:1.5;">
                      ⚠️ <strong>Burn after reading:</strong> This file will be permanently deleted after it is downloaded. Make sure to save it before opening.
                    </p>
                  </td>
                </tr>
              </table>
            </td>
          </tr>
          <!-- Footer -->
          <tr>
            <td style="padding:0 40px 32px;text-align:center;">
              <p style="margin:0;font-size:11px;color:#94a3b8;">
                Sent via <a href="${escapeHtml(appUrl)}" style="color:#8b5cf6;text-decoration:none;">${appName}</a> — Secure end-to-end encrypted file sharing
              </p>
            </td>
          </tr>
        </table>
      </td>
    </tr>
  </table>
</body>
</html>`;

	const { data, error } = await resend.emails.send({
		from: `${appName} <no-reply@filez.zone>`,
		to: [body.to],
		subject: `${body.fileName || 'A file'} was shared with you via ${appName}`,
		html
	});

	if (error) {
		console.error('Resend error:', error);
		return Response.json({ error: 'Failed to send email' }, { status: 500 });
	}

	return Response.json({ success: true, id: data?.id });
};

function escapeHtml(str: string): string {
	return str
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;');
}
